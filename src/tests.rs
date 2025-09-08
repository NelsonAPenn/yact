/*
 * Copyright 2023, 2024 Nelson Penn
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
use git2::Repository;
use std::{
    fs::Permissions,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{BuiltinTransformer, Configuration, ConfigurationItem, TransformerOptions};

use super::pre_commit;

fn config() -> Configuration {
    Configuration {
        requires_yact_version: None,
        items: vec![ConfigurationItem {
            glob: "*.md".to_string(),
            transformers: vec![TransformerOptions::Builtin(
                BuiltinTransformer::TrailingWhitespace,
            )],
        }],
    }
}

fn fresh_repo() -> (Repository, PathBuf) {
    let uuid = uuid::Uuid::new_v4().to_string();
    let repo_path = PathBuf::from_str(format!("/tmp/yact-test-{uuid}").as_str()).unwrap();

    let _ = std::fs::remove_dir_all(&repo_path);
    std::fs::create_dir(&repo_path).unwrap();
    let repo = Repository::init(&repo_path).unwrap();
    let mut index = repo.index().unwrap();
    std::fs::write(repo_path.join("README.md"), "# Blah\n").unwrap();
    index.add_path(std::path::Path::new("README.md")).unwrap();
    std::fs::write(
        repo_path.join("yactrc.toml"),
        toml::to_string_pretty(&config()).unwrap(),
    )
    .unwrap();
    index.add_path(std::path::Path::new("yactrc.toml")).unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    repo.commit(
        Some("HEAD"),
        &repo.signature().unwrap(),
        &repo.signature().unwrap(),
        "init",
        &tree,
        &[],
    )
    .unwrap();
    drop(tree);
    index.write().unwrap();

    (repo, repo_path)
}

#[test]
fn basic_operation_works() {
    /*
     * Stage some undesired whitespace changes (and that's it)
     */
    let (repo, repo_path) = fresh_repo();
    let mut index = repo.index().unwrap();
    std::fs::write(repo_path.join("README.md"), "# Blah     ").unwrap();
    let readme_path = Path::new("README.md");
    index.add_path(readme_path).unwrap();
    index.write().unwrap();

    let pre_commit_result = pre_commit(repo_path.as_path(), None);
    assert!(matches!(pre_commit_result, Err(crate::Error::EmptyIndex)));
    /*
     * After pre-commit runs, commit the index
     */
    index.read(true).unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    repo.commit(
        Some("HEAD"),
        &repo.signature().unwrap(),
        &repo.signature().unwrap(),
        "init",
        &tree,
        &[&repo.head().unwrap().peel_to_commit().unwrap()],
    )
    .unwrap();
    index.write().unwrap();
    let committed_id = repo
        .head()
        .unwrap()
        .peel_to_tree()
        .unwrap()
        .get_path(readme_path)
        .unwrap()
        .id();
    let object = repo
        .find_object(committed_id, None)
        .unwrap()
        .into_blob()
        .unwrap();
    assert_eq!(object.content(), b"# Blah\n");

    assert_eq!(
        std::fs::read(repo_path.join("README.md")).unwrap(),
        b"# Blah\n"
    );
    std::fs::remove_dir_all(repo_path).unwrap();
}

#[test]
fn conflict_handled_correctly() {
    /*
     * Stage some undesired whitespace changes (and that's it)
     */
    let (repo, repo_path) = fresh_repo();
    let mut index = repo.index().unwrap();
    std::fs::write(repo_path.join("README.md"), "# Blah     ").unwrap();
    let readme_path = Path::new("README.md");
    index.add_path(readme_path).unwrap();
    index.write().unwrap();
    /*
     * Introduce a conflict between working tree and staged changes
     */
    std::fs::write(repo_path.join("README.md"), "# Blab").unwrap();

    let pre_commit_result = pre_commit(repo_path.as_path(), None);
    assert!(matches!(pre_commit_result, Err(crate::Error::EmptyIndex)));

    /*
     * After pre-commit runs, commit the index
     */
    index.read(true).unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    repo.commit(
        Some("HEAD"),
        &repo.signature().unwrap(),
        &repo.signature().unwrap(),
        "init",
        &tree,
        &[&repo.head().unwrap().peel_to_commit().unwrap()],
    )
    .unwrap();
    index.write().unwrap();
    let committed_id = repo
        .head()
        .unwrap()
        .peel_to_tree()
        .unwrap()
        .get_path(readme_path)
        .unwrap()
        .id();
    let object = repo
        .find_object(committed_id, None)
        .unwrap()
        .into_blob()
        .unwrap();
    assert_eq!(object.content(), b"# Blah\n");

    assert_eq!(
        std::fs::read(repo_path.join("README.md")).unwrap(),
        b"# Blab"
    );
    std::fs::remove_dir_all(repo_path).unwrap();
}

/// Test that executable files are not deleted by yact
#[cfg(target_family = "unix")]
#[test]
fn executable_files_not_deleted() {
    let (repo, repo_path) = fresh_repo();
    let mut index = repo.index().unwrap();
    let mut permissions = repo_path
        .join("README.md")
        .metadata()
        .unwrap()
        .permissions();
    permissions.set_mode(0o770);
    std::fs::set_permissions(repo_path.join("README.md"), permissions).unwrap();
    let readme_path = Path::new("README.md");
    index.add_path(readme_path).unwrap();
    index.write().unwrap();

    let _ = pre_commit(repo_path.as_path(), None);

    assert!(repo_path.join("README.md").exists());
}
