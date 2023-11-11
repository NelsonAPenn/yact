use git2::{Repository};
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
    let index = repository.index()?;
    let tree = repository.head()?.peel_to_tree()?;
    let mut diff = repository.diff_tree_to_index(Some(&tree), Some(&index), None)?;
    diff.find_similar(None);

    for entry in diff.deltas()
    {
        println!("{:?}", entry);
        if !entry.new_file().is_binary()
        {
            transform(
                &repository,
                &repository.find_blob(entry.new_file().id())?,
                transformer::transformers::trailing_whitespace
            ).unwrap();

        }
        // println!("{:?}", str::from_utf8(entry.path.as_slice()));

    }
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
