mod cli;

use blackbox::parser::{FieldDef, FrameDefs};
use blackbox::Log;
use clap::Parser;
use cli::Cli;
use itertools::Itertools;
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

    for log in cli.logs {
        let out: Box<dyn Write> = if cli.stdout {
            Box::new(io::stdout().lock())
        } else {
            let mut name = log.clone();
            name.set_extension("csv");
            Box::new(File::create(name)?)
        };
        let mut out = BufWriter::new(out);

        let data = {
            let mut log = File::open(log)?;
            let mut data = Vec::new();
            log.read_to_end(&mut data)?;
            data
        };

        let log = config.parse(&data)?;

        write_header(&mut out, &log)?;

        for frame in log.main_frames() {
            for s in Itertools::intersperse(frame.iter().map(ToString::to_string), ",".to_owned()) {
                out.write_all(s.as_bytes())?;
            }

            writeln!(out)?;
        }

        out.flush()?;
    }

    Ok(())
}

fn write_header(out: &mut impl Write, log: &Log) -> io::Result<()> {
    let FrameDefs { intra, .. } = &log.headers().frames;

    for s in Itertools::intersperse(intra.iter().map(FieldDef::name), ",") {
        out.write_all(s.as_bytes())?;
    }

    writeln!(out)
}
