#![allow(clippy::print_stdout)]

use bpaf::{Bpaf, Parser};
use serde::Deserialize;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use xshell::{cmd, Result, Shell};

fn main() -> Result<()> {
    let sh = Shell::new()?;
    let args = args().run();

    {
        let _push_dir = sh.push_dir(get_root(&sh)?);
        cmd!(sh, "cargo fmt").run()?;
    }

    match args {
        // Already done in above block
        Args::Fmt => Ok(()),

        Args::Check { all } => {
            let lints = get_root(&sh)?.join("Cranky.toml");
            let lints = sh.read_file(lints)?;
            let lints: Lints = toml::from_str(&lints).unwrap();
            let lints = lints.into_clippy_args();

            let workspace = all.then_some("--workspace");

            cmd!(
                sh,
                "cargo clippy {workspace...} --all-targets -- {lints...}"
            )
            .quiet()
            .run()
        }

        Args::Test { coverage, args } => {
            if coverage {
                cmd!(sh, "cargo llvm-cov --package blackbox nextest --html").run()
            } else {
                cmd!(sh, "cargo nextest run --package blackbox {args...}").run()
            }
        }

        Args::Bench { test, args } => {
            if test {
                cmd!(sh, "cargo criterion --package blackbox --benches -- --test").run()
            } else {
                cmd!(sh, "cargo criterion --package blackbox --benches {args...}").run()
            }
        }

        Args::Profile {
            bench,
            filter,
            time,
            name,
        } => {
            let unixtime = get_unixtime();
            let output = get_root(&sh)?.join("target/profile");
            sh.create_dir(&output)?;
            let output = output.join(name.unwrap_or_else(|| format!("{bench}-{unixtime}.svg")));

            let time = time.to_string();

            cmd!(sh, "cargo flamegraph --dev --package blackbox --deterministic --palette rust --output {output} --bench {bench} -- --bench --profile-time {time} {filter}")
                .env("CARGO_PROFILE_BENCH_DEBUG", "true")
                .run()
        }

        Args::Run { release, args } => {
            let release = release.then_some("--release");

            let root = get_root(&sh)?;
            let cli_toml = root.join("cli/Cargo.toml");

            cmd!(
                sh,
                "cargo run {release...} --manifest-path {cli_toml} -- {args...}"
            )
            .run()
        }

        Args::Fuzz(fuzz) => {
            let fuzz_dir = get_root(&sh)?.join("fuzz");

            let dir_args = [
                "--fuzz-dir",
                fuzz_dir
                    .to_str()
                    .expect("valid Unicode path to <repo>/fuzz"),
            ];

            match fuzz {
                Fuzz::Add { target } => {
                    fn update_manifest(path: &Path, target: &str) -> io::Result<()> {
                        let mut f = File::open(path)?;

                        writeln!(f)?;
                        writeln!(f, "[[dir]]")?;
                        writeln!(f, "name = \"{target}\"")?;
                        writeln!(f, "path = \"fuzz_targets/{target}.rs\"")?;
                        writeln!(f, "test = false")?;
                        writeln!(f, "doc = false")
                    }

                    fn write_target(path: &Path) -> io::Result<()> {
                        let mut f = File::open(path)?;

                        writeln!(f, "#![no_main]")?;
                        writeln!(f)?;
                        writeln!(
                            f,
                            "use blackbox_fuzz::{{encoding, fuzz_target, UnalignedBytes}};"
                        )?;
                        writeln!(f)?;
                        writeln!(f, "fuzz_target!(|data: UnalignedBytes| {{")?;
                        writeln!(
                            f,
                            "    let (mut reference, mut biterator) = data.to_streams().unwrap();"
                        )?;
                        writeln!(f)?;
                        writeln!(f, "    assert_eq!(todo!(), todo!());")?;
                        writeln!(f, "}});")
                    }

                    update_manifest(&fuzz_dir.join("Cargo.toml"), &target)
                        .expect("updated fuzz/Cargo.toml");
                    let targets_dir = fuzz_dir.join("fuzz_targets");
                    sh.create_dir(&targets_dir)?;
                    write_target(&targets_dir.join(format!("{target}.rs")))
                        .expect("wrote fuzz target");

                    Ok(())
                }

                Fuzz::List => cmd!(sh, "cargo fuzz list {dir_args...}").run(),

                Fuzz::Run(FuzzRun {
                    target,
                    time,
                    backtrace,
                    input,
                }) => {
                    let total_time = time.map(|t| format!("-max_total_time={t}"));
                    let debug = backtrace.then_some("--debug");

                    let cmd = cmd!(
                        sh,
                        "cargo +nightly fuzz run {debug...} {dir_args...} {target} {input...} -- {total_time...}"
                    );

                    let cmd = if backtrace {
                        cmd.env("RUST_BACKTRACE", "1")
                    } else {
                        cmd
                    };

                    cmd.run()
                }

                Fuzz::Fmt { target, input } => {
                    cmd!(sh, "cargo +nightly fuzz fmt {dir_args...} {target} {input}").run()
                }

                Fuzz::Cov { target } => {
                    cmd!(sh, "cargo +nightly fuzz coverage {dir_args...} {target}").run()?;

                    let sysroot = cmd!(sh, "rustc +nightly --print target-libdir")
                        .quiet()
                        .read()?
                        .parse::<PathBuf>()
                        .unwrap()
                        .join("..")
                        .canonicalize()
                        .unwrap();
                    let cov = sysroot.join("bin/llvm-cov");

                    let coverage_dir = sh.current_dir().join("coverage").join(&target);
                    let profdata = coverage_dir.join("coverage.profdata");

                    let triple = sysroot.file_name().unwrap();
                    let bin = sh.current_dir().join("target").join(triple);
                    let bin = bin.join("coverage").join(triple);
                    let bin = bin.join("release").join(&target);

                    cmd!(sh, "{cov} show --format=html --instr-profile={profdata} --output-dir={coverage_dir} --ignore-filename-regex=^/rustc|/\\.cargo/ {bin}")
                        .quiet()
                        .run()?;

                    let index = coverage_dir.join("index.html");
                    println!("Saved coverage to {}", index.display());

                    Ok(())
                }

                Fuzz::Min { target, input } => match input {
                    Some(input) => cmd!(
                        sh,
                        "cargo +nightly fuzz tmin {dir_args...} {target} {input}"
                    )
                    .run(),
                    None => cmd!(
                        sh,
                        "cargo +nightly fuzz cmin {dir_args...} {target} corpus/{target}"
                    )
                    .run(),
                },
            }
        }

        Args::Install => {
            let tools = [
                "cargo-criterion",
                "cargo-fuzz",
                "cargo-llvm-cov",
                "cargo-nextest",
                "flamegraph",
            ];

            cmd!(sh, "cargo install --locked -- {tools...}").run()
        }
    }
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
enum Args {
    #[bpaf(command)]
    /// Formats all code in the workspace
    Fmt,

    #[bpaf(command)]
    /// Runs clippy for linting
    Check {
        #[bpaf(short, long, switch)]
        /// Lint entire workspace
        all: bool,
    },

    #[bpaf(command)]
    /// Runs nextest tests for `blackbox` lib
    Test {
        #[bpaf(switch)]
        /// Generates a coverage report while running tests
        coverage: bool,

        #[bpaf(positional)]
        /// Arguments for nextest
        args: Vec<OsString>,
    },

    #[bpaf(command)]
    /// Runs benchmarks for `blackbox` lib
    Bench {
        #[bpaf(long, switch)]
        /// Tests all benchmarks run successfully, ignores any extra args for criterion
        test: bool,

        #[bpaf(positional)]
        /// Arguments for criterion
        args: Vec<OsString>,
    },

    #[bpaf(command)]
    /// Generates a flamegraph chart for a benchmark
    Profile {
        #[bpaf(positional("bench"))]
        /// Which benchmark binary to run
        bench: String,

        #[bpaf(positional("filter"))]
        /// Filter to pass to the benchmark binary, see criterion docs
        filter: String,

        #[bpaf(short, long, argument("seconds"), fallback(10))]
        /// How long the benchmark should run (default: 10)
        time: u16,

        #[bpaf(short, long, argument("name"))]
        /// Overrides the default name ({bench}-{unixtime})
        name: Option<String>,
    },

    #[bpaf(command)]
    /// Runs `blackbox_decode`
    ///
    ///
    /// NB: pass `-- -h` to get help for `blackbox_decode`
    Run {
        #[bpaf(short, long, switch)]
        /// Run in release mode
        release: bool,

        #[bpaf(positional)]
        /// Passthrough arguments
        args: Vec<OsString>,
    },

    Fuzz(#[bpaf(external(fuzz_with_default_run))] Fuzz),

    #[bpaf(command)]
    /// Installs necessary dev tools
    Install,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(command)]
/// Manage & run fuzz targets
enum Fuzz {
    #[bpaf(command)]
    /// Sets up a new fuzz target
    Add {
        #[bpaf(positional("target"))]
        /// Sets the name of the new target
        target: String,
    },

    #[bpaf(command)]
    /// List all fuzz targets
    List,

    #[bpaf(command)]
    /// Runs a fuzz target
    Run(#[bpaf(external(fuzz_run))] FuzzRun),

    #[bpaf(command)]
    /// Pretty-prints the failing input
    Fmt {
        #[bpaf(positional("target"))]
        target: String,

        #[bpaf(positional("input"))]
        input: PathBuf,
    },

    #[bpaf(command)]
    /// Generates a coverage report for a target
    Cov {
        #[bpaf(positional("target"))]
        target: String,
    },

    #[bpaf(command)]
    /// Minimizes an input if provided, else minimizes the number of inputs in the corpus
    Min {
        #[bpaf(positional("target"))]
        target: String,

        #[bpaf(positional("input"))]
        input: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(generate(fuzz_run))]
struct FuzzRun {
    #[bpaf(positional("target"))]
    target: String,

    #[bpaf(external)]
    time: Option<u16>,

    #[bpaf(long, switch)]
    /// Runs in debug mode and prints a backtrace on panic
    backtrace: bool,

    #[bpaf(positional("input"))]
    /// Runs the target on only this input, if given
    input: Option<PathBuf>,
}

fn fuzz_with_default_run() -> impl bpaf::Parser<Fuzz> {
    let fuzz_run = fuzz_run().map(Fuzz::Run);
    bpaf::construct!([fuzz(), fuzz_run])
}

fn time_given() -> impl Parser<Option<u16>> {
    bpaf::long("time")
        .help("Passes -max_total_time=<seconds> to libFuzzer, defaulting to 15 minutes if passed without a value")
        .argument("seconds")
        .from_str()
        .map(Some)
}

fn time_default() -> impl Parser<Option<u16>> {
    bpaf::long("time").switch().hide().map(|x| x.then_some(900))
}

fn time() -> impl Parser<Option<u16>> {
    bpaf::construct!([time_given(), time_default()])
}

fn get_root(sh: &Shell) -> Result<PathBuf> {
    let path = cmd!(sh, "git rev-parse --show-toplevel").quiet().read()?;
    Ok(path.parse().unwrap())
}

fn get_unixtime() -> u64 {
    use std::time::SystemTime;

    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Debug, Deserialize)]
struct Lints {
    #[serde(default)]
    allow: Vec<String>,

    #[serde(default)]
    warn: Vec<String>,

    #[serde(default)]
    deny: Vec<String>,

    #[serde(default)]
    forbid: Vec<String>,
}

impl Lints {
    fn into_clippy_args(self) -> Vec<String> {
        let groups = [
            ("A", self.allow),
            ("W", self.warn),
            ("D", self.deny),
            ("F", self.forbid),
        ];

        let mut args = Vec::new();
        for (level, mut lints) in groups {
            for lint in lints.drain(..) {
                args.push(format!("-{level}"));
                args.push(lint);
            }
        }

        args
    }
}
