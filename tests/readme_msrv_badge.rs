use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[test]
fn matches_metadata() -> io::Result<()> {
    let readme = File::open("README.md")?;
    let readme = BufReader::new(readme);

    let msrv = readme
        .lines()
        .map(Result::unwrap)
        .find(|line| line.starts_with("[![MSRV]("))
        .expect("MSRV badge");

    let (msrv, _) = msrv.split_once(')').unwrap();
    let (_, msrv) = msrv.rsplit_once('=').unwrap();

    assert_eq!(get_msrv()?, msrv);

    Ok(())
}

fn get_msrv() -> io::Result<String> {
    let manifest = File::open("Cargo.toml")?;
    let manifest = BufReader::new(manifest);

    let msrv = manifest
        .lines()
        .map(Result::unwrap)
        .find(|line| line.contains("rust-version"))
        .expect("rust-version line");

    let (_, msrv) = msrv.split_once('=').unwrap();
    Ok(msrv.trim().trim().trim_matches('"').to_owned())
}
