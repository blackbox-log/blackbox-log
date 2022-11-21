mod cli;

use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use std::process::{ExitCode, Termination};

use blackbox_log::log::{LogView, MainView};
use blackbox_log::parser::Value;
use blackbox_log::units::si;
use mimalloc::MiMalloc;
use rayon::prelude::*;

use self::cli::Cli;

#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

#[derive(Debug)]
enum QuietResult<T> {
    Ok(T),
    Err(ExitCode),
}

impl<T> From<exitcode::ExitCode> for QuietResult<T> {
    fn from(code: exitcode::ExitCode) -> Self {
        Self::Err(ExitCode::from(u8::try_from(code).unwrap()))
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
        return QuietResult::from(exitcode::USAGE);
    }

    let result = cli.logs.par_iter().try_for_each(|filename| {
        let span = tracing::info_span!("file", name = ?filename);
        let _span = span.enter();

        let data = read_log_file(filename).map_err(|error| {
            tracing::error!(%error, "failed to read log file");
            exitcode::IOERR
        })?;

        let file = blackbox_log::File::new(&data);

        let log_count = file.log_count();
        if cli.stdout && log_count > 1 && cli.index.len() != 1 {
            tracing::error!(
                "found {log_count} logs, choose exactly one to write to stdout with `--index`"
            );
            return Err(exitcode::USAGE);
        }

        (0..log_count).into_par_iter().try_for_each(|i| {
            let human_i = i + 1;

            let span = tracing::info_span!("log", index = human_i);
            let _span = span.enter();

            let log = file.parse_by_index(i).map_err(|err| {
                tracing::debug!("error from parse_by_index: {err}");
                exitcode::DATAERR
            })?;

            let data = {
                let mut data = log.merged_data();
                if let Some(filter) = &cli.filter {
                    data.update_filter(filter);
                }
                data
            };

            let mut out = get_output(cli.stdout, filename, human_i, "csv")?;
            if let Err(error) = write_csv(&mut out, &data) {
                tracing::error!(%error, "failed to write csv");
                return Err(exitcode::IOERR);
            }

            Ok(())
        })
    });

    if let Err(code) = result {
        QuietResult::from(code)
    } else {
        QuietResult::Ok(())
    }
}

fn read_log_file(filename: &Path) -> io::Result<Vec<u8>> {
    let mut log = File::open(filename)?;
    let mut data = Vec::new();
    log.read_to_end(&mut data)?;
    Ok(data)
}

fn get_output(
    stdout: bool,
    filename: &Path,
    index: usize,
    extension: &str,
) -> Result<BufWriter<Box<dyn Write>>, exitcode::ExitCode> {
    let out: Box<dyn Write> = if stdout {
        Box::new(io::stdout().lock())
    } else {
        let mut out = filename.to_owned();
        out.set_extension(format!("{index:0>2}.{extension}"));

        let file = File::create(&out).map_err(|error| {
            tracing::error!(%error, file = %out.display(), "failed to open output file");
            exitcode::CANTCREAT
        })?;

        tracing::info!("Writing to '{}'", out.display());
        Box::new(file)
    };

    Ok(BufWriter::new(out))
}

fn write_csv(out: &mut impl Write, log: &MainView) -> io::Result<()> {
    write_csv_line(out, log.fields().map(|(name, _unit)| name))?;

    for frame in log.values() {
        write_csv_line(
            out,
            frame.map(|value| match value {
                Value::FrameTime(t) => format!("{:.0}", t.get::<si::time::microsecond>()),
                Value::Amperage(a) => format_float(a.get::<si::electric_current::ampere>()),
                Value::Voltage(v) => format_float(v.get::<si::electric_potential::volt>()),
                Value::Acceleration(a) => {
                    format_float(a.get::<si::acceleration::meter_per_second_squared>())
                }
                Value::Rotation(r) => {
                    format_float(r.get::<si::angular_velocity::degree_per_second>())
                }
                Value::FlightMode(f) => f.to_string(),
                Value::State(s) => s.to_string(),
                Value::FailsafePhase(f) => f.to_string(),
                Value::Boolean(b) => b.to_string(),
                Value::GpsCoordinate(c) => format!("{:.7}", c),
                Value::Altitude(a) => format!("{:.0}", a.get::<si::length::meter>()),
                Value::Velocity(v) => format_float(v.get::<si::velocity::meter_per_second>()),
                Value::GpsHeading(h) => format!("{h:.1}"),
                Value::Unsigned(u) => u.to_string(),
                Value::Signed(s) => s.to_string(),
                Value::Missing => String::new(),
            }),
        )?;
    }

    out.flush()
}

fn format_float(f: f64) -> String {
    format!("{f:.2}")
}

fn write_csv_line<T: AsRef<str>>(
    out: &mut impl Write,
    mut fields: impl Iterator<Item = T>,
) -> io::Result<()> {
    if let Some(first) = fields.next() {
        out.write_all(first.as_ref().as_bytes())?;

        for s in fields {
            out.write_all(b",")?;
            out.write_all(s.as_ref().as_bytes())?;
        }
    }

    out.write_all(b"\n")
}
