#![allow(clippy::print_stdout)]

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use bpaf::{Bpaf, Parser};
use serde::Deserialize;
use xshell::{cmd, Result, Shell};

fn main() -> Result<()> {
    let sh = Shell::new()?;
    let args = args().run();

    let is_ci = env::var("CI").is_ok();

    if !is_ci {
        let _push_dir = sh.push_dir(get_root(&sh)?);

        cmd!(sh, "cargo +nightly fmt").run()?;
    }

    match args {
        // Already done in above block
        Args::Fmt => Ok(()),

        Args::Check { workspace, args } => {
            fn run(cmd: xshell::Cmd, lints: &[String]) -> Result<()> {
                eprintln!("$ {cmd}");
                let cmd = cmd.arg("--").args(lints);
                cmd.quiet().run()
            }

            let root = get_root(&sh)?;

            let lints = root.join("Cranky.toml");
            let lints = sh.read_file(lints)?;
            let lints: Lints = toml::from_str(&lints).unwrap();
            let lints = lints.into_clippy_args();

            run(
                cmd!(sh, "cargo clippy --all-targets")
                    .args(get_workspace_args(workspace))
                    .args(&args),
                &lints,
            )?;

            if workspace || sh.current_dir() == root {
                run(
                    cmd!(sh, "cargo clippy --no-default-features --package blackbox").args(args),
                    &lints,
                )
            } else {
                Ok(())
            }
        }

        Args::Test {
            coverage,
            quiet,
            args,
        } => {
            let ci = if is_ci { "--profile=ci" } else { "" };
            let quiet = if !is_ci && quiet {
                "--status-level=leak"
            } else {
                ""
            };

            if coverage {
                if is_ci {
                    #[rustfmt::skip]
                    let cmd = cmd!(sh, "cargo llvm-cov --package blackbox --lcov --output-path coverage.lcov nextest {ci}");
                    cmd.run()
                } else {
                    cmd!(
                        sh,
                        "cargo llvm-cov --package blackbox --html nextest {quiet}"
                    )
                    .run()
                }
            } else {
                let workspace = get_workspace_args(true);
                cmd!(sh, "cargo nextest run {workspace...} {ci} {quiet}")
                    .args(args)
                    .run()
            }
        }

        Args::Bench { test, args } => {
            if test {
                cmd!(sh, "cargo criterion --package blackbox --benches -- --test").run()
            } else {
                cmd!(sh, "cargo criterion --package blackbox --benches")
                    .args(args)
                    .run()
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

            #[rustfmt::skip]
            let cmd = cmd!(
                sh,
                "cargo flamegraph --package blackbox --deterministic --palette rust --output {output} --bench {bench} -- --bench --profile-time {time} {filter}"
            );
            cmd.env("CARGO_PROFILE_BENCH_DEBUG", "true").run()
        }

        Args::Fuzz(fuzz) => {
            let root_dir = get_root(&sh)?;
            let fuzz_dir = root_dir.join("fuzz");

            let dir_args = [
                "--fuzz-dir",
                fuzz_dir
                    .to_str()
                    .expect("valid Unicode path to <repo>/fuzz"),
            ];

            match fuzz {
                Fuzz::List => cmd!(sh, "cargo fuzz list {dir_args...}").run(),

                Fuzz::Run {
                    target,
                    time,
                    backtrace,
                    input,
                } => {
                    let total_time = time.map(|t| format!("-max_total_time={t}"));
                    let debug = backtrace.then_some("--dev");

                    #[rustfmt::skip]
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

                    let coverage_dir = fuzz_dir.join("coverage").join(&target);
                    let profdata = coverage_dir.join("coverage.profdata");

                    let triple = sysroot.file_name().unwrap();
                    let bin = root_dir.join("target").join(triple);
                    let bin = bin.join("coverage").join(triple);
                    let bin = bin.join("release").join(&target);

                    #[rustfmt::skip]
                    let cmd = cmd!(
                        sh,
                        "{cov} show --format=html --instr-profile={profdata} --output-dir={coverage_dir} --ignore-filename-regex=^/rustc|/\\.cargo/ {bin}"
                    );
                    cmd.quiet().run()?;

                    let index = coverage_dir.join("index.html");
                    println!("Saved coverage to {}", index.display());

                    Ok(())
                }

                Fuzz::Min { target, input } => {
                    let (command, final_arg) = input.map_or_else(
                        || ("cmin", fuzz_dir.join("corpus").join(&target)),
                        |input| ("tmin", input),
                    );

                    cmd!(
                        sh,
                        "cargo +nightly fuzz {command} {dir_args...} {target} {final_arg}"
                    )
                    .run()
                }
            }
        }

        Args::Install => {
            let tools = [
                "cargo-criterion",
                "cargo-fuzz",
                "cargo-llvm-cov",
                "cargo-nextest",
                "flamegraph",
                "typos-cli",
            ];

            cmd!(sh, "cargo install --locked --").args(tools).run()
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
        #[bpaf(short, long)]
        /// Lint entire workspace
        workspace: bool,

        #[bpaf(positional)]
        /// Extra arguments for clippy
        args: Vec<OsString>,
    },

    #[bpaf(command)]
    /// Runs nextest tests
    Test {
        /// Generates a coverage report while running tests (only for
        /// `blackbox`)
        coverage: bool,

        #[bpaf(short, long)]
        /// Hide output for successful tests
        quiet: bool,

        #[bpaf(positional)]
        /// Arguments for nextest
        args: Vec<OsString>,
    },

    #[bpaf(command)]
    /// Runs benchmarks for `blackbox` lib
    Bench {
        /// Tests all benchmarks run successfully, ignores any extra args for
        /// criterion
        test: bool,

        #[bpaf(positional)]
        /// Arguments for criterion
        args: Vec<OsString>,
    },

    #[bpaf(command)]
    /// Generates a flamegraph chart for a benchmark
    Profile {
        #[bpaf(short, long, argument("seconds"), fallback(10))]
        /// How long the benchmark should run (default: 10)
        time: u16,

        #[bpaf(short, long, argument("name"))]
        /// Overrides the default name ({bench}-{unixtime})
        name: Option<String>,

        #[bpaf(positional("bench"))]
        /// Which benchmark binary to run
        bench: String,

        #[bpaf(positional("filter"))]
        /// Filter to pass to the benchmark binary, see criterion docs
        filter: String,
    },

    Fuzz(#[bpaf(external(fuzz))] Fuzz),

    #[bpaf(command)]
    /// Installs necessary dev tools
    Install,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(command)]
/// Manage & run fuzz targets
enum Fuzz {
    #[bpaf(command)]
    /// List all fuzz targets
    List,

    #[bpaf(command)]
    /// Runs a fuzz target
    Run {
        #[bpaf(external)]
        time: Option<u16>,

        /// Runs in debug mode and prints a backtrace on panic
        backtrace: bool,

        #[bpaf(positional("target"))]
        target: String,

        #[bpaf(positional("input"))]
        /// Runs the target on only this input, if given
        input: Option<PathBuf>,
    },

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
    /// Minimizes an input if provided, else minimizes the number of inputs in
    /// the corpus
    Min {
        #[bpaf(positional("target"))]
        target: String,

        #[bpaf(positional("input"))]
        input: Option<PathBuf>,
    },
}

fn time_given() -> impl Parser<Option<u16>> {
    let help = "Passes -max_total_time=<seconds> to libFuzzer, defaulting to 15 minutes if passed \
                without a value";

    bpaf::long("time")
        .help(help)
        .argument::<u16>("seconds")
        .optional()
}

fn time_default() -> impl Parser<Option<u16>> {
    bpaf::long("time").flag(Some(900), None).hide()
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

fn get_workspace_args(workspace: bool) -> Vec<&'static str> {
    if workspace {
        let mut args = vec!["--workspace"];

        if cfg!(not(target_os = "linux")) {
            let packages = ["blackbox-fuzz", "blackbox-sys"];

            for package in packages {
                args.push("--exclude");
                args.push(package);
            }
        }

        args
    } else {
        vec![]
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bpaf_invariants() {
        args().check_invariants(true);
    }
}
