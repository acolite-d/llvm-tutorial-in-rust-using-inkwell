use std::env;
use std::process::Command;

fn main() {

    let out_dir = env::var("OUT_DIR").unwrap();

    let status = Command::new("clang")
        .args(["-shared", "src/clib/io.c"])
        .arg("-o")
        .arg(format!("{}/libio.so", out_dir))
        .status()
        .expect("Failed to invoke clang and build shared library for external C functions!");

    if !status.success() {
        panic!("Compilation of C add-on libraries failed!");
    }

    println!("cargo:rustc-link-search=native={out_dir}");
    println!("cargo:rustc-link-lib=dylib=io");
}