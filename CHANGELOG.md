# Changelog

## [0.3.3] - 2025-11-28
### Changed
- Updated dependencies and bumped MSRV to 1.82 ([704]).

## [0.3.2] - 2024-9-11
### Added
- Added support for computing source-hashes of Slice files, for tools to utilize ([696]).
### Fixed
- Fixed redefinition errors on containers triggering false-positives for their contents ([700]).
### Changed
- The compiler no longer emits a build summary message when using JSON formatted output ([702]).

## [0.3.1] - 2024-3-27
### Enhancements
- Input files are now loaded and parsed in the order they're passed ([694]).

## [0.3.0] - 2024-2-7
### Added
- Added a new built-in generic type: `Result<S, F>` ([687]).
- Added support for compact enums ([686]).
- Added support for specifying explicit discriminants on enumerators with fields ([688]).
### Enhancements
- Allow `@param` tags to be used for documenting enumerator fields.
- Implemented the `Default` trait for `Ast` and `CompilationState`.
- Implemented the `Hash` trait for `SliceOptions` and `DiagnosticFormat`.
### Fixed
- Improved the cycle detection logic to correctly check fields in enumerators ([689]).
### Breaking
- Enums with fields can no longer be used as dictionary keys ([685]).
- `CompilationState` no longer implements `Send` (so we have greater freedom to evolve it).
### Changed
- The files `code_gen_util.rs` and `code_block.rs` were moved out of this crate (into `slicec-cs`).

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
- Added improved location tracking to messages and tags in doc comments ([670]).
- Improved the `Visitor` to automatically skip unpatched type references ([672]).
- Implemented the `Send` and `Sync` traits for some of the compiler's types.
### Fixed
- Fixed crash caused by compiling a Slice file with no module declaration.
### Breaking
- Interfaces can no longer be used as types in Slice definitions ([675]).
- Removed unused `is_numeric_or_bool` function from `Primitive`.

## [0.1.1] - 2023-10-5
### Added
- Document the crate's MSRV (Minimum Supported Rust Version).
### Changed
- Improved the handling of escape sequences in string literals ([659]).

## [0.1.0] - 2023-9-6
Initial public release!

[704]: https://github.com/icerpc/slicec/pull/704
[702]: https://github.com/icerpc/slicec/pull/702
[700]: https://github.com/icerpc/slicec/pull/700
[696]: https://github.com/icerpc/slicec/pull/696
[694]: https://github.com/icerpc/slicec/pull/694
[689]: https://github.com/icerpc/slicec/pull/689
[688]: https://github.com/icerpc/slicec/pull/688
[687]: https://github.com/icerpc/slicec/pull/687
[686]: https://github.com/icerpc/slicec/pull/686
[685]: https://github.com/icerpc/slicec/pull/685
[678]: https://github.com/icerpc/slicec/pull/678
[677]: https://github.com/icerpc/slicec/pull/677
[675]: https://github.com/icerpc/slicec/pull/675
[672]: https://github.com/icerpc/slicec/pull/672
[670]: https://github.com/icerpc/slicec/pull/670
[668]: https://github.com/icerpc/slicec/pull/668
[664]: https://github.com/icerpc/slicec/pull/664
[662]: https://github.com/icerpc/slicec/pull/662
[659]: https://github.com/icerpc/slicec/pull/659

[0.3.3]: https://github.com/icerpc/slicec/releases/tag/v0.3.3
[0.3.2]: https://github.com/icerpc/slicec/releases/tag/v0.3.2
[0.3.1]: https://github.com/icerpc/slicec/releases/tag/v0.3.1
[0.3.0]: https://github.com/icerpc/slicec/releases/tag/v0.3.0
[0.2.1]: https://github.com/icerpc/slicec/releases/tag/v0.2.1
[0.2.0]: https://github.com/icerpc/slicec/releases/tag/v0.2.0
[0.1.1]: https://github.com/icerpc/slicec/releases/tag/v0.1.1
[0.1.0]: https://github.com/icerpc/slicec/releases/tag/v0.1.0
