use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};

extern crate sha1;

fn main() {
    let branch = std::env::args()
        .nth(1)
        .expect("Usage: branchswitch <branch>");

    let python = DependencyDefinition {
        file: Path::new("requirements.txt"),
        install_cmd: Cmd {
            cmd: "pip",
            args: &["install", "-r", "requirements.txt"],
        },
    };

    let definitions = vec![&python];

    let checksummed: Vec<ChecksumedDependencyDefinition> = definitions
        .into_iter()
        .map(|def| def.note_checksum())
        .filter(|res| res.is_ok())
        .map(|res| res.unwrap())
        .collect();

    let switch_cmd = Cmd {
        cmd: "git",
        args: &["switch", &branch],
    };
    if let Err(e) = switch_cmd.run() {
        println!("{:?}", e);
        panic!(e)
    }
    println!("We got this far");

    let errors: Vec<CommandError> = checksummed
        .into_iter()
        .map(|checksummed_definition| checksummed_definition.update_dependencies_if_necessary())
        .filter(|res| !res.is_ok())
        .map(|res| res.unwrap_err())
        .collect();

    for error in errors {
        match error {
            CommandError::IoError(e) => println!("{}", e),
            CommandError::RunError => println!("something went wrong"),
        }
    }
}

// #[derive(Copy, Clone)]
struct DependencyDefinition<'a> {
    file: &'a Path,
    install_cmd: Cmd<'a>,
}

impl DependencyDefinition<'_> {
    fn checksum(&self) -> io::Result<String> {
        let mut buffer = Vec::new();

        let mut f = File::open(self.file)?;
        f.read_to_end(&mut buffer)?;
        let mut m = sha1::Sha1::new();
        m.update(&buffer);

        Ok(m.digest().to_string())
    }
    fn note_checksum(&self) -> io::Result<ChecksumedDependencyDefinition> {
        let checksum = self.checksum()?;

        Ok(ChecksumedDependencyDefinition {
            definition: DependencyDefinition {
                file: self.file.clone(),
                install_cmd: Cmd {
                    cmd: self.install_cmd.cmd.clone(),
                    args: self.install_cmd.args.clone(),
                },
            },
            checksum_before_switch: checksum,
        })
    }
}

// #[derive(Copy, Clone)]
struct Cmd<'a> {
    cmd: &'a str,
    args: &'a [&'a str],
}

impl Cmd<'_> {
    fn run(&self) -> Result<(), CommandError> {
        let mut child = Command::new(self.cmd)
            .args(self.args)
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
}

struct ChecksumedDependencyDefinition<'a> {
    definition: DependencyDefinition<'a>,
    checksum_before_switch: String,
}

impl ChecksumedDependencyDefinition<'_> {
    fn update_dependencies_if_necessary(&self) -> Result<(), CommandError> {
        let new_checksum = self.definition.checksum()?;
        if new_checksum == self.checksum_before_switch {
            return Ok(());
        }

        self.definition.install_cmd.run()
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
