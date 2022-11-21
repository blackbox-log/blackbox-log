#![allow(clippy::default_trait_access)]

use std::path::PathBuf;

use bpaf::{construct, Bpaf, Parser};
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, private, version)]
#[allow(unused, clippy::default_trait_access)]
pub(crate) struct Cli {
    #[bpaf(external)]
    pub index: Vec<usize>,

    /// Prints the limits and range of each field (TODO)
    pub limits: bool,

    /// Writes log to stdout instead of a file
    pub stdout: bool,

    #[bpaf(external)]
    pub altitude_offset: i16,

    #[bpaf(external)]
    pub gps: GpsFormats,

    // TODO
    // #[arg(long)]
    // /// Set magnetic declination in degrees.minutes format (e.g. -12.58 for New York)
    // declination: (),
    //
    // TODO
    // #[arg(long)]
    // /// Set magnetic declination in decimal degrees (e.g. -12.97 for New York)
    // declination_dec: (),
    #[bpaf(external)]
    pub filter: Option<Vec<String>>,

    #[bpaf(external)]
    pub verbosity: LevelFilter,

    // TODO: accept - for stdin
    /// One or more logs to parse
    #[bpaf(
        positional("log"),
        guard(at_least_one, "at least one log file is required")
    )]
    pub logs: Vec<PathBuf>,
}

fn index() -> impl Parser<Vec<usize>> {
    let help = "Chooses which log(s) should be decoded or omit to decode all\n(applies to all \
                logs & can be repeated)";

    bpaf::short('i')
        .long("index")
        .help(help)
        .argument::<usize>("index")
        .many()
        .map(|mut v| {
            v.sort_unstable();
            v.dedup();
            v
        })
}

fn at_least_one(logs: &Vec<PathBuf>) -> bool {
    !logs.is_empty()
}

fn altitude_offset() -> impl Parser<i16> {
    let old = bpaf::long("alt-offset").argument::<i16>("").hide();
    let new = bpaf::long("altitude-offset")
        .help("Sets the altitude offset in meters (TODO)")
        .argument::<i16>("offset");

    construct!([new, old]).fallback(0)
}

fn filter() -> impl Parser<Option<Vec<String>>> {
    bpaf::short('f')
        .long("filter")
        .help("Selects fields to output by name, excluding any index or units\n(comma separated)")
        .argument::<String>("fields")
        .map(|s| {
            s.split(',')
                .into_iter()
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .guard(
            |set| !set.is_empty(),
            "filter must contain at least one field",
        )
        .optional()
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GpsFormats {
    pub merged: bool,
    pub separate: bool,
    pub gpx: bool,
}

fn gps() -> impl Parser<GpsFormats> {
    bpaf::long("gps")
        .help("One or more formats to write GPS data (merged, separate (csv), gpx)")
        .argument::<String>("format")
        .guard(
            |s| {
                matches!(
                    s.to_ascii_lowercase().as_str(),
                    "merged" | "separate" | "gpx"
                )
            },
            "expected either separate or gpx",
        )
        .many()
        .map(|v| GpsFormats {
            merged: v.iter().any(|s| s == "merged"),
            separate: v.iter().any(|s| s == "separate"),
            gpx: v.iter().any(|s| s == "gpx"),
        })
}

fn verbosity() -> impl Parser<LevelFilter> {
    const DEFAULT: usize = if cfg!(debug_assertions) { 4 } else { 3 };
    const LEVELS: [LevelFilter; 6] = [
        LevelFilter::OFF,
        LevelFilter::ERROR,
        LevelFilter::WARN,
        LevelFilter::INFO,
        LevelFilter::DEBUG,
        LevelFilter::TRACE,
    ];

    fn plural(x: usize) -> &'static str {
        if x == 1 { "" } else { "s" }
    }

    let debug = bpaf::long("debug").switch().hide();

    let max = DEFAULT;
    let help = format!("Reduces debug output up to {max} time{}", plural(max));
    let quiet = bpaf::short('q')
        .long("quiet")
        .help(help)
        .req_flag(())
        .many()
        .map(|v| v.len());

    let max = LEVELS.len() - DEFAULT - 1;
    let help = format!("Increases debug output up to {max} time{}", plural(max));
    let verbose = bpaf::short('v')
        .long("verbose")
        .help(help)
        .req_flag(())
        .many()
        .map(|v| v.len());

    construct!(debug, verbose, quiet).map(|(debug, v, q)| {
        if debug {
            LEVELS[LEVELS.len() - 1]
        } else {
            LEVELS[(v + DEFAULT).saturating_sub(q).min(LEVELS.len() - 1)]
        }
    })
}

impl Cli {
    pub fn parse() -> Self {
        cli()
            .usage("Usage: blackbox_decode [options] <log>...")
            .run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bpaf_invariants() {
        cli().check_invariants(true);
    }
}
