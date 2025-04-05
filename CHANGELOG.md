# Changelog

## [Unreleased]

## [0.4.3] - 2024.04.13

### Added

- Support INAV version 8.0

### Changed

- Bumped MSRV to 1.81

## [0.4.2] - 2024-04-28

### Added

- Support Betaflight version 4.5.x

### Changed

- Support all minor versions of supported INAV major versions

## [0.4.1] - 2023-12-06

### Added

- Support INAV version 7.0

## [0.4.0] - 2023-10-29

### Changed

- Relicensed as MIT OR APACHE-2.0 to match Rust convention
- **BREAKING**: Library users no longer need to keep track of the
  internal `Reader` instance used in parsing. The `File::get_reader`
  method has been replaced with `File::parse` that returns the parsed
  `Headers` directly. A `DataParser` is now created using the `Headers::
  {data_parser,data_parser_with_filters}` methods.
- **BREAKING**: The filter API has been redone to improve clarity.
  `FieldFilterSet` is now `FilterSet` and there is a new `Filter` enum to
  represent the filter for each frame kind rather than using an `Option`.

## [0.3.2] - 2023-07-04

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

[unreleased]: https://github.com/blackbox-log/blackbox-log/compare/v0.4.3...HEAD
[0.4.3]: https://github.com/blackbox-log/blackbox-log/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/blackbox-log/blackbox-log/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/blackbox-log/blackbox-log/compare/v0.4.0...v0.4.1
[0.3.2]: https://github.com/blackbox-log/blackbox-log/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/blackbox-log/blackbox-log/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/blackbox-log/blackbox-log/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/blackbox-log/blackbox-log/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/blackbox-log/blackbox-log/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/blackbox-log/blackbox-log/releases/tag/v0.1.0
