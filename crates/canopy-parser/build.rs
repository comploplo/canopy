use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let udpipe_dir = workspace_dir.join("third-party").join("udpipe");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");

    // Build UDPipe static library
    build_udpipe(&udpipe_dir, &out_dir);

    // Generate Rust bindings
    generate_bindings(&udpipe_dir, &out_dir);

    // Link with the built library
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=udpipe");

    // Link with system libraries that UDPipe needs
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=m");
    }

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=c++");
    }

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=msvcrt");
    }
}

fn build_udpipe(udpipe_dir: &Path, out_dir: &Path) {
    let src_dir = udpipe_dir.join("src");
    let lib_dir = udpipe_dir.join("src_lib_only");

    assert!(src_dir.exists(), "UDPipe source directory not found at {}. Please ensure third-party/udpipe is properly cloned.", src_dir.display());

    println!("Building UDPipe static library...");

    // Build UDPipe using its Makefile in the src directory
    let status = Command::new("make")
        .current_dir(&src_dir)
        .arg("lib")
        .status()
        .expect("Failed to execute UDPipe build command");

    assert!(status.success(), "UDPipe build failed");

    // Look for the built library (it might be in src or src_lib_only)
    let possible_lib_paths = vec![
        src_dir.join("libudpipe.a"),
        lib_dir.join("libudpipe.a"),
        src_dir.join("udpipe.a"),
    ];

    let mut lib_found = false;
    for lib_src in &possible_lib_paths {
        if lib_src.exists() {
            let lib_dst = out_dir.join("libudpipe.a");
            std::fs::copy(lib_src, &lib_dst).expect("Failed to copy UDPipe library");
            println!(
                "UDPipe library copied from {} to {}",
                lib_src.display(),
                lib_dst.display()
            );
            lib_found = true;
            break;
        }
    }

    assert!(
        lib_found,
        "UDPipe library not found in any of: {possible_lib_paths:?}"
    );

    println!("UDPipe build complete!");
}

fn generate_bindings(udpipe_dir: &Path, out_dir: &Path) {
    println!("Generating UDPipe bindings...");

    let lib_dir = udpipe_dir.join("src_lib_only");
    let header_path = lib_dir.join("udpipe.h");

    assert!(
        header_path.exists(),
        "UDPipe header not found at {}",
        header_path.display()
    );

    let bindings = bindgen::Builder::default()
        .header(header_path.to_string_lossy())
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++14")
        .clang_arg(format!("-I{}", lib_dir.display()))
        // Allowlist UDPipe types and functions - be more specific about namespaces
        .allowlist_function("ufal::udpipe::.*")
        .allowlist_type("ufal::udpipe::.*")
        .allowlist_var("ufal::udpipe::.*")
        // Also allowlist top-level items in the namespace
        .allowlist_type("token")
        .allowlist_type("word")
        .allowlist_type("multiword_token")
        .allowlist_type("empty_node")
        .allowlist_type("sentence")
        .allowlist_type("model")
        .allowlist_type("input_format")
        .allowlist_type("output_format")
        .allowlist_type("pipeline")
        .allowlist_type("trainer")
        .allowlist_type("evaluator")
        .allowlist_type("string_piece")
        // Make std types opaque to avoid binding issues
        .opaque_type("std::.*")
        .opaque_type("string")
        .opaque_type("vector")
        .opaque_type("basic_string.*")
        // Use the ufal::udpipe namespace
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        // Derive common traits for generated types
        .derive_debug(true)
        .derive_default(false) // Some C++ classes may not have default constructors
        .derive_copy(false) // C++ classes with non-trivial destructors
        // Generate the bindings
        .generate()
        .expect("Unable to generate UDPipe bindings");

    let bindings_path = out_dir.join("udpipe_bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Couldn't write UDPipe bindings");

    println!("UDPipe bindings generated at {}", bindings_path.display());

    // Tell Rust to link against the UDPipe static library
    println!("cargo:rustc-link-lib=static=udpipe");
    println!("cargo:rustc-link-search=native={}", out_dir.display());

    // Also need to link against C++ standard library
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=c++");
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=stdc++");
}
