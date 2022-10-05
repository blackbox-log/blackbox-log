const FILES: &[&str] = &[
    "upstream/src/platform.c",
    "upstream/src/tools.c",
    "upstream/src/stream.c",
    "upstream/src/decoders.c",
    "src/negative_14_bit.c",
];

fn main() {
    for file in FILES {
        println!("cargo:rerun-if-changed={file}");
    }

    cc::Build::new()
        .flag("-w") // Disable all warnings
        .files(FILES)
        .compile("upstream");
}
