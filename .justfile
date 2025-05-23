_default:
    @just --list --unsorted

# Format all crates
fmt *args='':
    cargo +nightly fmt {{ args }}

# Run clippy
[no-cd]
check *args='--all-features':
    cargo clippy --all-targets {{ args }}

# Run clippy for a no_std target
check-no_std:
    cargo clippy --no-default-features --target thumbv7em-none-eabihf

# Run tests
[no-cd]
test *args='':
    cargo nextest run --all-features --status-level=leak {{ args }}

# Run all tests
[no-cd]
test-all:
    cargo nextest run --all-features --status-level=leak --run-ignored=all

# Generate a full test coverage report using cargo-llvm-cov
[no-cd]
cov *args='':
    cargo llvm-cov --all-features --html nextest --status-level=leak --run-ignored=all --ignore-filename-regex generated {{ args }}

# Run benchmarks with cargo-criterion
[no-cd]
bench *args='--benches':
    cargo criterion {{ args }} 

# Test that all benchmarks run successfully
[no-cd]
bench-test:
    cargo criterion --benches -- --test

# Profile a benchmark using cargo-flamegraph
[no-cd]
profile-bench file bench time='10' *args='':
    cargo flamegraph --deterministic --palette rust --bench {{ file }} -- --bench --profile-time {{ time }} {{ bench }} {{ args }}

# Run codegen
codegen:
    cargo run -p codegen

devpod-up:
    devpod up . --devcontainer-path .devcontainer/default-prebuilt/devcontainer.json

devpod-delete:
    devpod delete .

devpod-ssh:
    devpod ssh --command 'cd /workspaces/blackbox-log && zsh' .
