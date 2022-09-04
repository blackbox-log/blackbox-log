mod cli;

use blackbox::Log;
use clap::Parser;
use cli::Cli;
use std::fs::File;

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(cli.log_level_filter())
        .init();

    for log in cli.logs {
        let log = File::open(log)?;

        let _log = Log::new(log)?;
        // dbg!(log);
    }

    Ok(())
}
