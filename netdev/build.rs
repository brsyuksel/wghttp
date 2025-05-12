fn main() {
    cc::Build::new()
        .files(["src/libnetdev/libnetdev.c"])
        .include("src/libnetdev")
        .compile("netdev");

    println!("cargo:rerun-if-changed=src/libnetdev/libnetdev.c");
    println!("cargo:rerun-if-changed=src/libnetdev/libnetdev.h");
}
