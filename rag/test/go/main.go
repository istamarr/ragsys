package main

import (
    "fmt"
    "io/ioutil"
    "log"
    "os"

    "github.com/wasmerio/wasmer-go/wasmer"
    "github.com/wasmerio/wasmer-go/wasmer/wasi"
)

func main() {
    //parent code
    wasmBytes, err := ioutil.ReadFile("pkg/my_wasm_lib_bg.wasm")
    if err != nil {
        log.Fatalf("failed to read wasm file: %v", err)
    }
    store := wasmer.NewStore(wasmer.NewEngine())
    module, err := wasmer.NewModule(store, wasmBytes)
    if err != nil {
        log.Fatalf("failed to compile wasm module: %v", err)
    }
    wasiEnv, err := wasi.NewStateBuilder("example").Finalize()
    if err != nil {
        log.Fatalf("failed to create WASI environment: %v", err)
    }

    //extend to
    importObject, err := wasiEnv.GenerateImportObject(store, module)
    if err != nil {
        log.Fatalf("failed to generate import object: %v", err)
    }
    instance, err := wasmer.NewInstance(module, importObject)
    if err != nil {
        log.Fatalf("failed to instantiate wasm module: %v", err)
    }
    clientFunction, err := instance.Exports.GetFunction("llm_pgd_client")
    if err != nil {
        log.Fatalf("failed to get function: %v", err)
    }
    //using llm
    request := "World"
    result, err := clientFunction(request)
    if err != nil {
        log.Fatalf("failed to call function: %v", err)
    }

    fmt.Println(result)
}
