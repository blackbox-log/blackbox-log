mod cli;

use blackbox::Log;
use clap::Parser;
use cli::Cli;
use std::fs::File;
use std::io::Read;

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(cli.log_level_filter())
        .init();

    for log in cli.logs {
        let data = {
            let mut log = File::open(log)?;
            let mut data = Vec::new();
            log.read_to_end(&mut data)?;
            data
        };

        let _log = Log::new(&data)?;
        // dbg!(log);
    }

    Ok(())
}
