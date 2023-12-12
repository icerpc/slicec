# Changelog

## [Future] - Sometime
### Added
- Added support for compact enums ([686]).
### Breaking
- Only simple enums can be used as dictionary key types ([685]).

## [0.2.1] - 2023-11-29
### Enhancements
- Added default no-op implementations to `Visitor` to make it easier to implement ([678]).
### Fixed
- Fixed crash caused by some syntax errors when the parser expected EOL ([677]).

## [0.2.0] - 2023-11-28
### Added
- Added support for enums with associated fields ([664]).
- Added support for specifying scoped exceptions in `@throws` doc comment tags ([662]).
- Added `is_within` to check if a `Location` is within a `Span` ([668]).
### Enhancements
- Added improve location tracking to messages and tags in doc comments ([670]).
- Improved the `Visitor` to automatically skip unpatched type references ([672]).
- Implemented the `Send` and `Sync` trait for some of the compiler's types.
### Fixed
- Fixed crash caused by compiling a Slice file with no module declaration.
### Breaking
- Interfaces can no longer be used a type in Slice definitions ([675]).
- Removed unused `is_numeric_or_bool` function from `Primitive`.

## [0.1.1] - 2023-10-5
### Added
- Document the crate's MSRV (Minimum Supported Rust Version).
### Changed
- Improved the handling of escape sequences in string literals ([659]).

## [0.1.0] - 2023-9-6
Initial public release!

[678]: https://github.com/icerpc/slicec/pull/678
[677]: https://github.com/icerpc/slicec/pull/677
[675]: https://github.com/icerpc/slicec/pull/675
[672]: https://github.com/icerpc/slicec/pull/672
[670]: https://github.com/icerpc/slicec/pull/670
[668]: https://github.com/icerpc/slicec/pull/668
[664]: https://github.com/icerpc/slicec/pull/664
[662]: https://github.com/icerpc/slicec/pull/662
[659]: https://github.com/icerpc/slicec/pull/659

[0.2.1]: https://github.com/icerpc/slicec/releases/tag/v0.2.1
[0.2.0]: https://github.com/icerpc/slicec/releases/tag/v0.2.0
[0.1.1]: https://github.com/icerpc/slicec/releases/tag/v0.1.1
[0.1.0]: https://github.com/icerpc/slicec/releases/tag/v0.1.0
