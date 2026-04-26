//! # zwasm-sdk
//!
//! Rust bindings for [zwasm](https://github.com/clojurewasm/zwasm): a small, fast, and spec-complete WebAssembly runtime written in Zig.
//!
//! ## Features
//! - **Tiny and fast**: ~1.2MB binary, JIT for ARM64/x86_64, full SIMD, threads, GC, exception handling, and more.
//! - **100% spec conformance**: Passes all official Wasm spec tests and proposals through 3.0.
//! - **Component Model**: WIT parser, Canonical ABI, WASI Preview 1+2, component linking.
//! - **Security**: Deny-by-default WASI, capability flags, resource limits.
//! - **Zero dependencies**: Pure Zig core, no libc required.
//!
//! ## Supported platforms
//! - Linux (x86_64, aarch64)
//! - macOS (aarch64)
//! - Windows (x86_64)
//!
//! ## Example
//! ```rust
//! use zwasm_sdk::{Module};
//! let wasm_bytes = include_bytes!("minimal.wasm");
//! let module = Module::new(wasm_bytes)?;
//! let results = module.invoke("f", &[])?;
//! assert_eq!(results[0], 42);
//! ```
//!
//! ## Design
//! zwasm uses a 4-tier execution pipeline:
//! - Bytecode → Predecoded IR → Register IR → Native JIT (ARM64/x86_64)
//! - All Wasm 3.0 proposals, threads, SIMD, GC, exception handling supported
//! - Allocator-parameterized: caller controls memory allocation
//!
//! See the upstream [README](https://github.com/clojurewasm/zwasm) and [ARCHITECTURE.md](https://github.com/clojurewasm/zwasm/blob/main/ARCHITECTURE.md) for details.
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
pub use module::Module;
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
