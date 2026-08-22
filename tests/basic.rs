use zwasm_sdk::engine::Engine;
use zwasm_sdk::func::Func;
use zwasm_sdk::instance::Instance;
use zwasm_sdk::module::Module;
use zwasm_sdk::store::Store;
use zwasm_sdk::val::Val;

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
    let instance = Instance::new(&store, &module, &[]);
    assert!(instance.is_ok());
}

#[test]
fn test_invoke_no_args() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, RETURN42_WASM).unwrap();
    let instance = Instance::new(&store, &module, &[]).unwrap();
    let func = instance.get_func(0).unwrap();
    let results = func.call(&[]).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], Val::I32(42));
}

#[test]
fn test_invoke_with_args() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, ADD_WASM).unwrap();
    let instance = Instance::new(&store, &module, &[]).unwrap();
    let func = instance.get_func(0).unwrap();

    let args = [Val::I32(10), Val::I32(32)];
    let results = func.call(&args).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], Val::I32(42));
}

#[test]
fn test_invoke_add_zero() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, ADD_WASM).unwrap();
    let instance = Instance::new(&store, &module, &[]).unwrap();
    let func = instance.get_func(0).unwrap();

    let args = [Val::I32(0), Val::I32(0)];
    let results = func.call(&args).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], Val::I32(0));
}

#[test]
fn test_get_func_out_of_range() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, RETURN42_WASM).unwrap();
    let instance = Instance::new(&store, &module, &[]).unwrap();
    let func = instance.get_func(99);
    assert!(func.is_err());
}

#[test]
fn test_get_func_by_name() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, RETURN42_WASM).unwrap();
    let instance = Instance::new(&store, &module, &[]).unwrap();
    let func = instance.get_func_by_name(&module, "f").unwrap();
    let results = func.call(&[]).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], Val::I32(42));
}

#[test]
fn test_get_func_by_name_not_found() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, RETURN42_WASM).unwrap();
    let instance = Instance::new(&store, &module, &[]).unwrap();
    let func = instance.get_func_by_name(&module, "nonexistent");
    assert!(func.is_err());
}

#[test]
fn test_get_func_by_name_add() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();
    let module = Module::new(&store, ADD_WASM).unwrap();
    let instance = Instance::new(&store, &module, &[]).unwrap();
    let func = instance.get_func_by_name(&module, "add").unwrap();

    let args = [Val::I32(10), Val::I32(32)];
    let results = func.call(&args).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], Val::I32(42));
}

// (module
//   (import "env" "h" (func (param i32) (result i32)))
//   (func (export "f") (param i32) (result i32) (local.get 0) (call 0)))
const CALLBACK_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x06, 0x01, 0x60, 0x01, 0x7f, 0x01, 0x7f,
    0x02, 0x09, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x01, 0x68, 0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0x07,
    0x05, 0x01, 0x01, 0x66, 0x00, 0x01, 0x0a, 0x08, 0x01, 0x06, 0x00, 0x20, 0x00, 0x10, 0x00, 0x0b,
];

unsafe extern "C" fn add_one(
    args: *const zwasm_sys::wasm_val_vec_t,
    results: *mut zwasm_sys::wasm_val_vec_t,
) -> *mut zwasm_sys::wasm_trap_t {
    let arg = (*args).data;
    let res = (*results).data;
    (*res).kind = zwasm_sys::wasm_valkind_enum_WASM_I32 as u8;
    (*res).of.i32_ = (*arg).of.i32_ + 1;
    std::ptr::null_mut()
}

#[test]
fn test_host_function() {
    let engine = Engine::new().unwrap();
    let store = Store::new(&engine).unwrap();

    // Create functype: (i32) -> (i32)
    let mut params = zwasm_sys::wasm_valtype_vec_t {
        size: 0,
        data: std::ptr::null_mut(),
    };
    let mut results = zwasm_sys::wasm_valtype_vec_t {
        size: 0,
        data: std::ptr::null_mut(),
    };
    let param_type =
        unsafe { zwasm_sys::wasm_valtype_new(zwasm_sys::wasm_valkind_enum_WASM_I32 as u8) };
    let result_type =
        unsafe { zwasm_sys::wasm_valtype_new(zwasm_sys::wasm_valkind_enum_WASM_I32 as u8) };
    unsafe {
        zwasm_sys::wasm_valtype_vec_new(&mut params, 1, &param_type);
        zwasm_sys::wasm_valtype_vec_new(&mut results, 1, &result_type);
    };
    let functype = unsafe { zwasm_sys::wasm_functype_new(&mut params, &mut results) };

    let host_fn = unsafe { Func::new_host(&store, functype, Some(add_one)) }.unwrap();
    unsafe { zwasm_sys::wasm_functype_delete(functype) };

    let module = Module::new(&store, CALLBACK_WASM).unwrap();
    let instance = Instance::new(&store, &module, &[&host_fn]).unwrap();
    let f = instance.get_func_by_name(&module, "f").unwrap();

    let call_results = f.call(&[Val::I32(41)]).unwrap();
    assert_eq!(call_results.len(), 1);
    assert_eq!(call_results[0], Val::I32(42));
}
