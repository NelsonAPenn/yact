/*
 * Copyright 2023, 2024, 2025, 2026 Nelson Penn
 *
 * This file is part of Yet Another Commit Transformer.
 *
 * Yet Another Commit Transformer is free software: you can redistribute it
 * and/or modify it under the terms of the GNU General Public License as
 * published by the Free Software Foundation, either version 3 of the License,
 * or (at your option) any later version.
 *
 * Yet Another Commit Transformer is distributed in the hope that it will be
 * useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
 * Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Yet Another Commit Transformer. If not, see <https://www.gnu.org/licenses/>.
 */
use std::{path::PathBuf, process::ExitCode};

use clap::{Args, Parser, Subcommand, crate_version};
use semver::Version;
use yact::{Error, init, pre_commit};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(short, long, value_name = "path to workspace")]
    path: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Format changes. (This is meant to be run automatically.)
    PreCommit,
    /// Link the yact executable into the pre-commit hook path.
    Init(InitArgs),
}

#[derive(Args)]
pub struct InitArgs {
    /// Try to overwrite the member at the pre-commit hook path if it exists.
    #[arg(short, long, default_value_t = false)]
    force: bool,
}

pub fn main() -> ExitCode {
    let args = Cli::parse();

    let result = match args.command {
        None | Some(Command::PreCommit) => {
            let version = Version::parse(crate_version!()).ok();
            pre_commit(args.path.unwrap_or(PathBuf::from(".")), version)
        }
        Some(Command::Init(init_args)) => {
            init(args.path.unwrap_or(PathBuf::from(".")), init_args.force)
        }
    };

    match result {
        Err(Error::EmptyIndex) => {
            eprintln!("Aborting commit; all staged changes were unwanted formatting changes.");
            ExitCode::FAILURE
        }
        Err(Error::TransformerError(message)) => {
            eprintln!(
                "Error occurred in one of the pre-commit transformers: {}",
                message
            );
            ExitCode::FAILURE
        }
        Err(Error::RepositoryNotFound) => {
            eprintln!(
                "Repository not found in current or parent directories. yact must be run from within a git repository."
            );
            ExitCode::FAILURE
        }
        Err(Error::UnableToDetermineYactPath) => {
            eprintln!("Unable to determine path to yact executable.");
            ExitCode::FAILURE
        }
        Err(Error::PreCommitHookAlreadyExists) => {
            eprintln!(
                "Cannot install yact in pre-commit hook path: member already exists at path. Rerun init command with --force to replace."
            );
            ExitCode::FAILURE
        }
        Err(Error::InvalidYactVersion(requirement, version)) => {
            eprintln!(
                "Repository is configured to require yact version {}, but current version is {}. Please install an appropriate version or update the requirement.",
                requirement, version
            );
            ExitCode::FAILURE
        }
        Err(Error::GitError(err)) => {
            eprintln!("Unexpected git error: {}", err);
            ExitCode::FAILURE
        }
        Err(Error::IoError(err)) => {
            eprintln!("Unexpected IO error: {}", err);
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
            eprintln!(
                "Could not resolve yactrc.toml configuration file. Ensure it is located at the root of the repository"
            );
            ExitCode::FAILURE
        }
        Err(Error::InvalidGlob(glob)) => {
            eprintln!("Invalid glob found in configuration: '{}'", glob);
            ExitCode::FAILURE
        }
        Ok(_) => ExitCode::SUCCESS,
    }
}
