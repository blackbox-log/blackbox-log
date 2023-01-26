# `blackbox-log`

[![CI](https://github.com/wetheredge/blackbox/actions/workflows/ci.yaml/badge.svg)](https://github.com/wetheredge/blackbox/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/wetheredge/blackbox/branch/main/graph/badge.svg)](https://codecov.io/gh/wetheredge/blackbox)
[![dependency status](https://deps.rs/repo/github/wetheredge/blackbox/status.svg)](https://deps.rs/repo/github/wetheredge/blackbox)
[![crates.io](https://img.shields.io/crates/v/blackbox-log)](https://crates.io/blackbox-log)
[![docs.rs](https://img.shields.io/docsrs/blackbox-log)](https://docs.rs/blackbox-log)
[![MSRV](https://img.shields.io/static/v1?logo=rust&label=MSRV&color=dea584&message=1.66)](https://github.com/rust-lang/rust/blob/master/RELEASES.md)
[![license](https://img.shields.io/github/license/wetheredge/blackbox)](https://github.com/wetheredge/blackbox/blob/main/COPYING)


This is a Rust port of Betaflight's & INAV's blackbox tools. Check the [GitHub
organization][org] for related projects. Or, read the [docs] to get started.

> **Note:** `blackbox-log` is not quite ready for production use yet --
consider it early-mid beta quality.

## Why?

There are two official parser implementations, each with a copy maintained by
Betaflight and one by INAV, so why another?

Neither is all that great for building other software with:
- `blackbox_decode` ([BF][bf-tools], [INAV][inav-tools]) has missed some of
  the changes in the format in the last few years, so its output is no longer
  entirely correct. Additionally, it decodes and writes *everything* to disk,
  so you pay for data your application may not need.
- The log viewer's parser ([BF][bf-viewer], [INAV][inav-viewer]) isn't meant to
  be used by anything else and is tightly coupled with its GUI. It's written in
  JavaScript, which limits the places it can reasonably be embedded.

This project aims to fill that niche. An ergonomic, up-to-date API usable
anywhere that supports Rust or WebAssembly.

## Contributing

At the moment, `blackbox-log` is still in heavy development and probably isn't
quite ready for code contributions. However, [bug reports][bugs] are welcomed.
If at all possible, please include a log file exhibiting the bug. If you've got
any other questions or ideas, feel free to start a [discussion][discussions].

## References/Prior Art

- `blackbox_decode` ([Betaflight][bf-tools], [INAV][inav-tools])
- Blackbox log viewer ([Betaflight][bf-viewer], [INAV][inav-viewer])
- [Betaflight][betaflight] and [INAV][inav] source code; mainly in `src/main/blackbox/`
- The INAV blackbox [documentation](https://github.com/iNavFlight/inav/blob/master/docs/development/Blackbox%20Internals.md)
- [`fc-blackbox`](https://lib.rs/crates/fc-blackbox)

## License

In accordance with the [GNU FAQ][gpl-ports]'s guidance that ports are
derivative works, all code is licensed under the GPLv3 to match the Betaflight
and INAV projects.

[org]: https://github.com/blackbox-log/
[docs]: https://docs.rs/blackbox-log
[bf-tools]: https://github.com/betaflight/blackbox-tools
[bf-viewer]: https://github.com/betaflight/blackbox-log-viewer
[inav-tools]: https://github.com/iNavFlight/blackbox-tools
[inav-viewer]: https://github.com/iNavFlight/blackbox-log-viewer
[betaflight]: https://github.com/betaflight/betaflight
[inav]: https://github.com/iNavFlight/inav
[bugs]: https://github.com/blackbox-log/blackbox-log/issues
[discussions]: https://github.com/blackbox-log/blackbox-log/discussions
[gpl-ports]: https://www.gnu.org/licenses/gpl-faq.html#TranslateCode
