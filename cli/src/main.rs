mod cli;

use blackbox::parser::Frame;
use blackbox::Log;
use clap::Parser;
use cli::Cli;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use std::process::{ExitCode, Termination};

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
        .with_max_level(cli.log_level_filter())
        .init();

    if cli.logs.len() > 1 && cli.stdout {
        tracing::error!("cannot write multiple logs to stdout");
        return QuietResult::err(exitcode::USAGE);
    }

    let config = cli.to_blackbox_config();

    for filename in cli.logs {
        let span = tracing::info_span!("file", name = %filename.display());
        let _span = span.enter();

        let data = match read_log_file(&filename) {
            Ok(data) => data,
            Err(error) => {
                tracing::error!(%error, "failed to read log file");
                return QuietResult::err(exitcode::IOERR);
            }
        };

        let file = blackbox::File::new(&data);

        let log_count = file.log_count();
        if cli.stdout && log_count > 1 {
            tracing::error!("found {log_count} logs, choose one to write to stdout with `--index`");
            return QuietResult::err(exitcode::USAGE);
        }

        for i in 0..log_count {
            let index = i + 1;

            let span = tracing::info_span!("log", index);
            let _span = span.enter();

            let log = match file.parse_index(&config, i) {
                Ok(log) => log,
                Err(_) => return QuietResult::err(exitcode::DATAERR),
            };

            let mut out = match get_output(cli.stdout, &filename, index) {
                Ok(out) => BufWriter::new(out),
                Err(error) => {
                    tracing::error!(%error, "failed to open output file");
                    return QuietResult::err(exitcode::CANTCREAT);
                }
            };

            if let Err(error) = write_csv(&mut out, &log) {
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

fn write_csv(out: &mut impl Write, log: &Log) -> io::Result<()> {
    write_header(out, log)?;

    for frame in log.main_frames() {
        out.write_all(frame.iteration().to_string().as_bytes())?;
        write!(out, ",")?;
        out.write_all(frame.time().to_string().as_bytes())?;

        for s in frame.values().iter().map(ToString::to_string) {
            write!(out, ",")?;
            out.write_all(s.as_bytes())?;
        }

        writeln!(out)?;
    }

    out.flush()
}

fn write_header(out: &mut impl Write, log: &Log) -> io::Result<()> {
    let mut fields = log.headers().main_fields();
    out.write_all(fields.next().unwrap().as_bytes())?;

    for s in fields {
        write!(out, ",")?;
        out.write_all(s.as_bytes())?;
    }

    writeln!(out)
}
