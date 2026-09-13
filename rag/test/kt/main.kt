import org.wasmer.Instance
import org.wasmer.Module
import java.nio.file.Files
import java.nio.file.Paths

fun main() {
    val wasmPath = Paths.get("pkg/my_wasm_lib_bg.wasm")
    val wasmBytes = Files.readAllBytes(wasmPath)

    val module = Module(wasmBytes)
    val instance = Instance(module)

    val clientFunction = instance.exports.getFunction("llm_pgd_client")
    val result = clientFunction.apply("World")

    println(result.toString())
    instance.close()
}