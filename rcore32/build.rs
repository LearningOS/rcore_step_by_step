extern crate cc;

fn main() {
    println!("cargo:rerun-if-env-changed=LOG");
    println!("cargo:rerun-if-env-changed=BOARD");

    let sfsimg = std::env::var("SFSIMG").unwrap();
    println!("cargo:rerun-if-changed={}", sfsimg);
    println!("cargo:rerun-if-env-changed=SFSIMG");
}
