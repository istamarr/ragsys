import 'dart:typed_data';
import 'dart:html';
import 'package:js/js.dart';

@JS()
@anonymous
class WasmModule {
  external dynamic get exports;
}

@JS()
external dynamic WebAssembly(dynamic a, dynamic b);

Future<WasmModule> loadWasm(String url) async {
  final response = await HttpRequest.request(url, responseType: 'arraybuffer');
  final bytes = response.response as ByteBuffer;
  final module = await promiseToFuture(WebAssembly.compile(bytes));
  return await promiseToFuture(WebAssembly.instantiate(module, {}));
}

void main() async {
  final module = await loadWasm('pkg/my_wasm_lib_bg.wasm');
  final clientFunction = module.exports.llm_pgd_client;
  final result = clientFunction('World');
  print(result);  // print "Devel, World!"
}
