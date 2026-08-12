use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    let zwasm_src_dir = env::current_dir()
        .expect("Failed to get current dir")
        .join("zwasm");

    println!("cargo:rerun-if-env-changed=DOCS_RS");

    // docs.rs builds in an offline sandbox that has no Zig toolchain, so the zwasm C
    // library cannot be built there. rustdoc never links the native library, so we
    // generate bindings straight from the vendored header and skip the Zig build.
    let header_path = if env::var_os("DOCS_RS").is_some() {
        zwasm_src_dir.join("include/zwasm.h")
    } else {
        build_zwasm(&out_dir, &zwasm_src_dir)
    };

    if !header_path.exists() {
        panic!(
            "Error: zwasm.h not found at {}.\n\
            The header file must be present for bindgen to generate Rust bindings.\n\
            Please ensure the zwasm C build step completed successfully and the header is copied to the expected location.",
            header_path.display()
        );
    }

    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .allowlist_function("zwasm_.*")
        .allowlist_type("zwasm_.*")
        .allowlist_var("zwasm_.*")
        .generate()
        .expect("Unable to generate bindings with bindgen. Please check that zwasm.h is valid and accessible.");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings.rs! Check write permissions and disk space.");
}

/// Builds the zwasm C library with Zig, emits the link directives, and returns the
/// path to the installed `zwasm.h`.
fn build_zwasm(out_dir: &Path, zwasm_src_dir: &Path) -> PathBuf {
    let zig_local_cache_dir = out_dir.join("zig-local-cache");
    let zig_global_cache_dir = out_dir.join("zig-global-cache");
    let zig_install_prefix = out_dir.join("zig-install");

    fs::create_dir_all(&zig_local_cache_dir).expect("Failed to create Zig local cache directory");
    fs::create_dir_all(&zig_global_cache_dir).expect("Failed to create Zig global cache directory");
    fs::create_dir_all(&zig_install_prefix).expect("Failed to create Zig install directory");

    // Check if zig is available
    if Command::new("zig").arg("--version").output().is_err() {
        panic!("Error: 'zig' command not found. Please install Zig and ensure it is in your PATH.");
    }

    // Build zwasm C library using zig
    let status = Command::new("zig")
        .current_dir(zwasm_src_dir)
        .env("ZIG_LOCAL_CACHE_DIR", &zig_local_cache_dir)
        .env("ZIG_GLOBAL_CACHE_DIR", &zig_global_cache_dir)
        .arg("build")
        .arg("lib")
        .arg("-Dpic=true")
        .arg("-Dcompiler-rt=true")
        .arg("-Doptimize=ReleaseSafe")
        .arg("-p")
        .arg(zig_install_prefix.to_str().unwrap())
        .status()
        .expect("Failed to execute 'zig build lib'. Is Zig installed and zwasm source present?");

    if !status.success() {
        panic!("Error: Failed to build zwasm C library with Zig. Please check the build output for details.");
    }

    let lib_dir = zig_install_prefix.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=zwasm");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());

    zig_install_prefix.join("include/zwasm.h")
}
