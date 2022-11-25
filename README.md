# `blackbox-log`

[![CI](https://github.com/wetheredge/blackbox/actions/workflows/ci.yaml/badge.svg)](https://github.com/wetheredge/blackbox/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/wetheredge/blackbox/branch/main/graph/badge.svg)](https://codecov.io/gh/wetheredge/blackbox)
[![dependency status](https://deps.rs/repo/github/wetheredge/blackbox/status.svg)](https://deps.rs/repo/github/wetheredge/blackbox)
[![MSRV](https://img.shields.io/static/v1?logo=rust&label=MSRV&color=dea584&message=1.65)](https://github.com/rust-lang/rust/blob/master/RELEASES.md)
[![license](https://img.shields.io/github/license/wetheredge/blackbox)](https://github.com/wetheredge/blackbox/blob/main/COPYING)

This is a WIP Rust port of the Betaflight & INAV blackbox tools. It provides a
Rust library crate and a ([mostly][comparison]) drop-in replacement
for `blackbox_decode`, with a WASM/JavaScript interface Coming Soon™.

## Roadmap

- [ ] Betaflight log parsing
  - [x] Spec-compliant error recovery
  - [ ] Handle outputting GPS data
    - [x] Merged CSV
    - [x] Separate CSV
    - [ ] Separate GPX
- [ ] `blackbox_decode` replacement
- [x] Snapshot testing with [`insta`](https://insta.rs)
  - [x] [`fc-blackbox` test files](https://github.com/ilya-epifanov/fc-blackbox/tree/main/src/test-data)
- [ ] INAV support
- [ ] EmuFlight support
- [ ] JavaScript interface using WebAssembly
- [ ] Web-based [log viewer][bf-viewer]?
- [ ] [QUICKSILVER](https://github.com/BossHobby/QUICKSILVER) support??

## `blackbox_decode` feature comparison

|                            | [wetheredge/blackbox][repo] | [betaflight/blackbox-tools][bf-tools] |
|----------------------------|:---------------------------:|:-------------------------------------:|
| Log format v1              |              :x:            |           :heavy_check_mark:          |
| Recent Betaflight versions |      :heavy_check_mark:     |                   :x:                 |
| Raw log output             |              :x:            |           :heavy_check_mark:          |
| Current meter simulation   |              :x:            |           :heavy_check_mark:          |
| IMU simulation             |              :x:            |           :heavy_check_mark:          |
| Output unit custimization  |              :x:            |           :heavy_check_mark:          |
| Output field filter        |      :heavy_check_mark:     |                   :x:                 |
| Parallel log parsing       |      :heavy_check_mark:     |                   :x:                 |

## Benchmarks

As of [f5163d9](https://github.com/wetheredge/blackbox/tree/f5163d92a3574e5e251acd5b5da3d0c0d84c23cf):

```shell
$ exa -lbs size --no-time --no-permissions --no-user blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL
6.6Mi blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL

$ hyperfine -w 10 -L version f5163d9,betaflight './blackbox_decode-{version} blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL'
Benchmark #1: ./blackbox_decode-f5163d9 blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL
  Time (mean ± σ):     661.9 ms ±   8.0 ms    [User: 604.2 ms, System: 59.0 ms]
  Range (min … max):   646.9 ms … 673.0 ms    10 runs

Benchmark #2: ./blackbox_decode-betaflight blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL
  Time (mean ± σ):      1.068 s ±  0.019 s    [User: 1.018 s, System: 0.046 s]
  Range (min … max):    1.042 s …  1.102 s    10 runs

Summary
  './blackbox_decode-f5163d9 blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL' ran
    1.61 ± 0.03 times faster than './blackbox_decode-betaflight blackbox-log/tests/logs/gimbal-ghost/LOG00001.BFL'
```

`…/gimbal-ghost/LOG00001.BFL` contains only one log. Files with multiple logs
will see even larger improvements since logs are decoded in parallel using
[`rayon`](https://lib.rs/crates/rayon).

> **Note**: Adding GPS support and fixing the remaining bugs may impact
performance. Benchmarks will be updated before 1.0.

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

[repo]: https://github.com/wetheredge/blackbox
[betaflight]: https://github.com/betaflight/betaflight
[inav]: https://github.com/iNavFlight/inav
[emuflight]: https://github.com/emuflight/EmuFlight
[comparison]: #blackbox_decode-feature-comparison
[bf-tools]: https://github.com/betaflight/blackbox-tools
[bf-viewer]: https://github.com/betaflight/blackbox-log-viewer
[inav-tools]: https://github.com/iNavFlight/blackbox-tools
[inav-docs]: https://github.com/iNavFlight/inav/blob/master/docs/development/Blackbox%20Internals.md
[gpl-ports]: https://www.gnu.org/licenses/gpl-faq.html#TranslateCode
