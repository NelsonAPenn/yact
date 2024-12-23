use git2::{
    build::{CheckoutBuilder, TreeUpdateBuilder},
    MergeOptions, Pathspec, Repository, Tree, TreeWalkMode, TreeWalkResult,
};
use std::path::Path;

mod builtin_transformers;
mod config;
mod external_transformers;
#[cfg(test)]
mod tests;
mod transformer;

pub use builtin_transformers::BuiltinTransformer;
pub use config::{Configuration, ConfigurationItem, TransformerOptions};
pub use external_transformers::ShellCommandTransformer;
pub use transformer::{create_shell_transformer, transform, Transformer};

#[derive(Debug)]
pub enum Error {
    ConfigurationNotFound,
    ConfigurationParseError(toml::de::Error),
    ConfigurationEncodingError(std::str::Utf8Error),
    RepositoryIsBare,

    /// An error was returned from `libgit2`.
    GitError(git2::Error),

    /// One of the transformers encountered an error.
    TransformerError(String),

    /// No other errors, but the resulting index was empty.
    ///
    /// The commit should be aborted.
    EmptyIndex,
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self::ConfigurationParseError(err)
    }
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

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::ConfigurationEncodingError(err)
    }
}

fn build_worktree_slice<'repo>(
    repo: &'repo Repository,
    formatted: &'repo Tree,
    ancestor: &'repo Tree,
) -> Result<Tree<'repo>, git2::Error> {
    let mut builder = TreeUpdateBuilder::new();
    let repo_path = repo.workdir().unwrap();
    formatted.walk(TreeWalkMode::PreOrder, |path, entry| {
        let relative_file_path = Path::new(path).join(entry.name().unwrap());
        let absolute_file_path = repo_path.join(&relative_file_path);
        if absolute_file_path.is_file() {
            let oid = repo
                .odb()
                .unwrap()
                .write(
                    git2::ObjectType::Blob,
                    &std::fs::read(absolute_file_path).unwrap(),
                )
                .unwrap();
            builder.upsert(
                relative_file_path.to_str().unwrap(),
                oid,
                git2::FileMode::Blob,
            );
        }
        TreeWalkResult::Ok
    })?;
    repo.find_tree(builder.create_updated(repo, ancestor).unwrap())
}

pub fn load_configuration(repository: &Repository) -> Result<Configuration, Error> {
    let path = repository.workdir().ok_or(Error::RepositoryIsBare)?;

    let file =
        std::fs::read(path.join(".yactrc.toml")).map_err(|_| Error::ConfigurationNotFound)?;
    let config_str = std::str::from_utf8(&file)?;

    Ok(toml::from_str(config_str)?)
}

pub fn pre_commit<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    let repository = Repository::discover(path)?;
    let configuration = load_configuration(&repository)?;
    let mut index = repository.index()?;
    let index_tree = repository.find_tree(index.write_tree()?)?;
    let last_committed_tree = repository.head()?.peel_to_tree()?;
    let mut diff =
        repository.diff_tree_to_tree(Some(&last_committed_tree), Some(&index_tree), None)?;
    diff.find_similar(None)?;
    let mut transformed_tree_builder = TreeUpdateBuilder::new();

    for entry in diff.deltas() {
        if !entry.new_file().is_binary() {
            let matching_config_item = configuration.items.iter().find(|config_item| {
                Pathspec::new([&config_item.pathspec])
                    .unwrap()
                    .matches_path(
                        entry.new_file().path().unwrap(),
                        git2::PathspecFlags::DEFAULT,
                    )
            });
            if matching_config_item.is_none() {
                continue;
            }
            let transformers = matching_config_item
                .unwrap()
                .transformers
                .iter()
                .map(|x| x.transformer())
                .collect::<Vec<_>>();

            eprintln!(
                "Transforming staged file: {}",
                entry.new_file().path().unwrap().to_str().unwrap()
            );
            let oid = transformer::apply_transform_pipeline(
                &repository,
                &repository.find_blob(entry.new_file().id())?,
                &transformers,
            )?;
            transformed_tree_builder.upsert(
                entry.new_file().path_bytes().unwrap(),
                oid,
                entry.new_file().mode(),
            );
        }
    }

    let transformed_tree =
        repository.find_tree(transformed_tree_builder.create_updated(&repository, &index_tree)?)?;
    index.read_tree(&transformed_tree)?;
    index.write()?;

    let mini_worktree = build_worktree_slice(&repository, &transformed_tree, &index_tree)?;

    let mut merged_index = repository.merge_trees(
        &index_tree,
        &mini_worktree,
        &transformed_tree,
        Some(
            MergeOptions::new()
                .file_favor(git2::FileFavor::Ours)
                .fail_on_conflict(false),
        ),
    )?;
    /*
     * Build a tree for each file in the transformed tree from the workdir,
     * merge trees (use ours), and checkout changes (update only, force).
     */
    repository.checkout_index(
        Some(&mut merged_index),
        Some(
            CheckoutBuilder::new()
                .allow_conflicts(true)
                .update_only(true)
                .update_index(false)
                .force(),
        ),
    )?;

    let final_diff =
        repository.diff_tree_to_tree(Some(&last_committed_tree), Some(&transformed_tree), None)?;

    if final_diff.stats()?.files_changed() == 0 {
        return Err(Error::EmptyIndex);
    }

    Ok(())
}
