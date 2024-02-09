use git2::{
    build::{CheckoutBuilder, TreeUpdateBuilder},
    ApplyLocation, DiffOptions, Pathspec, Repository,
};
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
pub use transformer::{create_shell_transformer, transform, Transformer};

pub mod transformer {
    use git2::{Blob, Oid, Repository};
    use std::io::Write;
    use std::process::Stdio;

    /// A generic trait for transforming staged files.
    ///
    /// Example implementors might be a builtin trailing whitespace transformer,
    /// or shell transformer.
    pub trait Transformer: Fn(&[u8]) -> Result<Vec<u8>, String> {}
    impl<T> super::Transformer for T where T: Fn(&[u8]) -> Result<Vec<u8>, String> {}

    /// Apply a transform to an existing blob, creating another (for example,
    /// applying linting)
    pub fn transform<T>(
        repository: &Repository,
        blob: &Blob,
        transformer: T,
    ) -> Result<Oid, crate::Error>
    where
        T: Transformer,
    {
        let transformed = transformer(blob.content())?;
        Ok(repository.blob(transformed.as_slice())?)
    }

    /// Apply many transform to an existing blob, creating another (for example,
    /// applying linting)
    pub fn apply_transform_pipeline(
        repository: &Repository,
        blob: &Blob,
        transformers: &Vec<Box<dyn Transformer>>,
    ) -> Result<Oid, crate::Error> {
        let mut transformer_iter = transformers.iter();
        let mut transformed = transformer_iter.next().expect("at least one item")(blob.content())?;
        for transformer in transformer_iter {
            transformed = transformer(transformed.as_slice())?;
        }

        Ok(repository.blob(transformed.as_slice())?)
    }

    /// create a shell transformer from a command with process and arguments
    /// configured.
    pub fn create_shell_transformer<T: Fn() -> std::process::Command>(
        command_getter: T,
    ) -> impl Transformer {
        move |data: &[u8]| {
            let mut child = command_getter()
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|_| "shell transformer failed")?;
            let mut stdin = child.stdin.take().ok_or("failed to get stdin")?;
            let clone = data.iter().copied().collect::<Vec<u8>>();
            std::thread::spawn(move || {
                stdin
                    .write_all(clone.as_slice())
                    .expect("Failed to write to stream");
            });
            let stdout = child
                .wait_with_output()
                .map_err(|_| "Failed to read stdout")?
                .stdout;
            Ok(stdout)
        }
    }

    pub mod transformers {
        use super::Transformer;

        pub fn trailing_whitespace(data: &[u8]) -> Result<Vec<u8>, String> {
            let str_data = std::str::from_utf8(data).map_err(|err| format!("{:?}", err))?;
            let mut out = String::with_capacity(data.len());
            for line in str_data.lines() {
                out.push_str(line.trim_end());
                out.push('\n');
            }
            Ok(out.into_bytes())
        }
    }
}
#[derive(Debug)]
pub enum Error {
    GitError(git2::Error),
    TransformerError(String),
}

impl From<git2::Error> for Error {
    fn from(err: git2::Error) -> Self {
        Self::GitError(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::TransformerError(err)
    }
}

pub fn pre_commit(configuration: &Configuration) -> Result<(), git2::Error> {
    let repository = Repository::discover(".")?;
    let mut index = repository.index()?;
    let index_tree_id = index.write_tree()?;
    let index_tree = repository.find_tree(index_tree_id)?;
    eprintln!("Created tree {:?} from index", index_tree);
    let last_committed_tree = repository.head()?.peel_to_tree()?;
    eprintln!("Calculating staged diff...");
    let mut diff =
        repository.diff_tree_to_tree(Some(&last_committed_tree), Some(&index_tree), None)?;
    diff.find_similar(None)?;
    eprintln!("Transforming files...");
    let mut transformed_tree_builder = TreeUpdateBuilder::new();

    for entry in diff.deltas() {
        if !entry.new_file().is_binary() {
            let matching_pathspec = configuration.keys().find(|pathspec| {
                Pathspec::new([pathspec]).unwrap().matches_path(
                    entry.new_file().path().unwrap(),
                    git2::PathspecFlags::DEFAULT,
                )
            });
            if matching_pathspec.is_none() {
                continue;
            }
            let matching_pathspec = matching_pathspec.unwrap();
            let transformers = configuration[matching_pathspec]
                .iter()
                .map(|x| x.transformer())
                .collect();

            eprintln!("Transforming entry {:?}", entry.new_file().path().unwrap());
            let oid = transformer::apply_transform_pipeline(
                &repository,
                &repository.find_blob(entry.new_file().id())?,
                &transformers,
            )
            .unwrap();
            transformed_tree_builder.upsert(
                entry.new_file().path_bytes().unwrap(),
                oid,
                entry.new_file().mode(),
            );
        }
    }

    let transformed_tree_id = transformed_tree_builder.create_updated(&repository, &index_tree)?;
    let transformed_tree = repository.find_tree(transformed_tree_id)?;
    eprintln!("Created transformed tree {:?}...", transformed_tree);
    index.read_tree(&transformed_tree)?;
    index.write()?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub enum BuiltinTransformer {
    TrailingWhitespace,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ShellCommandTransformer {
    Rustfmt,
    Prettier,
    PyIsort,
    PyBlack,
}

impl ShellCommandTransformer {
    pub fn command_str(&self) -> &'static str {
        match self {
            Self::Rustfmt => "rustfmt",
            Self::Prettier => "prettier",
            Self::PyIsort => "isort",
            Self::PyBlack => "black",
        }
    }

    pub fn configure_command(&self, command: &mut std::process::Command) {
        match self {
            Self::Rustfmt => {
                command.args(&["--emit", "stdout"]);
            }
            _ => {
                todo!();
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum TransformerOptions {
    Builtin(BuiltinTransformer),
    RawCommand(ShellCommandTransformer),
    Node(ShellCommandTransformer),
    Yarn(ShellCommandTransformer),
    Poetry(ShellCommandTransformer),
}

impl TransformerOptions {
    pub fn transformer(&self) -> Box<dyn Transformer> {
        match self {
            Self::Builtin(BuiltinTransformer::TrailingWhitespace) => {
                Box::new(transformer::transformers::trailing_whitespace)
            }
            Self::RawCommand(command_type) => {
                let command_type = command_type.clone();
                Box::new(create_shell_transformer(move || {
                    let mut command = std::process::Command::new(command_type.command_str());
                    command_type.configure_command(&mut command);
                    command
                }))
            }
            Self::Poetry(command_type) => {
                let command_type = command_type.clone();
                Box::new(create_shell_transformer(move || {
                    let mut command = std::process::Command::new("poetry");
                    command.args(&["run", command_type.command_str()]);
                    command_type.configure_command(&mut command);
                    command
                }))
            }
            Self::Node(command_type) => {
                let command_type = command_type.clone();
                Box::new(create_shell_transformer(move || {
                    let mut command = std::process::Command::new("npx");
                    command.arg(command_type.command_str());
                    command_type.configure_command(&mut command);
                    command
                }))
            }
            Self::Yarn(command_type) => {
                let command_type = command_type.clone();
                Box::new(create_shell_transformer(move || {
                    let mut command = std::process::Command::new("yarn");
                    command.args(&["run", command_type.command_str()]);
                    command_type.configure_command(&mut command);
                    command
                }))
            }
        }
    }
}

pub type Configuration<'a> = HashMap<&'a str, Vec<TransformerOptions>>;
