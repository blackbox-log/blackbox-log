#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::ffi::OsString;
use std::path::PathBuf;
use std::{env, fs};

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

            let workspace_args = get_workspace_args(workspace, false);

            run(
                cmd!(sh, "cargo clippy").args(&workspace_args).args(&args),
                &lints,
            )?;

            if !is_ci {
                let _env = sh.push_env("RUSTFLAGS", "--cfg bench");
                run(
                    cmd!(sh, "cargo clippy --benches").args(&workspace_args),
                    &lints,
                )?;
            }

            if !is_ci && workspace_args.iter().any(|s| s.contains("fuzz")) {
                let _env = sh.push_env("RUSTFLAGS", "--cfg fuzzing");
                run(cmd!(sh, "cargo clippy --package blackbox-fuzz"), &lints)?;
            }

            if !is_ci && (workspace || sh.current_dir() == root) {
                run(
                    cmd!(
                        sh,
                        "cargo clippy --package blackbox-log --no-default-features"
                    )
                    .args(args),
                    &lints,
                )
            } else {
                Ok(())
            }
        }

        Args::Doc { private, deps } => {
            let private = private.then_some("--document-private-items");
            let no_deps = (!deps).then_some("--no-deps");

            cmd!(sh, "cargo doc -p blackbox-log {no_deps...} {private...}").run()
        }

        Args::Test {
            coverage,
            ignored,
            quiet,
            args,
        } => {
            let ci = if is_ci { "--profile=ci" } else { "" };
            let quiet = if quiet { "--status-level=leak" } else { "" };
            let ignored = if ignored || coverage {
                "--run-ignored=all"
            } else {
                ""
            };

            if coverage {
                let base = cmd!(sh, "cargo llvm-cov --package blackbox-log --all-features");

                if is_ci {
                    base.args(&[
                        "--lcov",
                        "--output-path=coverage.lcov",
                        "nextest",
                        ci,
                        ignored,
                    ])
                    .run()
                } else {
                    base.args(&["--html", "nextest", quiet, ignored]).run()
                }
            } else {
                let workspace = get_workspace_args(true, false);

                cmd!(
                    sh,
                    "cargo nextest run --all-features {workspace...} {ci} {quiet} {ignored}"
                )
                .args(args)
                .run()
            }
        }

        Args::Bench { test, args } => {
            let _env = sh.push_env("RUSTFLAGS", "--cfg bench");

            if test {
                cmd!(
                    sh,
                    "cargo criterion --package blackbox-log --benches -- --test"
                )
                .run()
            } else {
                let bench = if args.is_empty() {
                    "--benches"
                } else {
                    "--bench"
                };

                cmd!(sh, "cargo criterion --package blackbox-log")
                    .arg(bench)
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
                "cargo flamegraph --package blackbox-log --deterministic --palette rust --output {output} --bench {bench} -- --bench --profile-time {time} {filter}"
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
                Fuzz::List => {
                    let dir = fuzz_dir.join("src/bin");

                    for entry in fs::read_dir(dir).unwrap().map(Result::unwrap) {
                        let path = entry.path();
                        let name = path.file_stem().unwrap();
                        println!("{}", name.to_string_lossy());
                    }

                    Ok(())
                }

                Fuzz::Check { target } => {
                    cmd!(sh, "cargo +nightly fuzz check {dir_args...} {target...}").run()
                }

                Fuzz::Run {
                    target,
                    time,
                    jobs,
                    backtrace,
                    input,
                } => {
                    let total_time = time.map(|t| format!("-max_total_time={t}"));
                    let debug = backtrace.then_some("--dev");
                    let jobs = jobs.map(|jobs| format!("--jobs={jobs}"));

                    #[rustfmt::skip]
                    let cmd = cmd!(
                        sh,
                        "cargo +nightly fuzz run {debug...} {jobs...} {dir_args...} {target} {input...} -- {total_time...}"
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
                "cargo-insta",
                "cargo-llvm-cov",
                "cargo-nextest",
                "flamegraph",
                "typos-cli",
            ];

            cmd!(sh, "cargo install --locked --").args(tools).run()
        }

        Args::Wasm => {
            let _push_dir = sh.push_dir(get_root(&sh)?);

            let target = "wasm32-unknown-unknown";
            cmd!(
                sh,
                "cargo +nightly build --package blackbox-log-wasm --target {target} --release"
            )
            .env("RUSTFLAGS", "-C target-feature=+bulk-memory,+sign-ext")
            .run()?;

            #[rustfmt::skip]
            cmd!(
                sh,
                "wasm-opt -O3 target/{target}/release/blackbox_log_wasm.wasm -o blackbox-log-js/src/blackbox-log.wasm --enable-multivalue --enable-bulk-memory --enable-sign-ext"
            )
            .run()?;

            Ok(())
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
    /// Builds the documentation
    Doc {
        #[bpaf(long)]
        /// Include docs for private items
        private: bool,

        #[bpaf(long)]
        /// Include docs for dependencies
        deps: bool,
    },

    #[bpaf(command)]
    /// Runs nextest tests
    Test {
        /// Generates a coverage report while running tests (only for
        /// `blackbox-log`). Implies --ignored
        coverage: bool,

        /// Additionally run ignored tests
        ignored: bool,

        #[bpaf(short, long)]
        /// Hide output for successful tests
        quiet: bool,

        #[bpaf(positional)]
        /// Arguments for nextest
        args: Vec<OsString>,
    },

    #[bpaf(command)]
    /// Runs benchmarks for `blackbox-log` lib
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

    #[bpaf(command)]
    /// Builds and optimizes the WebAssembly build of blackbox-log-wasm and adds
    /// it to blackbox-log-js
    Wasm,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(command)]
/// Manage & run fuzz targets
enum Fuzz {
    #[bpaf(command)]
    /// List all fuzz targets
    List,

    #[bpaf(command)]
    /// Check fuzz target for errors
    Check {
        #[bpaf(positional("target"))]
        target: Option<String>,
    },

    #[bpaf(command)]
    /// Runs a fuzz target
    Run {
        #[bpaf(external)]
        time: Option<u16>,

        #[bpaf(external)]
        jobs: Option<usize>,

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

fn time() -> impl Parser<Option<u16>> {
    let help = "Passes -max_total_time=<seconds> to libFuzzer, defaulting to 15 minutes if passed \
                without a value";

    let given = bpaf::long("time")
        .help(help)
        .argument::<u16>("seconds")
        .optional();

    let default = bpaf::long("time").flag(Some(900), None).hide();

    bpaf::construct!([given, default])
}

fn jobs() -> impl Parser<Option<usize>> {
    let base = bpaf::short('j').long("jobs");
    let help =
        "Passes --jobs=<count> to cargo fuzz, defaulting to $(nproc) if passed without a value";

    let with_value_or_missing = base
        .clone()
        .help(help)
        .argument::<usize>("count")
        .optional();

    let nproc = std::thread::available_parallelism().unwrap().into();
    let no_value = base.flag(Some(nproc), None).hide();

    bpaf::construct!([with_value_or_missing, no_value])
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

fn get_workspace_args(workspace: bool, fuzz: bool) -> Vec<&'static str> {
    if workspace {
        let mut args = vec!["--workspace"];

        let packages: &[_] = if cfg!(not(target_os = "linux")) {
            &["blackbox-fuzz", "blackbox-sys"]
        } else if !fuzz {
            &["blackbox-fuzz"]
        } else {
            &[]
        };

        for package in packages {
            args.push("--exclude");
            args.push(package);
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
