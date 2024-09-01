use std::process::ExitCode;

use clap::Parser;
use yact::{pre_commit, BuiltinTransformer, Error, ShellCommandTransformer, TransformerOptions};

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /*
     * TODO: add config file and configuration
     * #[arg(short, long, value_name="CONFIG_FILE")]
     * config: Option<PathBuf>,
     */
}

pub fn main() -> ExitCode {
    let _cli = Args::parse();

    let repo = git2::Repository::discover(".").unwrap();
    let path = repo.workdir().unwrap();

    let file = std::fs::read(path.join(".yactrc.toml"))
        .map_err(|_| Error::ConfigurationNotFound)
        .unwrap();
    let config_str = std::str::from_utf8(&file).unwrap();

    let config = toml::from_str(config_str).unwrap();

    match pre_commit(&config, ".") {
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
        Ok(_) => ExitCode::SUCCESS,
        _ => panic!(),
    }
}
