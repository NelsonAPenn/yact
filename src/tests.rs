use std::path::Path;

use git2::{IndexEntry, Repository, Signature};

use crate::{BuiltinTransformer, Configuration, TransformerOptions};

use super::pre_commit;

const TEST_REPOSITORY_PATH: &str = "/tmp/yact-test";

fn config() -> Configuration<'static>
{
    [
        (
            "*.md",
            vec![TransformerOptions::Builtin(
                BuiltinTransformer::TrailingWhitespace,
            )],
        ),
    ]
    .into_iter()
    .collect()
}

fn fresh_repo() -> Repository
{
    let _ = std::fs::remove_dir_all("/tmp/yact-test");
    std::fs::create_dir("/tmp/yact-test").unwrap();
    let repo = Repository::init("/tmp/yact-test").unwrap();
    let mut index = repo.index().unwrap();
    std::fs::write("/tmp/yact-test/README.md", "# Blah").unwrap();
    index.add_path(std::path::Path::new("README.md")).unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    repo.commit(Some("HEAD"), &repo.signature().unwrap(), &repo.signature().unwrap(), "init", &tree, &[] ).unwrap();
    drop(tree);
    index.write().unwrap();
    std::env::set_current_dir("/tmp/yact-test").unwrap();

    repo
}

#[test]
fn basic_transformation_works()
{
    /*
     * Stage some undesired whitespace changes (and that's it)
     */
    let repo = fresh_repo();
    let mut index = repo.index().unwrap();
    std::fs::write("/tmp/yact-test/README.md", "# Blah     ").unwrap();
    let readme_path = Path::new("README.md");
    index.add_path(&readme_path).unwrap();
    index.write().unwrap();

    pre_commit(&config()).unwrap();
    
    
    /*
     * After pre-commit runs, commit the index
     */
    index.read(true).unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    let oid = repo.commit(Some("HEAD"), &repo.signature().unwrap(), &repo.signature().unwrap(), "init", &tree, &[&repo.head().unwrap().peel_to_commit().unwrap()] ).unwrap();
    index.write().unwrap();
    let committed_id = repo.head().unwrap().peel_to_tree().unwrap().get_path(&readme_path).unwrap().id();
    let object = repo.find_object(committed_id, None).unwrap().into_blob().unwrap();
    assert_eq!(object.content(), b"# Blah\n");

    assert_eq!(std::fs::read("README.md").unwrap(), b"# Blah\n");
}