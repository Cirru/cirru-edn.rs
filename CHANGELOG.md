# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2025-12-26

### Changed

- **BREAKING**: Upgraded `cirru_parser` from `0.1.35` to `0.2.0`
  - Parser now provides enhanced error reporting with position information
  - Updated error handling to use new `CirruError` type

### Added

- New error handling system with detailed position information:
  - `EdnError` enum with variants for different error types
  - `Position` struct tracking line, column, and byte offset
  - Four error categories: `ParseError`, `StructureError`, `ValueError`, `DeserializationError`
- Position-aware error messages showing line, column, and byte offset when available
- `ERROR_HANDLING.md` documentation with comprehensive error handling guide
- New `error_demo.rs` example demonstrating various error scenarios
- `EdnError::message()` method to extract error messages
- Helper methods for creating errors: `EdnError::parse()`, `EdnError::structure()`, `EdnError::value()`, `EdnError::deserialization()`

### Improved

- Error messages now include context from `cirru_parser` 0.2.0
- Better error categorization for easier debugging
- All parse errors now use structured `EdnError` type instead of plain strings

### Fixed

- Error handling consistency across all parsing functions

## [0.6.21] - Previous versions

See git history for changes in previous versions.
