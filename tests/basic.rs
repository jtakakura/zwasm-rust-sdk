use zwasm_sdk::engine::Engine;
use zwasm_sdk::instance::Instance;
use zwasm_sdk::module::Module;
use zwasm_sdk::store::Store;

// (func (export "f") (result i32) (i32.const 42))
const RETURN42_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x03,
    0x02, 0x01, 0x00, 0x07, 0x05, 0x01, 0x01, 0x66, 0x00, 0x00, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x41,
    0x2a, 0x0b,
];

// (func (export "add") (param i32 i32) (result i32) (i32.add (local.get 0) (local.get 1)))
const ADD_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01,
    0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, 0x0a, 0x09,
    0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
];

// Minimal valid wasm: magic + version only
const MINIMAL_WASM: &[u8] = &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

fn i32_val(v: i32) -> zwasm_sys::wasm_val_t {
    zwasm_sys::wasm_val_t {
        kind: zwasm_sys::wasm_valkind_enum_WASM_I32 as u8,
        of: zwasm_sys::wasm_val_t__bindgen_ty_1 { i32_: v },
    }
}

unsafe fn get_i32(val: &zwasm_sys::wasm_val_t) -> i32 {
    val.of.i32_
}

#[test]
fn test_engine_new() {
    let engine = Engine::new();
    assert!(engine.is_ok());
}

#[test]
fn test_engine_default() {
    let _engine = Engine::default();
}

#[test]
fn test_store_new() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine);
    assert!(store.is_ok());
}

#[test]
fn test_module_new() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, RETURN42_WASM);
    assert!(module.is_ok());
}

#[test]
fn test_module_invalid_wasm() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, &[0x00, 0x00, 0x00, 0x00]);
    assert!(module.is_err());
}

#[test]
fn test_module_minimal() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, MINIMAL_WASM);
    assert!(module.is_ok());
}

#[test]
fn test_instance_new() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, RETURN42_WASM).unwrap();
    let instance = Instance::new(&store, &module);
    assert!(instance.is_ok());
}

#[test]
fn test_invoke_no_args() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, RETURN42_WASM).unwrap();
    let instance = Instance::new(&store, &module).unwrap();
    let func = instance.get_func(0).unwrap();
    let results = func.call(&[]).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(unsafe { get_i32(&results[0]) }, 42);
}

#[test]
fn test_invoke_with_args() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, ADD_WASM).unwrap();
    let instance = Instance::new(&store, &module).unwrap();
    let func = instance.get_func(0).unwrap();

    let args = [i32_val(10), i32_val(32)];
    let results = func.call(&args).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(unsafe { get_i32(&results[0]) }, 42);
}

#[test]
fn test_invoke_add_zero() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, ADD_WASM).unwrap();
    let instance = Instance::new(&store, &module).unwrap();
    let func = instance.get_func(0).unwrap();

    let args = [i32_val(0), i32_val(0)];
    let results = func.call(&args).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(unsafe { get_i32(&results[0]) }, 0);
}

#[test]
fn test_get_func_out_of_range() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, RETURN42_WASM).unwrap();
    let instance = Instance::new(&store, &module).unwrap();
    let func = instance.get_func(99);
    assert!(func.is_err());
}
