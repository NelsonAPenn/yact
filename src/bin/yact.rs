use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use yact::{pre_commit, Error};

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long, value_name = "path to workspace")]
    path: Option<PathBuf>,
}

pub fn main() -> ExitCode {
    let args = Args::parse();
    match pre_commit(&args.path.unwrap_or(PathBuf::from("."))) {
        Err(Error::EmptyIndex) => {
            eprintln!("Aborting commit. No staged changes or they were formatted away.");
            ExitCode::FAILURE
        }
        Err(Error::TransformerError(message)) => {
            eprintln!(
                "Error occurred in one of the pre-commit transformers: {}",
                message
            );
            ExitCode::FAILURE
        }
        Err(Error::GitError(err)) => {
            eprintln!("Unexpected git error: {}", err);
            ExitCode::FAILURE
        }
        Err(Error::ConfigurationParseError(err)) => {
            eprintln!("Failed to parse configuration: {}", err);
            ExitCode::FAILURE
        }
        Err(Error::ConfigurationEncodingError(_)) => {
            eprintln!("Configuration file was not valid UTF-8.");
            ExitCode::FAILURE
        }
        Err(Error::RepositoryIsBare) => {
            eprintln!("Cannot run yact on a bare repository.");
            ExitCode::FAILURE
        }
        Err(Error::ConfigurationNotFound) => {
            eprintln!("Could not resolve .yactrc.toml configuration file. Ensure it is located at the root of the repository");
            ExitCode::FAILURE
        }
        Ok(_) => ExitCode::SUCCESS,
    }
}
