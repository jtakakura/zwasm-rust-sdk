mod common;

use std::thread;
use std::time::{Duration, Instant};

use zwasm_sdk::{Config, Imports, Module, WasiConfig};

#[test]
fn module_validate_new_and_invoke() {
    Module::validate(common::RETURN42_WASM).expect("valid Wasm should pass validation");
    assert!(Module::validate(&[0x00, 0x00, 0x00, 0x00]).is_err());

    let module = Module::new(common::ADD_WASM).expect("module creation should succeed");
    let results = module.invoke("add", &[10, 32]).expect("invoke add");
    assert_eq!(results, vec![42]);
}

#[test]
fn invoke_nonexistent_export_returns_error() {
    let module = Module::new(common::RETURN42_WASM).expect("module creation should succeed");
    let err = module
        .invoke("missing", &[])
        .expect_err("missing export should return error");
    assert!(!err.0.is_empty());
}

#[test]
fn module_with_imports_calls_host_function() {
    let mut imports = Imports::new().expect("imports creation should succeed");
    imports
        .add_fn("env", "add", 2, 1, |args, results| {
            if args.len() != 2 || results.len() != 1 {
                return Err("invalid arg/result count".to_string());
            }
            results[0] = (args[0] as i32 + args[1] as i32) as u64;
            Ok(())
        })
        .expect("register host function");

    let module = Module::new_with_imports(common::IMPORT_WASM, &imports)
        .expect("module with imports should be created");
    let results = module.invoke("call_add", &[]).expect("invoke call_add");
    assert_eq!(results, vec![7]);
}

#[test]
fn imports_reject_nul_names() {
    let mut imports = Imports::new().expect("imports creation should succeed");

    assert!(imports.add_fn("en\0v", "add", 2, 1, |_, _| Ok(())).is_err());
    assert!(imports.add_fn("env", "ad\0d", 2, 1, |_, _| Ok(())).is_err());
}

#[test]
fn memory_read_write_and_out_of_bounds() {
    let module = Module::new(common::MEMORY_WASM).expect("memory module should be created");

    let write_data = [0xde, 0xad, 0xbe, 0xef];
    module
        .memory_write(0, &write_data)
        .expect("write in-bounds");

    let mut read_data = [0u8; 4];
    module
        .memory_read(0, &mut read_data)
        .expect("read in-bounds");
    assert_eq!(read_data, write_data);

    let oob = module.memory_size() as u32;
    assert!(module.memory_write(oob, &write_data).is_err());
    assert!(module.memory_read(oob, &mut read_data).is_err());
}

#[test]
fn configured_and_wasi_module_creation() {
    let mut config = Config::new().expect("config creation should succeed");
    config.set_timeout(10_000);
    config.set_fuel(1_000_000);
    config.set_force_interpreter(false);

    let module = Module::new_configured(common::RETURN42_WASM, &config)
        .expect("configured module should be created");
    assert_eq!(module.invoke("f", &[]).expect("invoke f"), vec![42]);

    let mut wasi = WasiConfig::new().expect("wasi config should be created");
    wasi.set_argv(&["guest", "--ok"])
        .expect("set argv should succeed");
    wasi.set_env(&[("DEMO", "1")])
        .expect("set env should succeed");
    wasi.preopen_dir(".", "/sandbox")
        .expect("preopen should succeed");

    let wasi_module = Module::new_wasi_configured(common::RETURN42_WASM, &wasi)
        .expect("wasi configured module should be created");
    assert_eq!(wasi_module.invoke("f", &[]).expect("invoke f"), vec![42]);
}

#[test]
fn wasi_config_rejects_nul_bytes() {
    let mut wasi = WasiConfig::new().expect("wasi config should be created");
    assert!(wasi.set_argv(&["ok", "bad\0arg"]).is_err());
    assert!(wasi.set_env(&[("K\0EY", "v")]).is_err());
    assert!(wasi.set_env(&[("KEY", "v\0")]).is_err());
    assert!(wasi.preopen_dir(".\0", "/sandbox").is_err());
    assert!(wasi.preopen_dir(".", "/sa\0ndbox").is_err());
}

#[test]
fn cancel_handle_interrupts_running_invoke() {
    let mut config = Config::new().expect("config creation should succeed");
    config.set_cancelable(true);

    let module = Module::new_configured(common::INFINITE_LOOP_WASM, &config)
        .expect("loop module should be created");
    let cancel_handle = module.cancel_handle();
    let started = Instant::now();

    let cancel_thread = thread::spawn(move || {
        thread::sleep(Duration::from_micros(100));
        for _ in 0..200 {
            cancel_handle.cancel();
            thread::sleep(Duration::from_micros(100));
        }
    });

    let err = module
        .invoke("loop", &[])
        .expect_err("infinite loop should only return via cancellation");

    cancel_thread.join().expect("cancel thread should join");

    assert!(
        err.is_interrupted(),
        "expected interrupted error, got: {err}"
    );
    assert!(err.is_canceled(), "expected canceled error, got: {err}");
    assert!(started.elapsed() < Duration::from_secs(2));
}
