mod cli;

use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::process;

use blackbox_log::data::ParseEvent;
use blackbox_log::frame::{Frame as _, FrameDef as _, GpsFrame, MainFrame, SlowFrame};
use blackbox_log::units::si;
use blackbox_log::{DataParser, Headers, Value};
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

        (0..file.log_count()).into_par_iter().try_for_each(|i| {
            let human_i = i + 1;

            let span = tracing::info_span!("log", index = human_i);
            let _span = span.enter();

            let mut log = file.get_reader(i);
            let headers = Headers::parse(&mut log).map_err(|err| {
                tracing::debug!("header parse error: {err}");
                exitcode::DATAERR
            })?;

            if cli.filter.is_some() {
                todo!("filters");
            }

            let field_names = headers
                .main_def()
                .iter_names()
                .chain(headers.slow_def().iter_names());

            let mut out = get_output(filename, human_i, "csv")?;
            if let Err(error) = write_csv_line(&mut out, field_names) {
                tracing::error!(%error, "failed to write csv header");
                return Err(exitcode::IOERR);
            }

            let mut gps_out = match headers.gps_def() {
                Some(def) if cli.gps => {
                    let mut out = get_output(filename, human_i, "gps.csv")?;

                    if let Err(error) = write_csv_line(&mut out, def.iter_names()) {
                        tracing::error!(%error, "failed to write gps csv header");
                        return Err(exitcode::IOERR);
                    }

                    Some(out)
                }
                _ => None,
            };

            let mut parser = DataParser::new(&mut log, &headers);
            let mut slow: String = ",".repeat(headers.slow_def().len().saturating_sub(1));
            while let Some(frame) = parser.next() {
                match frame {
                    ParseEvent::Event(_) => {}
                    ParseEvent::Slow(frame) => {
                        slow.clear();
                        write_slow_frame(&mut slow, frame);
                    }
                    ParseEvent::Main(main) => {
                        if let Err(error) = write_main_frame(&mut out, main, &slow) {
                            tracing::error!(%error, "failed to write csv");
                            return Err(exitcode::IOERR);
                        }
                    }
                    ParseEvent::Gps(gps) => {
                        if let Some(ref mut out) = gps_out {
                            if let Err(error) = write_gps_frame(out, gps) {
                                tracing::error!(%error, "failed to write gps csv");
                                return Err(exitcode::IOERR);
                            }
                        }
                    }
                }
            }

            if let Err(error) = out.flush() {
                tracing::error!(%error, "failed to flush csv");
                return Err(exitcode::IOERR);
            }

            if let Some(Err(error)) = gps_out.map(|mut out| out.flush()) {
                tracing::error!(%error, "failed to flush gps csv");
                return Err(exitcode::IOERR);
            }

            Ok(())
        })
    });

    if let Err(code) = result {
        process::exit(code);
    }
}

fn get_output(
    filename: &Path,
    index: usize,
    extension: &str,
) -> Result<BufWriter<File>, exitcode::ExitCode> {
    let mut out = filename.to_owned();
    out.set_extension(format!("{index:0>2}.{extension}"));

    let file = File::create(&out).map_err(|error| {
        tracing::error!(%error, file = %out.display(), "failed to open output file");
        exitcode::CANTCREAT
    })?;

    tracing::info!("Writing to '{}'", out.display());

    Ok(BufWriter::new(file))
}

fn write_main_frame(out: &mut impl Write, main: MainFrame, slow: &str) -> io::Result<()> {
    let mut fields = main.iter().map(|v| format_value(v.into()));

    if let Some(first) = fields.next() {
        out.write_all(first.as_bytes())?;

        for field in fields {
            out.write_all(b",")?;
            out.write_all(field.as_bytes())?;
        }

        if !slow.is_empty() {
            out.write_all(b",")?;
        }
    }

    out.write_all(slow.as_bytes())?;
    out.write_all(b"\n")
}

fn write_slow_frame(out: &mut String, slow: SlowFrame) {
    let mut fields = slow.iter().map(|v| format_value(v.into()));

    if let Some(first) = fields.next() {
        out.push_str(&first);

        for field in fields {
            out.push(',');
            out.push_str(&field);
        }
    }
}

fn write_gps_frame(out: &mut impl Write, gps: GpsFrame) -> io::Result<()> {
    write_csv_line(out, gps.iter().map(Value::from).map(format_value))
}
fn format_value(value: Value) -> String {
    fn format_float(f: f64) -> String {
        format!("{f:.2}")
    }

    match value {
        Value::FrameTime(t) => format!("{:.0}", t.get::<si::time::microsecond>()),
        Value::Amperage(a) => format_float(a.get::<si::electric_current::ampere>()),
        Value::Voltage(v) => format_float(v.get::<si::electric_potential::volt>()),
        Value::Acceleration(a) => {
            format_float(a.get::<si::acceleration::meter_per_second_squared>())
        }
        Value::Rotation(r) => format_float(r.get::<si::angular_velocity::degree_per_second>()),
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
    }
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

    out.write_all(b"\n")?;

    Ok(())
}
