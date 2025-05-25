use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    let cuda_include_path = std::path::Path::new(std::env!("CUDA_PATH")).join("include");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", cuda_include_path.to_str().unwrap()))
        .allowlist_function("cudaGraphicsD3D11RegisterResource")
        .allowlist_function("cudaGraphicsMapResources")
        .allowlist_function("cudaGraphicsResourceGetMappedPointer")
        .allowlist_function("cudaGraphicsUnmapResources")
        .allowlist_function("cudaGraphicsUnregisterResource")
        .allowlist_type("cudaGraphicsResource_t")
        .allowlist_type("cudaStream_t")
        .allowlist_type("cudaError_t")
        .generate()
        .expect("Unable to generate CUDA interop bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!(
        "cargo:warning=Writing bindings into: {}",
        out_path.to_str().unwrap()
    );
    bindings
        .write_to_file(out_path.join("cuda_interop.rs"))
        .expect("Couldn't write bindings!");
}
