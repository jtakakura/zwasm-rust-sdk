# zwasm-sys

zwasm-sys provides low-level Rust bindings to the zwasm C API.  
This crate offers FFI access to the zwasm WebAssembly runtime for use in higher-level Rust libraries and applications.


## Supported Rust Version

A recent stable Rust compiler is recommended. (Rust 2021 edition)

## Build and Link Requirements

- Requires [Zig](https://ziglang.org/) to be installed and available in your PATH (used to build the zwasm C library).
- The zwasm C API is built automatically as a submodule during the build process; no manual installation is needed.
- Supported platforms: **Unix-like systems (Linux, macOS)** only. Windows is not supported.

If you encounter build issues, please ensure Zig is installed and your environment supports building C libraries with Zig.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
zwasm-sys = "0.1"
```

## Version Compatibility

| zwasm-sys | zwasm C API |
|-----------|-------------|
| 0.1.0     | 1.11.0      |

## ⚠️ Safety and Usage Notes

This crate provides raw FFI bindings to the zwasm C API, generated automatically via `bindgen`.
**All functions and types exposed by zwasm-sys are inherently unsafe.**

- You must use `unsafe` blocks to call these functions.
- No safety guarantees are provided by this crate. It is your responsibility to ensure:
	- All pointers passed to FFI functions are valid and properly aligned.
	- Lifetimes and memory management follow the C API's requirements.
	- You handle error codes and null pointers as documented in the C API.
- Incorrect usage may lead to undefined behavior, memory corruption, or security vulnerabilities.

**Recommendation:**
Use this crate as a building block for higher-level, safe Rust libraries.
If you need a safe and ergonomic API, consider using or creating a wrapper crate that encapsulates unsafe usage.

### Example (unsafe FFI usage)

```rust
use zwasm_sys::*;
unsafe {
		let module = zwasm_module_new(ptr, len);
		// ... check for null, handle errors, etc.
}
```

## API Reference

- [zwasm C API Documentation](https://clojurewasm.github.io/zwasm/en/c-api.html)

## License

- **Rust code in this repository (including zwasm-sys):** MIT License
- **zwasm C API (submodule):** See [zwasm LICENSE](https://github.com/clojurewasm/zwasm/blob/main/LICENSE) for details. You must comply with the license of the zwasm C library in addition to the MIT License for Rust code.

## Contributing & Issue Reporting

Contributions, bug reports, and feature requests are welcome!

- Please open an issue if you find a bug or have a question.
- Pull requests are encouraged for improvements, documentation, or new tests.
- For major changes, please open an issue first to discuss what you would like to change.

See [CONTRIBUTING.md](../CONTRIBUTING.md) for details.
