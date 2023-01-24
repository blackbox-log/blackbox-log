_default:
    @just --list --unsorted

# Format all crates
fmt *args='':
    cargo +nightly fmt {{ args }}

# Run clippy using cargo-cranky
[no-cd]
check *args='':
    cargo cranky --all-features --all-targets {{ args }}

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
    cargo llvm-cov --all-features --html nextest --status-level=leak --run-ignored=all {{ args }}

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

# Install/update all dev tools from crates.io
install:
    cargo install --locked \
        cargo-cranky \
        cargo-criterion \
        cargo-llvm-cov \
        cargo-nextest \
        flamegraph \
        wasm-opt

    cargo install --locked --git https://github.com/wetheredge/wasm-multi-value-reverse-polyfill
