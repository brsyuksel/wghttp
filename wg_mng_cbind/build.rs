use bindgen;
use std::env;
use std::path::PathBuf;

fn main() {
    cc::Build::new()
        .file("unmanaged/wireguard.c")
        .compile("wireguard");

    println!("cargo:rustc-link-search=.");
    println!("cargo:rustc-link-lib=wireguard");
    println!("cargo:rerun-if-changed=unmanaged/wireguard.h");

    let bindings = bindgen::Builder::default()
        .header("unmanaged/wireguard.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("wireguard_bindings.rs"))
        .expect("can't write to file");
}
