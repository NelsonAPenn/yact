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
    transformers: &[Box<dyn Transformer>],
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
        let clone = data.to_vec();
        std::thread::spawn(move || {
            stdin
                .write_all(clone.as_slice())
                .expect("Failed to write to stream");
        });
        let output = child
            .wait_with_output()
            .map_err(|_| "Failed to wait on transformer process")?;

        if !output.status.success() {
            return Err("Transformer process produced nonzero exit code.".to_string());
        }
        Ok(output.stdout)
    }
}
