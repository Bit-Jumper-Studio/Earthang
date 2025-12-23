fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Get the target triple
    let target = std::env::var("TARGET").unwrap();
    
    // Handle MinGW-specific linking
    if target.contains("windows-gnu") {
        // REMOVED: Don't set built-in cfg flags
        // println!("cargo:rustc-cfg=target_os=\"windows\"");
        // println!("cargo:rustc-cfg=target_env=\"gnu\"");
        
        // Instead, set a custom cfg flag
        println!("cargo:rustc-cfg=mingw_build");
        
        // Set linker flags for MinGW
        println!("cargo:rustc-link-arg=-static");
        println!("cargo:rustc-link-arg=-lmsvcrt");
        println!("cargo:rustc-link-arg=-lmingwex");
        println!("cargo:rustc-link-arg=-lmingw32");
        println!("cargo:rustc-link-arg=-lgcc");
        println!("cargo:rustc-link-arg=-Wl,-Bstatic");
        println!("cargo:rustc-link-arg=-Wl,--gc-sections");
        
        // Prevent Rust from adding problematic MSVC flags
        println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    }
}