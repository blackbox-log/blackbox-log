#![allow(clippy::print_stdout)]

mod type_def;

use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use glob::glob;

use crate::type_def::TypeDef;

const TYPES_GLOB: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../types/*.yaml");

fn main() {
    let out_dir = get_out_dir();
    println!("writing generated files into {}", out_dir.display());

    match fs::remove_dir_all(&out_dir) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => panic!("failed to remove old generated dir: {err}"),
    }

    fs::create_dir(&out_dir).expect("failed to recreate generated dir");

    for yaml in glob(TYPES_GLOB).unwrap() {
        let yaml = yaml.unwrap();
        let filename = yaml.file_stem().unwrap();

        let mut out_path = out_dir.clone();
        out_path.push(filename);
        out_path.set_extension("rs");
        let mut out = File::create(&out_path).unwrap();

        let yaml = File::open(yaml).unwrap();
        let s = std::io::read_to_string(yaml).unwrap();

        let src = generate(&s);
        out.write_all(src.as_bytes()).unwrap();
    }
}

fn get_out_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop();
    dir.push("src/generated");
    dir
}

fn generate(yaml: &str) -> String {
    let type_def: TypeDef = serde_yaml::from_str(yaml).unwrap();
    let tokens = type_def.expand();

    rustfmt(&tokens.to_string())
}

fn rustfmt(src: &str) -> String {
    let mut cmd = Command::new("rustfmt")
        .arg("+nightly")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start `rustfmt +nightly`");

    let mut stdin = BufWriter::new(cmd.stdin.as_mut().unwrap());
    stdin.write_all(src.as_bytes()).unwrap();
    drop(stdin);

    let output = cmd.wait_with_output().unwrap();

    assert!(
        output.status.success(),
        "`rustfmt +nightly` exited unsuccessfully"
    );

    String::from_utf8(output.stdout).unwrap()
}
