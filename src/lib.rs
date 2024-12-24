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
mod builtin_transformers;
mod config;
mod error;
mod external_transformers;
mod pre_commit;
#[cfg(test)]
mod tests;
mod transformer;
pub use pre_commit::pre_commit;

pub use builtin_transformers::BuiltinTransformer;
pub use config::{load_configuration, Configuration, ConfigurationItem, TransformerOptions};
pub use error::Error;
pub use external_transformers::ShellCommandTransformer;
pub use transformer::{apply_transform_pipeline, create_shell_transformer, transform, Transformer};
