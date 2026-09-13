// use std::env;
// use std::path::PathBuf;
//
// fn main() {
//     // Example 1: Set a specific, absolute path
//     // Replace "C:\\Program Files\\OpenSSL-Win64" or "/usr/local/ssl"
//     // with the actual path to your OpenSSL installation.
//     let openssl_dir = "C:\\Program Files\\OpenSSL-Win64"; // Example for Windows
//     // let openssl_dir = "/path/to/your/openssl"; // Example for Linux/macOS
//
//     println!("cargo:rustc-env=OPENSSL_DIR={}", openssl_dir);
//
//     // Example 2: If OpenSSL is located within your project directory (less common)
//     // let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
//     // let custom_openssl_path = PathBuf::from(manifest_dir).join("openssl_installation");
//     // println!("cargo:rustc-env=OPENSSL_DIR={}", custom_openssl_path.display());
//
//
//     // Optional: Tell Cargo to re-run the build script if this environment variable changes
//     println!("cargo:rerun-if-env-changed=OPENSSL_DIR");
//
//     // Optional: Tell Cargo to re-run if the specific path content changes
//     // This can be complex to track, so often rerunning on env change is enough.
// }

fn main(){
    println!("Hello, world!");
}