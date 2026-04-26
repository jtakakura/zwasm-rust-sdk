mod common;

use std::error::Error;

use zwasm_sdk::{Module, WasiConfig};

fn main() -> Result<(), Box<dyn Error>> {
    let mut wasi = WasiConfig::new()?;

    wasi.set_argv(&["guest-program", "--flag"])?;
    wasi.set_env(&[("DEMO", "1"), ("RUNTIME", "zwasm")])?;
    wasi.preopen_dir(".", "/sandbox")?;

    // The fixture itself does not use WASI. This example focuses on configuration flow.
    let module = Module::new_wasi_configured(common::RETURN42_WASM, &wasi)?;
    let results = module.invoke("f", &[])?;

    println!("f() with WASI config -> {}", results[0]);
    Ok(())
}
