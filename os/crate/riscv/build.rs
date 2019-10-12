use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let name = env::var("CARGO_PKG_NAME").unwrap();

    if target.contains("riscv") && env::var_os("CARGO_FEATURE_INLINE_ASM").is_none() {
        fs::copy(
            format!("bin/{}.a", target),
            out_dir.join(format!("lib{}.a", name)),
        ).unwrap();

        println!("cargo:rustc-link-lib=static={}", name);
        println!("cargo:rustc-link-search={}", out_dir.display());
    }

    if target.contains("riscv32") {
        println!("cargo:rustc-cfg=riscv");
        println!("cargo:rustc-cfg=riscv32");
    } else if target.contains("riscv64") {
        println!("cargo:rustc-cfg=riscv");
        println!("cargo:rustc-cfg=riscv64");
    }
}
