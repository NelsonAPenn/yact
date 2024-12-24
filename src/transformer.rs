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
use git2::{Blob, Oid, Repository};
use std::io::Write;
use std::process::Stdio;

/// A generic trait for transforming staged files.
///
/// Example implementors might be a builtin trailing whitespace transformer,
/// or shell transformer.
pub trait Transformer: Fn(&[u8], Option<&str>) -> Result<Vec<u8>, String> {}
impl<T> super::Transformer for T where T: Fn(&[u8], Option<&str>) -> Result<Vec<u8>, String> {}

/// Apply a transform to an existing blob, creating another (for example,
/// applying linting)
pub fn transform<T>(
    repository: &Repository,
    blob: &Blob,
    transformer: T,
    extension: Option<&str>,
) -> Result<Oid, crate::Error>
where
    T: Transformer,
{
    let transformed = transformer(blob.content(), extension)?;
    Ok(repository.blob(transformed.as_slice())?)
}

/// Apply many transform to an existing blob, creating another (for example,
/// applying linting)
pub fn apply_transform_pipeline(
    repository: &Repository,
    blob: &Blob,
    transformers: &[Box<dyn Transformer>],
    extension: Option<&str>,
) -> Result<Oid, crate::Error> {
    if transformers.is_empty() {
        Ok(blob.id())
    } else {
        let mut transformer_iter = transformers.iter();
        let mut transformed = transformer_iter.next().unwrap()(blob.content(), extension)?;
        for transformer in transformer_iter {
            transformed = transformer(transformed.as_slice(), extension)?;
        }

        Ok(repository.blob(transformed.as_slice())?)
    }
}

/// create a shell transformer from a command with process and arguments
/// configured.
pub fn create_shell_transformer<T: Fn(Option<&str>) -> std::process::Command>(
    command_getter: T,
) -> impl Transformer {
    move |data: &[u8], extension: Option<&str>| {
        let mut child = command_getter(extension)
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
