# `blackbox-log-cli`

[![license](https://img.shields.io/github/license/wetheredge/blackbox)](https://github.com/wetheredge/blackbox/blob/main/COPYING)

This is a cli frontend for `blackbox-log` inspired by the original
`blackbox_decode`.

## Feature comparison

|                          | `blackbox_decode` | `blackbox-log-cli` |
|--------------------------|:-----------------:|:------------------:|
| Log format v1            | ✔️ | ❌ |
| Recent Betaflight logs   | ❌ | ✔️ |
| Raw output               | ✔️ | ❌ |
| Current meter simulation | ✔️ | ❌ |
| IMU simulation           | ✔️ | ❌ |
| Change output units      | ✔️ | ❌ |
| Filter output fields     | ❌ | ✔️ |
| Parallel log parsing     | ❌ | ✔️ |

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

## License

In accordance with the [GNU FAQ][gpl-ports]'s guidance that ports are
derivative works, all code is licensed under the GPLv3 to match the Betaflight
and INAV projects.

[gpl-ports]: https://www.gnu.org/licenses/gpl-faq.html#TranslateCode
