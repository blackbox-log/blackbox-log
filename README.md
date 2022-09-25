# Blackbox

[![CI](https://github.com/wetheredge/blackbox/actions/workflows/ci.yaml/badge.svg)](https://github.com/wetheredge/blackbox/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/wetheredge/blackbox/branch/main/graph/badge.svg?token=A89G71SJSD)](https://codecov.io/gh/wetheredge/blackbox)

This is a (very) WIP port of the [Betaflight blackbox tools][tools] as a Rust
crate. Its raison d’être is to allow applications to read blackbox files without
needing to bundle a copy of `blackbox_decode` as a separate binary and without
needing to parse CSV. It also aims to be significantly faster than the original.

## Roadmap

- [ ] Document the format
- [ ] Drop-in replacement for `blackbox_decode` CLI
- [ ] Support logs from INAV, EmuFlight, etc
- [ ] JavaScript interface using WebAssembly
- [ ] Web-based [log viewer][viewer]?

[tools]: https://github.com/betaflight/blackbox-tools
[viewer]: https://github.com/betaflight/blackbox-log-viewer
