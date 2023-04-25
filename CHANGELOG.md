# Changelog

## [Unreleased]

### Added

- Support INAV version 6.1

## [0.3.1] - 2023-04-25

### Added

- Support for INAV version 6.0
- `Stats.progress`: Estimate of how much of the log data has been parsed so far.

## [0.3.0] - 2023-03-01

### Added

- Parse the headers for debug mode, enabled features, motor protocol, and
  Betaflight's disabled fields

### Changed

- All public fields of `Headers` are now getters.
- Renamed `ParseError::UnknownFirmware` -> `InvalidFirmware` to better match
  its use for any error while parsing the `Firmware revision` header.
- Renamed the `firmware_kind` field of `Headers` to `firmware` and the
  `FirmwareKind` -> `Firmware`, now including the parsed version.
- `ParseError::UnsupportedVersion` is now `UnsupportedDataVersion`
- Unsupported versions of supported firmwares now return the new
  `ParseError::UnsupportedFirmwareVersion` error
- Filters are now applied when creating the `DataParser` using
  `DataParser::with_filters`

### Removed

- EmuFlight support ([`4b1c412`](https://github.com/blackbox-log/blackbox-log/commit/4b1c41298f7ab70b1b2a7efdb0a7b513a746a847))
- `Headers::version` and `LogVersion` representing the log format version
- The unstable `serde` feature

### Fixed

- Missing & misnamed flight modes

## [0.2.0] - 2023-01-28

This version was a major rework of the way the data section is parsed:

- `Log`/`LogView`s got replaced with `DataParser`
  - parsing is now lazy and works roughly like an iterator with a `next` method
    that returns an enum (`ParserEvent`) containing an `Event` or any data
    frame
- filtering is now done on the frame definitions newly exposed by `Headers`
  before creating a `DataParser`

## [0.1.0] - 2022-12-13

Initial release

[unreleased]: https://github.com/blackbox-log/blackbox-log/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/blackbox-log/blackbox-log/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/blackbox-log/blackbox-log/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/blackbox-log/blackbox-log/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/blackbox-log/blackbox-log/releases/tag/v0.1.0
