use clap::Parser;
use yact::{pre_commit, BuiltinTransformer, ShellCommandTransformer, TransformerOptions};

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /*
     * TODO: add config file and configuration
     * #[arg(short, long, value_name="CONFIG_FILE")]
     * config: Option<PathBuf>,
     */
}

pub fn main() {
    let _cli = Args::parse();
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
    pre_commit(&config, ".").unwrap();
}
