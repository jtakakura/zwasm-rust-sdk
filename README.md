# zwasm Rust SDK

zwasm-sdk provides safe, idiomatic Rust bindings to the [zwasm](https://github.com/clojurewasm/zwasm) WebAssembly runtime.

## Supported Rust Version

A recent stable Rust compiler is recommended. (Rust 2021 edition)

## Build and Platform Requirements

- Requires [Zig](https://ziglang.org/) 0.16.0+ in your PATH (used to build the zwasm C library)
- Supported platforms: **Linux (x86_64, aarch64), macOS (aarch64)**

## Version Compatibility

| zwasm-sdk | zwasm-sys | zwasm C API |
|-----------|-----------|-------------|
| 0.1.x     | 0.1.x     | 1.10.x      |

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
zwasm-sdk = "0.1"
```

## Example (safe API)

```rust
use zwasm_sdk::{Config, Module};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_bytes = std::fs::read("your_module.wasm")?;
    let mut config = Config::new()?;
    config.set_timeout(5_000);
    let module = Module::new_configured(&wasm_bytes, &config)?;
    let results = module.invoke("main", &[])?;
    println!("results = {results:?}");
    Ok(())
}
```

## Examples

Practical runnable examples are available in [examples/](examples/):

- [run_wasm.rs](examples/run_wasm.rs): Load a module, list exports, and invoke a function.
- [host_imports.rs](examples/host_imports.rs): Register host functions and call them from Wasm.
- [memory_io.rs](examples/memory_io.rs): Read/write module linear memory.
- [wasi_config.rs](examples/wasi_config.rs): Configure argv/env/preopen for WASI.

Run examples with:

```bash
cargo run --example run_wasm
cargo run --example host_imports
cargo run --example memory_io
cargo run --example wasi_config
```

## Safety and Usage Notes

zwasm-sdk provides a safe, ergonomic API for most use cases. All FFI unsafety is encapsulated. For advanced or low-level usage, see zwasm-sys.

## API Reference

- [API documentation on docs.rs](https://docs.rs/zwasm-sdk)
- [zwasm C API documentation](https://clojurewasm.github.io/zwasm/en/c-api.html)

## License

MIT License. See [LICENSE](LICENSE) for details.

## Contributing

Contributions, bug reports, and feature requests are welcome!
See [CONTRIBUTING.md](CONTRIBUTING.md) for details.
