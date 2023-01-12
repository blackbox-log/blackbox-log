set fallback := true
set dotenv-load := true

_default:
    @echo blackbox-log-wasm:
    @just --list --unsorted --list-heading ''
    @echo
    @echo Global:
    @cd .. && just --list --unsorted --list-heading ''

# Build .wasm
build:
    cargo +nightly build --target wasm32-unknown-unknown --release

# Build, optimize, and copy .wasm into blackbox-log-js
wasm: build
    wasm-opt -O3 ../target/wasm32-unknown-unknown/release/blackbox_log_wasm.wasm \
        -o ../blackbox-log-js/src/blackbox-log.wasm \
        --enable-bulk-memory --enable-multivalue --enable-sign-ext
