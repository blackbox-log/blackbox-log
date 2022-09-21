mod cli;

use blackbox::parser::Frame;
use blackbox::Log;
use clap::Parser;
use cli::Cli;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(cli.log_level_filter())
        .init();

    if cli.logs.len() > 1 && cli.stdout {
        tracing::error!("cannot write multiple logs to stdout");
        return Ok(());
    }

    let config = cli.to_blackbox_config();

    for filename in cli.logs {
        let data = {
            let mut log = File::open(&filename)?;
            let mut data = Vec::new();
            log.read_to_end(&mut data)?;
            data
        };

        for (i, log) in blackbox::parse_file(&config, &data)
            .enumerate()
            .map(|(i, log)| (i + 1, log))
        {
            let log = log?;

            let out: Box<dyn Write> = if cli.stdout {
                Box::new(io::stdout().lock())
            } else {
                let mut out = filename.clone();
                out.set_extension(format!("{i:0>2}.csv"));
                tracing::info!(
                    "Writing log {i} from '{}' to '{}'",
                    filename.display(),
                    out.display()
                );
                Box::new(File::create(out)?)
            };
            let mut out = BufWriter::new(out);

            write_header(&mut out, &log)?;

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

            out.flush()?;
        }
    }

    Ok(())
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
