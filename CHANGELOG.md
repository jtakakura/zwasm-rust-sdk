# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1] - 2026-08-13
### Changed
- Moved the repository to the zwasm organization and updated the repository and upstream URLs
- Updated the bundled zwasm C API to 1.11.1, the final release of the v1 line. The C header is unchanged, so the generated bindings are identical
- Pinned all GitHub Actions to full-length commit SHAs and added a Dependabot config

### Fixed
- docs.rs builds. The build script now skips the Zig build when `DOCS_RS` is set and generates the bindings from the bundled header
- Unresolved intra-doc links to `Config`, `Module`, `WasiConfig`, and `Imports`
- Removed Windows from the supported platform list in the crate docs. Only Linux and macOS are supported

## [0.1.0] - 2026-04-26
### Added
- Initial release of zwasm-sdk core API
- Safe Rust bindings for zwasm C API via zwasm-sys
- Unit tests (normal, error, edge cases)
- Integration tests and E2E tests using examples
- Practical examples: run_wasm, host_imports, memory_io, wasi_config
- CI with cargo fmt, clippy, test (Linux/macOS)
