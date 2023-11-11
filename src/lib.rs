use git2::{Repository, build::TreeUpdateBuilder};
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
    let mut index = repository.index()?;
    let index_tree_id = index.write_tree()?;
    let index_tree = repository.find_tree(index_tree_id)?;
    eprintln!("Created tree {:?} from index", index_tree);
    let last_committed_tree = repository.head()?.peel_to_tree()?;
    eprintln!("Calculating staged diff...");
    let mut diff = repository.diff_tree_to_tree(Some(&last_committed_tree), Some(&index_tree), None)?;
    diff.find_similar(None);
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

    eprintln!("Updated index to new tree!");
    index.write()?;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        pre_commit();
    }
}
