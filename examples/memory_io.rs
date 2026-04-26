mod common;

use std::error::Error;

use zwasm_sdk::Module;

fn main() -> Result<(), Box<dyn Error>> {
    let module = Module::new(common::MEMORY_WASM)?;

    println!("memory size: {} bytes", module.memory_size());

    let write_buf = [0xde, 0xad, 0xbe, 0xef];
    module.memory_write(0, &write_buf)?;

    let mut read_buf = [0u8; 4];
    module.memory_read(0, &mut read_buf)?;
    println!("read after write: {read_buf:02x?}");

    module.invoke("f", &[])?;

    let snapshot = module
        .memory_data_copy()
        .ok_or("module does not export linear memory")?;
    let value = u32::from_le_bytes([snapshot[0], snapshot[1], snapshot[2], snapshot[3]]);

    println!("u32 at memory[0..4] after invoke: {value}");
    Ok(())
}
