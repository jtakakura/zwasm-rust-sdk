mod common;

use std::env;
use std::error::Error;

use zwasm_sdk::Module;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    // Usage:
    //   cargo run --example run_wasm
    //   cargo run --example run_wasm -- ./path/to/module.wasm
    //   cargo run --example run_wasm -- ./path/to/module.wasm add 2 3
    // When no path is provided, a built-in fixture module is used.
    let wasm_bytes = if let Some(path) = args.get(1) {
        std::fs::read(path)?
    } else {
        common::RETURN42_WASM.to_vec()
    };

    let module = Module::new(&wasm_bytes)?;

    println!("Exported functions:");
    for i in 0..module.export_count() {
        if let Some(name) = module.export_name(i) {
            println!("  - {name}");
        }
    }

    if args.len() >= 3 {
        let name = &args[2];
        let call_args: Vec<u64> = args[3..]
            .iter()
            .map(|s| s.parse::<u64>())
            .collect::<Result<Vec<_>, _>>()?;

        let results = module.invoke(name, &call_args)?;
        println!("{name}({call_args:?}) -> {results:?}");
    } else if args.get(1).is_none() {
        let results = module.invoke("f", &[])?;
        println!("f() -> {results:?}");
    }

    Ok(())
}
