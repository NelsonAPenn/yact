/*
 * Copyright 2026 Nelson Penn
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

use crate::Error;
use git2::Repository;
use std::path::{Path, PathBuf};
use which::which;

fn get_absolute_executable_path() -> Result<PathBuf, Error> {
    let arg0 = std::env::args().next().unwrap();
    let path = std::path::PathBuf::from(arg0.clone());

    if let Some(name) = path.file_name()
        && name.to_str() == Some(arg0.as_str())
    {
        which(&arg0).map_err(|_| Error::UnableToDetermineYactPath)
    } else if path.is_absolute() {
        Ok(path)
    } else if path.is_relative() {
        Ok(path.canonicalize()?)
    } else {
        Err(Error::UnableToDetermineYactPath)
    }
}

pub fn init<P: AsRef<Path>>(path: P, force: bool) -> Result<(), Error> {
    let repository = Repository::discover(path)?;
    let repository_path = repository.workdir().ok_or(Error::RepositoryIsBare)?;

    let hook_path = repository_path
        .join(".git")
        .join("hooks")
        .join("pre-commit");

    if hook_path.exists() {
        if force {
            std::fs::remove_file(&hook_path)?;
        } else {
            return Err(Error::PreCommitHookAlreadyExists);
        }
    }
    let executable_path = get_absolute_executable_path()?;

    #[cfg(target_family = "unix")]
    std::os::unix::fs::symlink(executable_path, hook_path)?;

    #[cfg(not(target_family = "unix"))]
    std::fs::hard_link(executable_path, hook_path)?;

    Ok(())
}
