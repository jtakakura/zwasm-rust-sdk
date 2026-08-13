//! # zwasm-sdk
//!
//! Rust bindings for [zwasm](https://github.com/zwasm/zwasm): a small, fast, and spec-complete WebAssembly runtime written in Zig.
//!
//! ## Features
//! - **Tiny and fast**: ~1.2MB binary, JIT for ARM64/x86_64, full SIMD, threads, GC, exception handling, and more.
//! - **100% spec conformance**: Passes all official Wasm spec tests and proposals through 3.0.
//! - **Component Model**: WIT parser, Canonical ABI, WASI Preview 1+2, component linking.
//! - **Security**: Deny-by-default WASI, capability flags, resource limits.
//! - **Zero dependencies**: Pure Zig core, no libc required.
//! - **Interruption support**: cancel a running invocation from another thread with [`CancelHandle`].
//!
//! ## Supported platforms
//! - Linux (x86_64, aarch64)
//! - macOS (aarch64)
//!
//! ## Example
//! ```no_run
//! use zwasm_sdk::{Module};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let wasm_bytes: &[u8] = &[
//!         0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01,
//!         0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x05, 0x01, 0x01, 0x66, 0x00, 0x00, 0x0a, 0x06,
//!         0x01, 0x04, 0x00, 0x41, 0x2a, 0x0b,
//!     ];
//!     let module = Module::new(wasm_bytes)?;
//!     let results = module.invoke("f", &[])?;
//!     assert_eq!(results[0], 42);
//!     Ok(())
//! }
//! ```
//!
//! For long-running Wasm, enable cancellation in [`Config`] and use [`Module::cancel_handle`]
//! to interrupt an in-progress invocation from another thread. Interrupted calls can be
//! recognized with [`ZwasmError::is_interrupted`].
//!
//! ## Design
//! zwasm uses a 4-tier execution pipeline:
//! - Bytecode → Predecoded IR → Register IR → Native JIT (ARM64/x86_64)
//! - All Wasm 3.0 proposals, threads, SIMD, GC, exception handling supported
//! - Allocator-parameterized: caller controls memory allocation
//!
//! ## Examples
//! See practical examples in the repository:
//! - `examples/run_wasm.rs`
//! - `examples/host_imports.rs`
//! - `examples/memory_io.rs`
//! - `examples/wasi_config.rs`
//!
//! See the upstream [README](https://github.com/zwasm/zwasm) and [ARCHITECTURE.md](https://github.com/zwasm/zwasm/blob/v1.11.0/ARCHITECTURE.md) for details.
mod config;
mod error;
mod ffi;
mod imports;
mod module;
#[cfg(test)]
mod test_fixtures;
mod utils;
mod wasi;

pub use config::Config;
pub use error::ZwasmError;
pub use imports::Imports;
pub use module::{CancelHandle, Module};
pub use wasi::WasiConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_modules() {
        let m1 = Module::new(test_fixtures::RETURN42_WASM).expect("Failed to create module 1");
        let m2 = Module::new(test_fixtures::ADD_WASM).expect("Failed to create module 2");
        let m3 = Module::new(test_fixtures::MEMORY_WASM).expect("Failed to create module 3");

        let r1 = m1.invoke("f", &[]).expect("invoke m1.f");
        assert_eq!(r1[0], 42, "m1.f() == 42");

        let args = [100, 200];
        let r2 = m2.invoke("add", &args).expect("invoke m2.add");
        assert_eq!(r2[0], 300, "m2.add(100,200) == 300");

        assert!(m3.memory_size() >= 65536, "m3 has memory");
    }

    #[test]
    fn test_repeated_create_destroy() {
        for i in 0..100 {
            let module =
                Module::new(test_fixtures::RETURN42_WASM).expect("Failed to create module in loop");
            let results = module
                .invoke("f", &[])
                .expect("Failed to invoke function in loop");
            assert_eq!(results[0], 42, "f() == 42 in loop iteration {}", i);
        }
    }
}
