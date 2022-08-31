fn main() {
    cc::Build::new()
        .file("./upstream/src/platform.c")
        .file("./upstream/src/tools.c")
        .file("./upstream/src/stream.c")
        .file("./upstream/src/decoders.c")
        .file("./src/negative_14_bit.c")
        .flag("-w") // Disable all warnings
        .compile("blackbox");

    println!("cargo:rustc-link-lib=blackbox");
}
