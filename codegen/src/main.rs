mod type_def;

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write as _};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use glob::glob;

use crate::type_def::TypeDef;

const DATA_GLOB: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../types/data/*/*/*.yaml");
const META_GLOB: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../types/meta/*.yaml");

fn main() {
    let out_dir = get_out_dir();
    println!("writing generated files into {}", out_dir.display());

    match fs::remove_dir_all(&out_dir) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => panic!("failed to remove old generated dir: {err}"),
    }

    fs::create_dir(&out_dir).expect("failed to recreate generated dir");

    let mut type_defs = HashMap::<String, TypeDef>::new();
    for meta in glob(META_GLOB).unwrap() {
        let meta = meta.unwrap();
        let name = meta.file_stem().unwrap().to_string_lossy().to_string();

        let def = File::open(meta).unwrap();
        let def = serde_yaml::from_reader(def).unwrap();

        type_defs.insert(name, def);
    }

    for path in glob(DATA_GLOB).unwrap() {
        let path = path.unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap();

        let mut dirs = path
            .ancestors()
            .skip(1)
            .map(|p| p.file_name().unwrap().to_str().unwrap());
        let version = dirs.next().unwrap();
        let firmware = dirs.next().unwrap();

        let firmware = match firmware {
            "Betaflight" => "Betaflight",
            "INAV" => "Inav",
            _ => panic!("invalid firmware: `{firmware}`"),
        };

        let fw_version = format!("{firmware}{}", version.replace('.', "_"));

        let f = File::open(&path).unwrap();
        let data = serde_yaml::from_reader(f).unwrap();

        let def = type_defs.get_mut(name).unwrap();
        def.add_data(fw_version, data);
    }

    for (name, def) in type_defs {
        let mut out_path = out_dir.clone();
        out_path.push(name);
        out_path.set_extension("rs");
        let mut out = File::create(&out_path).unwrap();

        let tokens = def.expand();
        let src = rustfmt(&tokens.to_string());
        out.write_all(src.as_bytes()).unwrap();
    }
}

fn get_out_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop();
    dir.push("src/generated");
    dir
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
