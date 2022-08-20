#!/usr/bin/env -S just --justfile

set dotenv-load

@default:
	# @just --list --justfile {{justfile()}} --unsorted
	echo 'Usage:'
	echo '    just <command> [options]'
	echo
	echo 'Package commands:'
	echo '    (b)uild'
	echo '    (f)mt'
	echo '    (c)heck   # run clippy'
	echo '    (t)est    # run tests with nextest'
	echo '    bench     # run criterion benchmarks'
	echo
	echo '    flame-bench <bench> <out> <filter>'
	echo '              # save a flamegraph at <out>.svg of <bench> run for 10 seconds'
	echo
	echo 'Workspace commands:'
	echo '    fmt-all (fa)'
	echo '    check-all (ca)'
	echo '    test-all (ca)'
	echo '    bench-all'
	echo
	echo '# Will always run in <project_root>/fuzzing/'
	echo 'Fuzzing:'
	echo '    fuzz <command>           # cargo hfuzz <command>'
	echo '    fuzz-run (fr) <target>   # fuzz <target>'
	echo '    fuzz-debug <target>      # debug crashes for <target>'
	echo '    defuzz                   # alias for fuzz-debug'
	echo
	echo 'Misc:'
	echo '    install-dev-deps   # install/update all necessary cargo subcommands'

alias b := build
build *args='':
	cargo build {{args}}

alias f := fmt
alias format := fmt
fmt *args='':
	cd {{invocation_directory()}} && cargo fmt {{args}}

alias fa := fmt-all
alias format-all := fmt-all
fmt-all:
	cargo fmt

alias c := clippy
alias check := clippy
clippy *args='': fmt
	cd {{invocation_directory()}} && cargo clippy {{args}}

alias ca := clippy-all
alias check-all := clippy-all
clippy-all *args='': fmt-all
	cargo clippy --workspace --all-targets {{args}}

alias t := test
test +args='run': fmt
	cd {{invocation_directory()}} && cargo clippy --tests && cargo nextest {{args}}

alias ta := test-all
test-all +args='run': fmt
	cargo clippy --workspace --lib --tests && cargo nextest {{args}}

bench *args='': fmt
	cd {{invocation_directory()}} && cargo clippy --benches && cargo criterion --benches {{args}}

bench-all *args='': fmt
	cargo clippy --workspace --lib --benches && cargo criterion --workspace --benches {{args}}

flame-bench bench out filter:
	export CARGO_PROFILE_BENCH_DEBUG=true \
		&& cd {{invocation_directory()}} \
		&& cargo clippy --benches \
		&& cargo flamegraph --deterministic --output {{out}}.svg --bench {{bench}} -- --bench --profile-time 10 '{{filter}}'

export HFUZZ_BUILD_ARGS := '--profile=fuzz'
export HFUZZ_DEBUGGER := 'rust-gdb'
fuzz +args='-h':
	@echo "HFUZZ_BUILD_ARGS='$HFUZZ_BUILD_ARGS'"
	cd fuzzing/ && cargo hfuzz {{args}}

alias frun := fuzz-run
fuzz-run target *args='': fmt
	@echo "HFUZZ_BUILD_ARGS='$HFUZZ_BUILD_ARGS'"
	cd fuzzing/ && cargo clippy --bins && cargo hfuzz run {{target}} {{args}}

alias defuzz := fuzz-debug
fuzz-debug target *args='':
	#!/usr/bin/env bash
	echo "HFUZZ_BUILD_ARGS='$HFUZZ_BUILD_ARGS'"
	echo "HFUZZ_DEBUGGER='$HFUZZ_DEBUGGER'"
	args={{args}}
	default="hfuzz_workspace/{{target}}/*.fuzz"
	set -euxo pipefail
	cd fuzzing/ && cargo hfuzz run-debug {{target}} ${args:-$default}

install-dev-deps:
	cargo install cargo-criterion cargo-nextest flamegraph honggfuzz
