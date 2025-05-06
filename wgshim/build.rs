fn main() {
    cc::Build::new()
        .files(["src/libwgshim/libwgshim.c", "src/libwgshim/wireguard.c"])
        .include("src/libwgshim")
        .compile("wgshim");

    println!("cargo:rerun-if-changed=src/libwgshim/libwgshim.c");
    println!("cargo:rerun-if-changed=src/libwgshim/wireguard.c");
    println!("cargo:rerun-if-changed=src/libwgshim/libwgshim.h");
    println!("cargo:rerun-if-changed=src/libwgshim/wireguard.h");
}
