use yact::{pre_commit, BuiltinTransformer, ShellCommandTransformer, TransformerOptions};

pub fn main() {
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
    pre_commit(&config).unwrap();
}
