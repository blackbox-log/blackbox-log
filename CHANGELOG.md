# Changelog

## [Unreleased]

### Breaking

- Renamed `ParseError::UnknownFirmware` -> `ParseError::InvalidFirmware` to
  better match its use for any error while parsing the `Firmware revision`
  header.
- Renamed the `firmware_kind` field of `Headers` to `firmware` and the
  `FirmwareKind` -> `Firmware`, now including the parsed version.
- Fixed missing & misnamed flight modes
  - Renamed some variants of `FlightMode`

### Changed

- All fields of `Headers` are now methods.

### Removed

- `Headers::version` and `LogVersion`

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

[unreleased]: https://github.com/blackbox-log/blackbox-log/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/blackbox-log/blackbox-log/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/blackbox-log/blackbox-log/releases/tag/v0.1.0
