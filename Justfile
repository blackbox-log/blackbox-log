#!/usr/bin/env -S just --justfile

set dotenv-load

alias help := default
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
	echo '    (r)un           # Run blackbox_decode'
	echo '    rr              # Run release mode blackbox_decode'
	echo '    fmt-all (fa)'
	echo '    check-all (ca)'
	echo '    test-all (ca)'
	echo '    bench-all'
	echo
	echo '# Will always run in <project_root>/fuzz/'
	echo 'Fuzzing:'
	echo '    fuzz-add <target>           # Set up new fuzz target'
	echo '    fuzz-list                   # List all fuzz targets'
	echo '    fuzz-run <target>           # fuzz <target>'
	echo '    fuzz-fmt <target> <input>   # Pretty-print input for target'
	echo '    fuzz-cov <target>           # Generate coverage report for target'
	echo '    fuzz-cmin <target>          # Minify corpus for target'
	echo '    fuzz-tmin <target> <input>  # Minify input test for target'
	echo
	echo '    fls    # alias for fuzz-list'
	echo '    frun   # alias for fuzz-run'
	echo '    fcov   # alias for fuzz-cov'
	echo '    fcmin  # alias for fuzz-cmin'
	echo '    ftmin  # alias for fuzz-tmin'
	echo
	echo 'Misc:'
	echo '    install-dev-deps   # install/update all necessary cargo subcommands'

alias b := build
build *args='':
	cargo build {{args}}

alias r := run
run *args='':
	cargo run --manifest-path {{join(invocation_directory(), 'cli/Cargo.toml')}} -- {{args}}

rr *args='':
	cargo run --release --manifest-path {{join(invocation_directory(), 'cli/Cargo.toml')}} -- {{args}}

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
test +args='': fmt
	cd {{invocation_directory()}} && cargo clippy --tests && cargo nextest run {{args}}

alias ta := test-all
test-all +args='': fmt
	cargo clippy --workspace --lib --tests && cargo nextest run {{args}}

bench *args='': fmt
	cd {{invocation_directory()}} && cargo clippy --benches && cargo criterion --benches {{args}}

bench-all *args='': fmt
	cargo clippy --workspace --lib --benches && cargo criterion --workspace --benches {{args}}

flame-bench bench out filter:
	export CARGO_PROFILE_BENCH_DEBUG=true \
		&& cd {{invocation_directory()}} \
		&& cargo clippy --benches \
		&& cargo flamegraph --deterministic --output {{out}}.svg --bench {{bench}} -- --bench --profile-time 10 '{{filter}}'

@fuzz-add target:
	echo -e "\n[[bin]]\nname = \"{{target}}\"\npath = \"fuzz_targets/{{target}}.rs\"\ntest = false\ndoc = false" >> fuzz/Cargo.toml
	mkdir -p fuzz/fuzz_targets
	echo -e '#![no_main]\n\nuse blackbox_fuzz::{encoding, fuzz_target, UnalignedBytes};\n\nfuzz_target!(|data: UnalignedBytes| {\n    let (mut reference, mut biterator) = data.to_streams().unwrap();\n\n    assert_eq!(todo!(), todo!());\n});' > fuzz/fuzz_targets/{{target}}.rs
	echo 'Initialized fuzz/fuzz_targets/{{target}}.rs'

alias fls := fuzz-list
@fuzz-list:
	echo 'All available fuzzing targets:'
	cargo fuzz list

alias frun := fuzz-run
fuzz-run target *args='':
	cargo +nightly fuzz run {{target}} {{args}}

alias fcmin := fuzz-cmin
fuzz-cmin target *args='':
	cargo +nightly fuzz cmin {{target}} fuzz/corpus/{{target}} {{args}}

alias ftmin := fuzz-tmin
fuzz-tmin target input *args='':
	cargo +nightly fuzz tmin {{target}} fuzz/corpus/{{target}}/{{input}} {{args}}

fuzz-fmt target input *args='':
	cargo +nightly fuzz fmt {{target}} fuzz/corpus/{{target}}/{{input}} {{args}}

nightlySysroot := `rustc +nightly --print sysroot`
llvmCov := join(nightlySysroot, 'lib/rustlib/*/bin/llvm-cov')

alias fcov := fuzz-cov
fuzz-cov target *args='':
	cargo +nightly fuzz coverage {{target}} {{args}}
	@{{llvmCov}} show \
		--format=html \
		--instr-profile=fuzz/coverage/{{target}}/coverage.profdata \
		--output-dir=fuzz/coverage/{{target}} \
		target/*/coverage/*/release/{{target}}
	@echo
	@echo "Saved coverage to fuzz/coverage/{{target}}/index.html"

install-dev-deps:
	cargo install cargo-criterion cargo-fuzz cargo-nextest flamegraph
