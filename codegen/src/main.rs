#![allow(clippy::print_stdout)]

use std::fs::{self, File};
use std::io::{self, Write};

use glob::glob;

fn main() {
    let out_dir = codegen::get_out_dir();
    println!("writing generated files into {}", out_dir.display());

    match fs::remove_dir_all(&out_dir) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => panic!("failed to remove old generated dir: {err}"),
    }

    fs::create_dir(&out_dir).expect("failed to recreate generated dir");

    for yaml in glob(&codegen::get_types_glob()).unwrap() {
        let yaml = yaml.unwrap();
        let filename = yaml.file_stem().unwrap();

        let mut out_path = out_dir.clone();
        out_path.push(filename);
        out_path.set_extension("rs");
        let mut out = File::create(&out_path).unwrap();

        let yaml = File::open(yaml).unwrap();
        let s = std::io::read_to_string(yaml).unwrap();

        let src = codegen::run(&s);
        out.write_all(src.as_bytes()).unwrap();
    }
}
