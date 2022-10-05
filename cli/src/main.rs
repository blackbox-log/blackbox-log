mod cli;

use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use std::process::{ExitCode, Termination};

use blackbox::units::Unit;
use blackbox::Log;

use self::cli::Cli;

#[derive(Debug)]
enum QuietResult<T> {
    Ok(T),
    Err(ExitCode),
}

impl<T> QuietResult<T> {
    const FAILURE: Self = Self::Err(ExitCode::FAILURE);

    fn err(code: exitcode::ExitCode) -> Self {
        Self::Err(ExitCode::from(u8::try_from(code).unwrap()))
    }
}

impl<T> From<blackbox::parser::ParseResult<T>> for QuietResult<T> {
    fn from(result: blackbox::parser::ParseResult<T>) -> Self {
        match result {
            Ok(ok) => Self::Ok(ok),
            Err(_) => Self::FAILURE,
        }
    }
}

impl Termination for QuietResult<()> {
    fn report(self) -> ExitCode {
        match self {
            Self::Ok(()) => ExitCode::SUCCESS,
            Self::Err(code) => code,
        }
    }
}

fn main() -> QuietResult<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(cli.verbosity)
        .init();

    if cli.logs.len() > 1 && cli.stdout {
        tracing::error!("cannot write multiple logs to stdout");
        return QuietResult::err(exitcode::USAGE);
    }

    let config = cli.to_blackbox_config();

    for filename in &cli.logs {
        let span = tracing::info_span!("file", name = ?filename);
        let _span = span.enter();

        let data = match read_log_file(filename) {
            Ok(data) => data,
            Err(error) => {
                tracing::error!(%error, "failed to read log file");
                return QuietResult::err(exitcode::IOERR);
            }
        };

        let file = blackbox::File::new(&data);

        let log_count = file.log_count();
        if cli.stdout && log_count > 1 && cli.index.len() != 1 {
            tracing::error!(
                "found {log_count} logs, choose exactly one to write to stdout with `--index`"
            );
            return QuietResult::err(exitcode::USAGE);
        }

        for i in 0..log_count {
            let human_i = i + 1;

            let span = tracing::info_span!("log", index = human_i);
            let _span = span.enter();

            let log = match file.parse_by_index(&config, i) {
                Ok(log) => log,
                Err(_) => return QuietResult::err(exitcode::DATAERR),
            };

            let mut out = match get_output(cli.stdout, filename, human_i) {
                Ok(out) => BufWriter::new(out),
                Err(error) => {
                    tracing::error!(%error, "failed to open output file");
                    return QuietResult::err(exitcode::CANTCREAT);
                }
            };

            if let Err(error) = write_csv(&mut out, &log, &cli) {
                tracing::error!(%error, "failed to write csv");
                return QuietResult::err(exitcode::IOERR);
            }
        }
    }

    QuietResult::Ok(())
}

fn read_log_file(filename: &Path) -> io::Result<Vec<u8>> {
    let mut log = File::open(&filename)?;
    let mut data = Vec::new();
    log.read_to_end(&mut data)?;
    Ok(data)
}

fn get_output(stdout: bool, filename: &Path, index: usize) -> io::Result<Box<dyn Write>> {
    if stdout {
        Ok(Box::new(io::stdout().lock()))
    } else {
        let mut out = filename.to_owned();
        out.set_extension(format!("{index:0>2}.csv"));
        tracing::info!("Writing log to '{}'", out.display());
        Ok(Box::new(File::create(out)?))
    }
}

fn write_csv(out: &mut impl Write, log: &Log, config: &Cli) -> io::Result<()> {
    write_csv_line(
        out,
        log.fields().map(|(name, unit)| {
            let unit = config.get_unit(unit);
            if config.raw || unit.is_raw() {
                name.to_owned()
            } else {
                format!("{name} ({unit})")
            }
        }),
    )?;

    for frame in log.iter_frames() {
        write_csv_line(
            out,
            frame.map(|x| match x {
                Unit::FrameTime(t) => config.unit_frame_time.format(t),
                Unit::Amperage(a) => config.unit_amperage.format(a),
                Unit::Voltage(v) => config.unit_vbat.format(v),
                Unit::Acceleration(a) => config.unit_acceleration.format(a),
                Unit::Rotation(r) => config.unit_rotation.format(r),
                Unit::Unitless(x) => x.to_string(),
            }),
        )?;
    }

    out.flush()
}

fn write_csv_line(
    out: &mut impl Write,
    mut fields: impl Iterator<Item = String>,
) -> io::Result<()> {
    if let Some(first) = fields.next() {
        out.write_all(first.as_bytes())?;

        for s in fields {
            write!(out, ",")?;
            out.write_all(s.as_bytes())?;
        }
    }

    writeln!(out)
}
