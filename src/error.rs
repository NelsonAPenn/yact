/*
 * Copyright 2023, 2024, 2025, 2026 Nelson Penn
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

use git2::{ErrorClass, ErrorCode};
use semver::{Version, VersionReq};

#[derive(Debug)]
pub enum Error {
    ConfigurationNotFound,
    ConfigurationParseError(toml::de::Error),
    ConfigurationEncodingError(std::str::Utf8Error),
    UnableToDetermineYactPath,
    PreCommitHookAlreadyExists,
    InvalidYactVersion(VersionReq, Version),
    InvalidGlob(String),
    RepositoryNotFound,
    RepositoryIsBare,

    /// An error was returned from `libgit2`.
    GitError(git2::Error),

    /// One of the transformers encountered an error.
    TransformerError(String),

    /// Unexpected std::io::Error
    IoError(std::io::Error),

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
        if matches!(err.class(), ErrorClass::Repository)
            && matches!(err.code(), ErrorCode::NotFound)
        {
            Self::RepositoryNotFound
        } else {
            Self::GitError(err)
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
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
