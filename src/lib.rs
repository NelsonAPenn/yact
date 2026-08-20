/*
 * Copyright 2023, 2024, 2026 Nelson Penn
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

//! `yact` is a program that applies code formatters seamlessly to git commits
//! when installed as a pre-commit hook. This is the backing library for the
//! binary sharing the same name.
//!
//! - [pre_commit] contains the implementation of the core algorithm of the
//!   project.
//! - [Configuration] is the configuration struct which is conventionally stored
//!   as `yactrc.toml`.
//! - [BuiltinTransformer] is an enum containing the builtin transformers that
//!   can be used in the [Configuration].
//! - [ShellCommandTransformer] is an enum containing the external transformers
//!   that can be used in the [Configuration]. This serves as a helpful
//!   reference of all the available options.
mod builtin_transformers;
mod config;
mod error;
mod external_transformers;
mod init;
mod pre_commit;
#[cfg(test)]
mod tests;
mod transformer;
pub use init::init;
pub use pre_commit::pre_commit;

pub use builtin_transformers::BuiltinTransformer;
pub use config::{Configuration, ConfigurationItem, TransformerOptions, load_configuration};
pub use error::Error;
pub use external_transformers::{
    JavascriptPackageManagerType, RuffLintBehavior, ShellCommandTransformer,
};
pub use transformer::{Transformer, apply_transform_pipeline, create_shell_transformer, transform};
