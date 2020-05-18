use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};

extern crate sha1;

fn main() {
    println!("Hello");
    let path = Path::new("requirements.txt");
    if !path.exists() {
        println!("requirements.txt doesn't exist");
        return;
    }

    let mut f = File::open(path).expect("couldn't open file");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("couldn't read file");
    let mut m = sha1::Sha1::new();
    m.update(&buffer);
    let checksum_before = m.digest().to_string();

    if let Err(e) = run_cmd("git", &["switch", "master"]) {
        println!("{:?}", e);
        panic!(e)
    }
    println!("We got this far");

    let mut f = File::open(path).expect("couldn't open file");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("couldn't read file");
    let mut m = sha1::Sha1::new();
    m.update(&buffer);
    let checksum_after = m.digest().to_string();

    if checksum_before != checksum_after {
        println!("checksums don't match");
        println!("making sure dependencies are updated...");
        run_cmd("pip", &["install", "-r", "requirements.txt"]).expect("couldn't pip install");
    } else {
        println!("checksums match");
    }
}

fn run_cmd(cmd: &str, args: &[&str]) -> Result<(), CommandError> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(CommandError::RunError)
    }
}

#[derive(Debug)]
enum CommandError {
    IoError(io::Error),
    RunError,
}

impl From<io::Error> for CommandError {
    fn from(error: io::Error) -> Self {
        CommandError::IoError(error)
    }
}
