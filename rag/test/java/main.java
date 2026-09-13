import org.wasmer.Instance;
import org.wasmer.Module;
import org.wasmer.exports.Function;
import org.wasmer.exports.WasmerInstance;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

public class WasmExample {
    public static void main(String[] args) throws Exception {
        Path wasmPath = Paths.get("pkg/my_wasm_lib_bg.wasm");//lib
        byte[] wasmBytes = Files.readAllBytes(wasmPath);

        Module module = Module.fromBinary(wasmBytes);
        Instance instance = module.instantiate(null);

        //v.1
        Function clientFunction = instance.exports.getFunction("llm_pgd_client");
        Object result = clientFunction.apply("World");
        System.out.println(result.toString());
        instance.close();
    }
}

/**
 // in service call C or Rust Function

 import io.github.kawamuray.wasmtime.*;
 import io.github.kawamuray.wasmtime.Module;

 import java.nio.file.Files;
 import java.nio.file.Paths;
 import java.util.Arrays;

 public class WasmerSrv {
     private Engine engine;
     private static Store store;
     private static Instance instance;

     public void initialize() {
         engine = new Engine();
         store = new Store(engine);

         try {
             byte[] wasmBytes = Files.readAllBytes(Paths.get("pkg/my_wasm_lib_bg.wasm"));
             Module module = Module.fromBinary(engine, wasmBytes);

             instance = new Instance(store, module, Arrays.asList(new Extern[0]));

             bindFunctions();
         } catch (Exception e) {
             throw new RuntimeException("WASM initialization failed", e);
         }
     }

     private static void bindFunctions() {
         Func function = (Func) instance.getFunc(store, "hello_world").get();

         Val[] result = function.call(store, Val.fromI32(5), Val.fromI32(10));
         System.out.println("Result: " + result[0].i32());
     }

     public void shutdown() {
         if (instance != null) instance.close();
         if (store != null) store.close();
         if (engine != null) engine.close();
     }

     public static void main(String[] args) {
         bindFunctions();
     }

 }


 */
