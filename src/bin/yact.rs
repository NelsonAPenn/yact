use std::process::ExitCode;

use yact::{pre_commit, BuiltinTransformer, Error, ShellCommandTransformer, TransformerOptions};

pub fn main() -> ExitCode {
    let config = [
        (
            "**/*.rs",
            vec![TransformerOptions::RawCommand(
                ShellCommandTransformer::Rustfmt,
            )],
        ),
        (
            "**/*.py",
            vec![
                TransformerOptions::Poetry(ShellCommandTransformer::PyIsort),
                TransformerOptions::Poetry(ShellCommandTransformer::PyBlack),
            ],
        ),
        (
            "**/*.md",
            vec![TransformerOptions::Builtin(
                BuiltinTransformer::TrailingWhitespace,
            )],
        ),
        (
            "*.md",
            vec![TransformerOptions::Builtin(
                BuiltinTransformer::TrailingWhitespace,
            )],
        ),
    ]
    .into_iter()
    .collect();
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
    }
}
