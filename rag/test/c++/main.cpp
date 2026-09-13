#include <iostream>
#include <fstream>
#include <vector>
#include <wasmer.h>
#include <wasmer_wasi.h>

std::vector<char> readFile(const std::string& path) {
    std::ifstream file(path, std::ios::binary | std::ios::ate);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open file");
    }
    std::streamsize size = file.tellg();
    file.seekg(0, std::ios::beg);

    std::vector<char> buffer(size);
    if (!file.read(buffer.data(), size)) {
        throw std::runtime_error("Failed to read file");
    }

    return buffer;
}

int main() {
    try {
        // Parent Function
        // Read the WebAssembly module as bytes
        std::vector<char> wasm_bytes = readFile("pkg/my_wasm_lib_bg.wasm");
        wasmer_store_t* store = wasmer_store_new(wasmer_engine_new());
        wasmer_byte_array wasm_bytes_arr = { reinterpret_cast<const uint8_t*>(wasm_bytes.data()), wasm_bytes.size() };
        wasmer_module_t* module = nullptr;
        wasmer_result_t compile_result = wasmer_module_new(store, wasm_bytes_arr, &module);
        if (compile_result != WASMER_OK) {
            std::cerr << "Failed to compile wasm module" << std::endl;
            return 1;
        }
        // Set up the WASI environment
        wasmer_wasi_env_t* wasi_env = wasmer_wasi_env_new();
        wasmer_import_object_t* import_object = wasmer_wasi_env_generate_import_object(wasi_env, module);
        // Create a new instance of the module
        wasmer_instance_t* instance = nullptr;
        wasmer_result_t instantiate_result = wasmer_instance_new(module, import_object, &instance);
        if (instantiate_result != WASMER_OK) {
            std::cerr << "Failed to instantiate wasm module" << std::endl;
            return 1;
        }

        // extend/ include code:
        wasmer_export_func_t* client_function = nullptr;
        wasmer_instance_exports(instance, (wasmer_export_t**)&client_function, nullptr, "llm_pgd_client");
        if (!client_function) {
            std::cerr << "Failed to get function" << std::endl;
            return 1;
        }
        // Call the function
        const char* request = "World";
        wasmer_value_t params[] = { wasmer_value_t { .tag = WASM_I32, .value.I32 = reinterpret_cast<int32_t>(request) } };
        wasmer_value_t result = wasmer_value_t { .tag = WASM_I32 };

        wasmer_result_t call_result = wasmer_export_func_call(client_function, params, 1, &result, 1);
        if (call_result != WASMER_OK) {
            std::cerr << "Failed to call function" << std::endl;
            return 1;
        }

        std::cout << result.value.I32 << std::endl;
        // Clean up
        wasmer_instance_destroy(instance);
        wasmer_module_destroy(module);
        wasmer_store_destroy(store);

    } catch (const std::exception& e) {
        std::cerr << e.what() << std::endl;
        return 1;
    }

    return 0;
}
