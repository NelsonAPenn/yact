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
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellCommandTransformer {
    Rustfmt,
    ClangFormat,
    System {
        command: String,
        env: HashMap<String, String>,
        args: Vec<String>,
    },
    DenoFmt,
    Prettier,
    RuffFormat,
}

impl ShellCommandTransformer {
    pub fn command_str(&self) -> &str {
        match self {
            Self::Rustfmt => "rustfmt",
            Self::ClangFormat => "clang-format",
            Self::System { command, .. } => command.as_str(),
            Self::DenoFmt => "deno",
            Self::Prettier => "prettier",
            Self::RuffFormat => "ruff",
        }
    }

    pub fn configure_command(&self, command: &mut std::process::Command, extension: Option<&str>) {
        match self {
            Self::Rustfmt => {
                command.args(["--emit", "stdout"]);
            }
            Self::System { env, args, .. } => {
                command.envs(env);
                command.args(args);
            }
            Self::ClangFormat => {
                if let Some(extension) = extension {
                    command.args(["--assume-filename", &format!("example.{}", extension)]);
                }
            }
            Self::DenoFmt => {
                command.arg("fmt");
                if let Some(extension) = extension {
                    command.args(["--ext", extension]);
                }
                command.arg("-");
            }
            Self::Prettier => {
                if let Some(extension) = extension {
                    command.args(["--stdin-filepath", &format!("example.{}", extension)]);
                }
            }
            Self::RuffFormat => {
                command.arg("format");
                if let Some(extension) = extension {
                    command.args(["--stdin-filename", &format!("example.{}", extension)]);
                }
                command.args(["--quiet", "-"]);
            }
        }
    }
}
