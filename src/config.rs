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
use crate::{
    builtin_transformers, create_shell_transformer, BuiltinTransformer, Error,
    ShellCommandTransformer, Transformer,
};
use git2::Repository;
use glob::Pattern;
use semver::VersionReq;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum TransformerOptions {
    Builtin(BuiltinTransformer),
    External(ShellCommandTransformer),
}

impl TransformerOptions {
    pub fn transformer(&self) -> Box<dyn Transformer> {
        match self {
            Self::Builtin(BuiltinTransformer::TrailingWhitespace) => {
                Box::new(builtin_transformers::trailing_whitespace)
            }
            Self::External(command_type) => {
                let command_type = command_type.clone();
                Box::new(create_shell_transformer(move |extension: Option<&str>| {
                    let mut command = std::process::Command::new(command_type.command_str());
                    command_type.configure_command(&mut command, extension);
                    command
                }))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationItem {
    pub glob: String,
    pub transformers: Vec<TransformerOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub requires_yact_version: Option<VersionReq>,
    pub items: Vec<ConfigurationItem>,
}

pub fn load_configuration(repository: &Repository) -> Result<Configuration, Error> {
    let path = repository.workdir().ok_or(Error::RepositoryIsBare)?;

    let file = std::fs::read(path.join("yactrc.toml")).map_err(|_| Error::ConfigurationNotFound)?;
    let config_str = std::str::from_utf8(&file)?;
    let configuration: Configuration = toml::from_str(config_str)?;
    for item in &configuration.items {
        if Pattern::new(&item.glob).is_err() {
            return Err(Error::InvalidGlob(item.glob.clone()));
        }
    }

    Ok(configuration)
}
