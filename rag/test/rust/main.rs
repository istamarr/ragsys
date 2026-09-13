use std::fs::read;
use wasmer::{Store, Module, Instance, imports};
use wasmer_wasi::WasiState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //parent class
    let wasm_bytes = read("pkg/my_wasm_lib_bg.wasm")?;
    let store = Store::default();
    let module = Module::new(&store, wasm_bytes)?;
    let wasi_env = WasiState::new("example").finalize()?;
    let import_object = wasi_env.import_object(&module)?;

    //extend class
    let instance = Instance::new(&module, &import_object)?;
    let client_function = instance.exports.get_function("llm_pgd_client")?;

    let request = "World";
    let result = client_function.call(&[request.into()])?;

    println!("{}", result[0].to_string()?);
    Ok(())
}
