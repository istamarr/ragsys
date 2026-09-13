from wasmer import engine, Store, Module, Instance

# Load and compile the WebAssembly module
with open('pkg/my_wasm_lib_bg.wasm', 'rb') as f:
    wasm_bytes = f.read()

store = Store(engine.JIT)
module = Module(store, wasm_bytes)
instance = Instance(module)

# Access and call the "llm_pgd_client" function
client_function = instance.exports.llm_pgd_client
result = client_function('World')

print(result)  # print "Devel, World!"
