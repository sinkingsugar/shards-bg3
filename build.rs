fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    match target_os.as_str() {
        "macos" => {
            println!("cargo:rustc-cdylib-link-arg=-Wl,-install_name,@rpath/bg3.dylib");
            println!("cargo:rustc-cdylib-link-arg=-o");
            println!("cargo:rustc-cdylib-link-arg=test/externals/bg3.dylib");
        }
        "windows" => {
            println!("cargo:rustc-cdylib-link-arg=/OUT:test/externals/bg3.dll");
        }
        _ => {
            // For Linux and other Unix-like systems
            println!("cargo:rustc-cdylib-link-arg=-o");
            println!("cargo:rustc-cdylib-link-arg=test/externals/bg3.so");
        }
    }
} 