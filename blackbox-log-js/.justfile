set fallback := true

_default:
    @echo blackbox-log-js:
    @just --list --unsorted --list-heading ''
    @echo
    @echo Global:
    @cd .. && just --list --unsorted --list-heading ''

# Run prettier
fmt:
    pnpm prettier --write .

# Check formatting, lints, and types
check:
    pnpm prettier --check .
    pnpm eslint --cache src
    pnpm tsc

# Run the dev server
dev *args='':
    pnpm vite {{ args }}

# Check types and build for production
build:
    pnpm tsc
    pnpm vite build

# Remove build artifacts
clean:
    rm -f src/blackbox-log.wasm
    rm -rf dist

# Regenerate .wasm
wasm:
    @just ../blackbox-log-wasm/wasm
