mod cli;

use blackbox::parser::{FieldDef, FrameDefs};
use blackbox::Log;
use clap::Parser;
use cli::Cli;
use itertools::Itertools;
use std::fs::File;
use std::io::{self, Read, Write};

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(cli.log_level_filter())
        .init();

    let config = cli.to_blackbox_config();

    for log in cli.logs {
        let data = {
            let mut log = File::open(log)?;
            let mut data = Vec::new();
            log.read_to_end(&mut data)?;
            data
        };

        let log = config.parse(&data)?;
        let mut csv = File::create("out.csv")?;

        write_header(&mut csv, &log)?;

        for frame in log.main_frames() {
            for s in Itertools::intersperse(frame.iter().map(ToString::to_string), ",".to_owned()) {
                csv.write_all(s.as_bytes())?;
            }

            writeln!(csv)?;
        }
    }

    Ok(())
}

fn write_header(csv: &mut File, log: &Log) -> io::Result<()> {
    let FrameDefs { intra, .. } = &log.headers().frames;

    for s in Itertools::intersperse(intra.iter().map(FieldDef::name), ",") {
        csv.write_all(s.as_bytes())?;
    }

    writeln!(csv)
}
