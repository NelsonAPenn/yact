use git2::{Repository, build::{TreeUpdateBuilder, CheckoutBuilder}, DiffOptions, ApplyLocation};
pub use transformer::{Transformer, transform};

pub mod transformer
{
    use git2::{Blob, Repository, Oid};

    /// A generic trait for transforming staged files.
    ///
    /// Example implementors might be a builtin trailing whitespace transformer,
    /// or shell transformer.
    pub trait Transformer: Fn(&[u8]) -> Result<Vec<u8>, String>{}
    impl<T> super::Transformer for T
        where T: Fn(&[u8]) -> Result<Vec<u8>, String>
    {}

    /// Apply a tronsform to an existing blob, creating another (for example,
    /// applying linting)
    pub fn transform<T>(repository: &Repository, blob: &Blob, transformer: T) -> Result<Oid, crate::Error>
        where T: Transformer
    {
        let transformed = transformer(blob.content())?;
        Ok(repository.blob(transformed.as_slice())?)
    }

    pub mod transformers
    {
        pub fn trailing_whitespace(data: &[u8]) -> Result<Vec<u8>, String>
        {
            let str_data = std::str::from_utf8(data).map_err(|err| format!("{:?}", err))?;
            let mut out = String::with_capacity(data.len());
            for line in str_data.lines()
            {
                out.push_str(line.trim_end());
                out.push('\n');
            }
            Ok(out.into_bytes())
        }
    }

}
#[derive(Debug)]
pub enum Error
{
    GitError(git2::Error),
    TransformerError(String),
}

impl From<git2::Error> for Error
{
    fn from(err: git2::Error) -> Self
    {
        Self::GitError(err)
    }
}

impl From<String> for Error
{
    fn from(err: String) -> Self
    {
        Self::TransformerError(err)
    }
}



pub fn pre_commit() -> Result<(), git2::Error>
{
    let repository = Repository::discover(".")?;
    println!("{:#?}", repository.worktrees()?.iter().collect::<Vec<_>>());
    let mut index = repository.index()?;
    let index_tree_id = index.write_tree()?;
    let index_tree = repository.find_tree(index_tree_id)?;
    eprintln!("Created tree {:?} from index", index_tree);
    let last_committed_tree = repository.head()?.peel_to_tree()?;
    eprintln!("Calculating staged diff...");
    let mut diff = repository.diff_tree_to_tree(Some(&last_committed_tree), Some(&index_tree), None)?;
    diff.find_similar(None)?;
    eprintln!("Transforming files...");
    let mut transformed_tree_builder = TreeUpdateBuilder::new();

    for entry in diff.deltas()
    {
        if !entry.new_file().is_binary()
        {
            eprintln!("Transforming entry {:?}", entry);
            let oid = transform(
                &repository,
                &repository.find_blob(entry.new_file().id())?,
                transformer::transformers::trailing_whitespace
            ).unwrap();
            transformed_tree_builder.upsert(
                entry.new_file().path_bytes().unwrap(),
                oid,
                entry.new_file().mode(),
            );

        }
    }

    let transformed_tree_id = transformed_tree_builder.create_updated(
        &repository,
        &index_tree,
    )?;
    let transformed_tree = repository.find_tree(transformed_tree_id)?;
    eprintln!("Created transformed tree {:?}...", transformed_tree);
    index.read_tree(&transformed_tree)?;
    index.write()?;
    eprintln!("Updated index to new tree!");
    let mut workdir_diff = repository.diff_tree_to_workdir(
        Some(&index_tree),
        None,
    )?;
    println!("staged changes: {:?}", workdir_diff.stats());
    workdir_diff.merge(&repository.diff_tree_to_tree(
        Some(&index_tree),
        Some(&transformed_tree),
        None,
    )?)?;
    println!("with transformations: {:?}", workdir_diff.stats());
    let mut new_dirty_index = repository.apply_to_tree(
        &index_tree,
        &workdir_diff,
        None
    )?;
    repository.checkout_index(
        Some(&mut new_dirty_index),
        Some(&mut CheckoutBuilder::new()
             .safe()
             .update_only(true)
             .use_ours(true)
             .allow_conflicts(true)
             .conflict_style_merge(true)
         ),
    )?;

    eprintln!("updated workdir with changes");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        pre_commit();
    }
}
