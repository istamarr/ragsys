// //Setup Hanya Jika Env Setelah install vcpkg di windows admin user di block access
// //Hanya untuk windows karena admin user tidak dapat diakses, vcpkg seharusnys sudah solve semua
// //https://github.com/Microsoft/vcpkg check catatan simple note for installation:
// // USING VCPKG to Leptonica and TESSERACT windows
// //
// //
// //     git clone https://github.com/Microsoft/vcpkg.git
// //   cd vcpkg
// //   ./bootstrap-vcpkg.sh # "./bootstrap-vcpkg.bat" for powershell
// //   ./vcpkg integrate install
// //   ./vcpkg install leptonica
// //   ./vcpkg install tesseract
// //
// //
// // set LEPTONICA_LINK_LIBS=lept
// // set LEPTONICA_INLCUDE_PATH=C:\Users\istamar.nugraha\DEV_ISTAMAR\SYSTEM_ISTA\vcpkg\installed\x64-windows\include
// // set LEPTONICA_LINK_PATHS=C:\Users\istamar.nugraha\DEV_ISTAMAR\SYSTEM_ISTA\vcpkg\installed\x64-windows\lib
// //
// //
// // cargo:rerun-if-env-changed=TESSERACT_INCLUDE_PATHS
// // cargo:rerun-if-env-changed=TESSERACT_LINK_PATHS
// // cargo:rerun-if-env-changed=TESSERACT_LINK_LIBS
// //
// //
// // export TESSERACT_INCLUDE_PATHS=C:\Users\istamar.nugraha\DEV_ISTAMAR\SYSTEM_ISTA\vcpkg\installed\x64-windows\include
// //  export TESSERACT_LINK_PATHS=C:\Users\istamar.nugraha\DEV_ISTAMAR\SYSTEM_ISTA\vcpkg\installed\x64-windows\lib
// //  export TESSERACT_LINK_LIBS=tesseract
// //

// // fn main() {
// //     let vcpkg_root = "C:\\Users\\istamar.nugraha\\DEV_ISTAMAR\\SYSTEM_ISTA\\vcpkg";
// //     let lib_dir = format!("{}\\installed\\x64-windows\\lib", vcpkg_root);
// //
// //     println!("cargo:rustc-link-search=native={}", lib_dir);
// //
// //     // Static linking (append :static to library names)
// //     println!("cargo:rustc-link-lib=static=zlib");
// //     println!("cargo:rustc-link-lib=static=libpng16");
// //     println!("cargo:rustc-link-lib=static=tiff");
// //     println!("cargo:rustc-link-lib=static=openjp2");
// //     println!("cargo:rustc-link-lib=static=libwebp");
// //     println!("cargo:rustc-link-lib=static=jpeg");
// //     println!("cargo:rustc-link-lib=static=gif");
// //     println!("cargo:rustc-link-lib=static=leptonica-1.85.0");
// //     println!("cargo:rustc-link-lib=static=tesseract55");
// // }

// // run : create_links.bat


// // build.rs
// use std::fs;
// use std::path::Path;

// fn main() {
//     // let vcpkg_root = "C:\\Users\\istamar.nugraha\\DEV_ISTAMAR\\SYSTEM_ISTA\\vcpkg";
//     let vcpkg_root = "C:\\Users\\thinkpad123\\vcpkg";
//     let vcpkg_lib_dir = format!("{}\\installed\\x64-windows\\lib", vcpkg_root);
//     // Absolute path so the linker /LIBPATH directive always resolves correctly
//     let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
//     let project_lib_dir = format!("{}\\libs", manifest_dir);

//     // Create libs directory if it doesn't exist
//     fs::create_dir_all(&project_lib_dir).unwrap();

//     // Copy all needed libraries
//     let libraries = [
//         ("tesseract55.lib", "tesseract.lib"),
//         ("leptonica-1.87.0.lib", "leptonica.lib"),
//         ("leptonica-1.87.0.lib", "lept.lib"),
//         ("zlib.lib", "zlib.lib"),
//         ("libpng16.lib", "libpng16.lib"),
//         ("tiff.lib", "tiff.lib"),
//         ("openjp2.lib", "openjp2.lib"),
//         ("libwebp.lib", "libwebp.lib"),
//         ("jpeg.lib", "jpeg.lib"),
//         ("gif.lib", "gif.lib"),
//     ];

//     let mut any_lib_present = false;
//     for (source, dest) in &libraries {
//         let source_path = Path::new(&vcpkg_lib_dir).join(source);
//         let dest_path = Path::new(&project_lib_dir).join(dest);

//         if source_path.exists() && !dest_path.exists() {
//             println!("cargo:warning=Copying {} to {}", source, dest);
//             let _ = fs::copy(&source_path, &dest_path);
//         }

//         // Only emit link directive if the .lib file is present in libs/
//         if dest_path.exists() {
//             any_lib_present = true;
//             let lib_name = dest.trim_end_matches(".lib");
//             println!("cargo:rustc-link-lib=static={}", lib_name);
//         }
//     }

//     // Only add the search path when at least one lib was found
//     if any_lib_present {
//         println!("cargo:rustc-link-search=native={}", project_lib_dir);
//         println!("cargo:warning=Using lib search path: {}", project_lib_dir);
//     } else {
//         println!("cargo:warning=vcpkg libs not found at {} — skipping Tesseract/Leptonica linking", vcpkg_lib_dir);
//     }

//     // Auto-logging integration
//     println!("cargo:rerun-if-changed=src/bin/");
//     println!("cargo:rerun-if-changed=build.rs");

//     // Get the binary name from environment or use a default
//     let binary_name = std::env::var("CARGO_BIN_NAME").unwrap_or_else(|_| "unknown".to_string());

//     // Only generate auto-logging for actual binaries, not for the main library
//     if binary_name != "unknown" && binary_name != "raga" {
//         // Create a generated file that includes auto-logging
//         let generated_code = format!(
//             r#"// Auto-generated logging integration for binary: {}
// // This file is automatically generated by build.rs

// // Include the auto-logging macro
// use ai_model_server::auto_binary_logging;

// // Auto-logging initialization macro call
// auto_binary_logging!("{}");

// // Re-export auto-logging macros for convenience
// pub use ai_model_server::{{auto_log, auto_progress, auto_command, auto_command_result}};

// "#,
//             binary_name, binary_name
//         );

//         // Create the generated directory if it doesn't exist
//         let generated_dir = Path::new("src").join("generated");
//         if !generated_dir.exists() {
//             let _ = fs::create_dir_all(&generated_dir);
//         }

//         // Write the generated file
//         let generated_file = generated_dir.join(format!("{}_auto_logging.rs", binary_name));
//         if let Err(e) = fs::write(&generated_file, generated_code) {
//             eprintln!("Warning: Failed to write auto-logging file: {}", e);
//         } else {
//             println!("Auto-logging generated for: {}", binary_name);
//         }
//     }
// }

fn main(){}
