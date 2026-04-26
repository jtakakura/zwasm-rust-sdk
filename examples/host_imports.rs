mod common;

use std::error::Error;

use zwasm_sdk::{Imports, Module};

fn main() -> Result<(), Box<dyn Error>> {
    let mut imports = Imports::new()?;

    imports.add_fn("env", "add", 2, 1, |args, results| {
        if args.len() != 2 || results.len() != 1 {
            return Err("invalid arg/result count".to_string());
        }

        let a = args[0] as i32;
        let b = args[1] as i32;
        results[0] = (a + b) as u64;
        Ok(())
    })?;

    let module = Module::new_with_imports(common::IMPORT_WASM, &imports)?;
    let results = module.invoke("call_add", &[])?;

    println!("call_add() -> {}", results[0]);
    Ok(())
}
