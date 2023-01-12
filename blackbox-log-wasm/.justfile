set fallback := true
set dotenv-load := true

_default:
    @echo blackbox-log-wasm:
    @just --list --unsorted --list-heading ''
    @echo
    @echo Global:
    @cd .. && just --list --unsorted --list-heading ''

targetDir := '../target/wasm32-unknown-unknown/release'
wasmFile := targetDir / 'blackbox-log.wasm'

# Build
build:
    cargo build --release
    cp {{ targetDir / 'blackbox_log_wasm.wasm' }} {{ wasmFile }}

# Apply multi-value transform
multivalue:
    multi-value-reverse-polyfill {{ wasmFile }} \
        'headers_firmwareRevision i32 i32' 'headers_boardInfo i32 i32' 'headers_craftName i32 i32'

    mv {{ targetDir / 'blackbox-log.multivalue.wasm' }} {{ wasmFile }}

# Run wasm-opt
opt:
    wasm-opt -O3 {{ wasmFile }} -o {{ wasmFile }} \
        --enable-bulk-memory --enable-multivalue --enable-sign-ext

# Full build & optimize, then copy into blackbox-log-js
wasm: build multivalue opt
    cp {{ wasmFile }} ../blackbox-log-js/src/blackbox-log.wasm

# Show disassembly
dis:
    wasm-dis {{ wasmFile }} | less
