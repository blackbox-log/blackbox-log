# Blackbox

[![CI](https://github.com/wetheredge/blackbox/actions/workflows/ci.yaml/badge.svg)](https://github.com/wetheredge/blackbox/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/wetheredge/blackbox/branch/main/graph/badge.svg?token=A89G71SJSD)](https://codecov.io/gh/wetheredge/blackbox)
[![dependency status](https://deps.rs/repo/github/wetheredge/blackbox/status.svg)](https://deps.rs/repo/github/wetheredge/blackbox)
[![MSRV](https://img.shields.io/static/v1?logo=rust&label=MSRV&color=dea584&message=1.64)](https://github.com/rust-lang/rust/blob/master/RELEASES.md)
[![license](https://img.shields.io/github/license/wetheredge/blackbox)](https://github.com/wetheredge/blackbox/blob/main/COPYING)

This is a (very) WIP port of the [Betaflight blackbox tools][bf-tools] as a Rust
crate. Its raison d’être is to allow applications to read blackbox files without
needing to bundle a copy of `blackbox_decode` as a separate binary and without
needing to parse CSV. It also aims to be significantly faster than the original.

## Roadmap

- [x] Document the format: See [INAV's documentation][inav-docs]
- [ ] Drop-in replacement for `blackbox_decode` CLI
- [ ] Support logs from INAV, EmuFlight, etc
- [ ] JavaScript interface using WebAssembly
- [ ] Web-based [log viewer][bf-viewer]?

## License

The parser is heavily based on the original implementations in
[Betaflight's][bf-tools] and [INAV's][inav-tools] repositories along with the
[INAV documentation][inav-docs], so in accordance with the
[GNU FAQ on ports](https://www.gnu.org/licenses/gpl-faq.html#TranslateCode),
this is licensed under GPLv3 to match.

[bf-tools]: https://github.com/betaflight/blackbox-tools
[bf-viewer]: https://github.com/betaflight/blackbox-log-viewer
[inav-tools]: https://github.com/iNavFlight/blackbox-tools
[inav-docs]: https://github.com/iNavFlight/inav/blob/master/docs/development/Blackbox%20Internals.md
