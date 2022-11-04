# `blackbox-log`

[![CI](https://github.com/wetheredge/blackbox/actions/workflows/ci.yaml/badge.svg)](https://github.com/wetheredge/blackbox/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/wetheredge/blackbox/branch/main/graph/badge.svg)](https://codecov.io/gh/wetheredge/blackbox)
[![dependency status](https://deps.rs/repo/github/wetheredge/blackbox/status.svg)](https://deps.rs/repo/github/wetheredge/blackbox)
[![MSRV](https://img.shields.io/static/v1?logo=rust&label=MSRV&color=dea584&message=1.65)](https://github.com/rust-lang/rust/blob/master/RELEASES.md)
[![license](https://img.shields.io/github/license/wetheredge/blackbox)](https://github.com/wetheredge/blackbox/blob/main/COPYING)

This is a WIP Rust port of the Betaflight & INAV blackbox tools. It provides a
Rust library crate and a (mostly, [see below][comparison]) drop-in replacement
for `blackbox_decode`, with a WASM/JavaScript interface Coming Soon™.

## Roadmap

- [ ] Betaflight log parsing
  - [x] Spec-compliant error recovery
  - [ ] Handle outputting GPS data
- [ ] `blackbox_decode` replacement
- [x] Snapshot testing with [`insta`](https://insta.rs)
  - [x] [`fc-blackbox` test files](https://github.com/ilya-epifanov/fc-blackbox/tree/main/src/test-data)
- [ ] INAV support
- [ ] EmuFlight support
- [ ] JavaScript interface using WebAssembly
- [ ] Web-based [log viewer][bf-viewer]?
- [ ] [QUICKSILVER](https://github.com/BossHobby/QUICKSILVER) support??

## `blackbox_decode` feature comparison

|                            |   `blackbox-log`   |     Betaflight     |
|----------------------------|:------------------:|:------------------:|
| Log format v1              |         :x:        | :heavy_check_mark: |
| Recent Betaflight versions | :heavy_check_mark: |         :x:        |
| Raw log output             |         :x:        | :heavy_check_mark: |
| Current meter simulation   |         :x:        | :heavy_check_mark: |
| IMU simulation             |         :x:        | :heavy_check_mark: |
| Output field filter        | :heavy_check_mark: |         :x:        |
| Parallel log parsing       | :heavy_check_mark: |         :x:        |

## Benchmarks

Needs more formal benchmarks, but initial tests show it to be significantly
faster than `blackbox_decode`, especially as the number of logs/files
increases.

## Prior art

- `blackbox_decode` ([Betaflight's][bf-tools] and [INAV's][inav-tools])
- [Betaflight][betaflight] and [INAV][inav] source code; mainly
  `src/main/blackbox/*`
- The INAV blackbox [documentation][inav-docs]
- [`fc-blackbox`](https://lib.rs/crates/fc-blackbox)

## License

In accordance with the [GNU FAQ][gpl-ports]'s guidance that ports are
derivative works, all code is licensed under the GPLv3 to match the Betaflight
and INAV projects.

[betaflight]: https://github.com/betaflight/betaflight
[inav]: https://github.com/iNavFlight/inav
[emuflight]: https://github.com/emuflight/EmuFlight
[comparison]: #blackbox_decode-feature-comparison
[bf-tools]: https://github.com/betaflight/blackbox-tools
[bf-viewer]: https://github.com/betaflight/blackbox-log-viewer
[inav-tools]: https://github.com/iNavFlight/blackbox-tools
[inav-docs]: https://github.com/iNavFlight/inav/blob/master/docs/development/Blackbox%20Internals.md
[gpl-ports]: https://www.gnu.org/licenses/gpl-faq.html#TranslateCode
