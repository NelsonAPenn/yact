/*
 * Copyright 2023, 2024, 2025 Nelson Penn
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
use crate::{apply_transform_pipeline, load_configuration, Error};
use git2::{
    build::{CheckoutBuilder, TreeUpdateBuilder},
    FileMode, MergeOptions, Repository, Tree, TreeWalkMode, TreeWalkResult,
};
use glob::{MatchOptions, Pattern};
use semver::Version;
use std::path::Path;

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
            /*
             * TODO: complete logic for determining file mode. On Unix, symlink
             * and executable must be handled. On Windows, file mode will
             * probably have to match the file mode from the ancestor /
             * formatted tree.
             */
            let mut file_mode = FileMode::Blob;

            #[cfg(target_family = "unix")]
            {
                if let Ok(metadata) = absolute_file_path.metadata() {
                    use std::os::unix::fs::MetadataExt;

                    let raw_mode = metadata.mode();
                    if (raw_mode & 0o111) != 0 {
                        file_mode = FileMode::BlobExecutable;
                    }
                }
            }

            let oid = repo
                .odb()
                .unwrap()
                .write(
                    git2::ObjectType::Blob,
                    &std::fs::read(absolute_file_path).unwrap(),
                )
                .unwrap();
            builder.upsert(relative_file_path.to_str().unwrap(), oid, file_mode);
        }
        TreeWalkResult::Ok
    })?;
    repo.find_tree(builder.create_updated(repo, ancestor).unwrap())
}

pub fn get_last_committed_tree_or_default(repository: &Repository) -> Result<Tree<'_>, Error> {
    let last_committed_tree = repository.head().map(|x| x.peel_to_tree());
    match last_committed_tree {
        Ok(tree) => Ok(tree?),
        Err(err) if matches!(err.code(), git2::ErrorCode::UnbornBranch) => {
            let treebuilder = repository.treebuilder(None)?;
            let oid = treebuilder.write()?;
            Ok(repository.find_tree(oid)?)
        }
        Err(e) => Err(e.into()),
    }
}

/// Load configuration, apply appropriate transformers to staged changes, and
/// merge results back into worktree.
///
/// This is the core algorithm provided by this project.
pub fn pre_commit<P: AsRef<Path>>(path: P, check_version: Option<Version>) -> Result<(), Error> {
    let repository = Repository::discover(path)?;
    let repository_path = repository.workdir().ok_or(Error::RepositoryIsBare)?;
    std::env::set_current_dir(repository_path)?;
    let configuration = load_configuration(&repository)?;

    if let Some(ref version) = check_version {
        if let Some(version_requirement) = configuration.requires_yact_version {
            if !version_requirement.matches(version) {
                return Err(Error::InvalidYactVersion(
                    version_requirement.clone(),
                    version.clone(),
                ));
            }
        }
    }

    let mut index = repository.index()?;
    let index_tree = repository.find_tree(index.write_tree()?)?;
    let last_committed_tree = get_last_committed_tree_or_default(&repository)?;
    let mut diff =
        repository.diff_tree_to_tree(Some(&last_committed_tree), Some(&index_tree), None)?;
    diff.find_similar(None)?;
    let mut transformed_tree_builder = TreeUpdateBuilder::new();

    for entry in diff.deltas() {
        if !entry.new_file().is_binary() {
            let matching_config_item = configuration.items.iter().find(|config_item| {
                let pattern = Pattern::new(&config_item.glob).unwrap();
                pattern.matches_path_with(
                    entry.new_file().path().unwrap(),
                    MatchOptions {
                        case_sensitive: true,
                        require_literal_separator: true,
                        require_literal_leading_dot: true,
                    },
                )
            });
            if matching_config_item.is_none() {
                continue;
            }
            let transformers = matching_config_item
                .unwrap()
                .transformers
                .iter()
                .map(|x| x.transformer(repository_path))
                .collect::<Vec<_>>();

            eprintln!(
                "Transforming staged file: {}",
                entry.new_file().path().unwrap().to_str().unwrap()
            );
            let extension = entry
                .new_file()
                .path()
                .unwrap()
                .extension()
                .and_then(|x| x.to_str());
            let oid = apply_transform_pipeline(
                &repository,
                &repository.find_blob(entry.new_file().id())?,
                &transformers,
                extension,
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
