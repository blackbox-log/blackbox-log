use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};

#[test]
fn readme() -> io::Result<()> {
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

#[test]
fn workflows() -> io::Result<()> {
    let msrv = get_msrv()?;

    for entry in fs::read_dir(".github/workflows")? {
        let path = entry?.path();
        let workflow = File::open(&path)?;
        let workflow = BufReader::new(workflow);

        for (i, line) in workflow
            .lines()
            .map(Result::unwrap)
            .enumerate()
            .map(|(i, line)| (i + 1, line))
            .filter(|(_, line)| line.contains("# MSRV"))
        {
            let (line, _) = line.split_once('#').unwrap();
            let (_, line) = line.split_once(&['-', ':']).unwrap();
            let line = line.trim();

            assert_eq!(
                msrv,
                line,
                "Incorrect MSRV on line {i} of {}",
                path.display(),
            );
        }
    }

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
