use blackbox::{Log, ParseResult};
use std::fs::File;

fn main() -> ParseResult<()> {
    let data = File::open("flight2only.bbl")?;

    let logs = Log::new(data);
    dbg!(logs)?;

    Ok(())
}
