mod cli;

use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::process;

use blackbox_log::log::LogView;
use blackbox_log::units::si;
use blackbox_log::{Log, Value};
use mimalloc::MiMalloc;
use rayon::prelude::*;

use self::cli::{Action, Cli};

#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

fn main() {
    let parser = lexopt::Parser::from_env();
    let bin = parser
        .bin_name()
        .unwrap_or(env!("CARGO_BIN_NAME"))
        .to_owned();

    let cli = match Cli::parse(parser) {
        Ok(Action::Run(cli)) => cli,
        Ok(Action::Help) => {
            cli::print_help(&bin);
            process::exit(exitcode::OK);
        }
        Ok(Action::Version) => {
            cli::print_version();
            process::exit(exitcode::OK);
        }
        #[allow(clippy::print_stderr)]
        Err(err) => {
            eprintln!("{err}");
            process::exit(exitcode::USAGE);
        }
    };

    tracing_subscriber::fmt()
        .with_max_level(cli.verbosity)
        .init();

    if let Err(err) = cli.validate() {
        tracing::error!("{err}");
        process::exit(exitcode::USAGE);
    }

    let result = cli.logs.par_iter().try_for_each(|filename| {
        let span = tracing::info_span!("file", name = ?filename);
        let _span = span.enter();

        let data = fs::read(filename).map_err(|error| {
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

            let mut log = file.get_reader(i);
            let log = Log::parse(&mut log).map_err(|err| {
                tracing::debug!("error from parse_by_index: {err}");
                exitcode::DATAERR
            })?;

            let data = {
                let mut data = if cli.gps.merged {
                    log.merged_data()
                } else {
                    log.data()
                };

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

            if cli.gps.separate {
                let mut out = get_output(cli.stdout, filename, human_i, "gps.csv")?;
                if let Err(error) = write_csv(&mut out, &data) {
                    tracing::error!(%error, "failed to write gps csv");
                    return Err(exitcode::IOERR);
                }
            }

            Ok(())
        })
    });

    if let Err(code) = result {
        process::exit(code);
    }
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

fn write_csv<'v: 'd, 'd, V: LogView<'v, 'd>>(out: &mut impl Write, log: &'v V) -> io::Result<()>
where
    V::Value: Into<Value>,
{
    write_csv_line(out, log.fields().map(|(name, _unit)| name))?;

    for frame in log.values() {
        write_csv_line(
            out,
            frame.map(|value| match value.into() {
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
