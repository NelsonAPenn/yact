/*
 * Copyright 2023, 2024, 2025 Nelson Penn
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
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JavascriptPackageManagerType {
    Npm,
    Yarn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellCommandTransformer {
    Rustfmt,
    Gofmt,
    ClangFormat,
    System {
        command: String,
        env: HashMap<String, String>,
        args: Vec<String>,
    },
    DenoFmt,
    Prettier {
        /// Path to the directory containing package.json, if different from
        /// repository root.
        package_json_directory: Option<PathBuf>,
        package_manager_type: Option<JavascriptPackageManagerType>,
    },
    RuffFormat {
        venv_path: Option<PathBuf>,
    },
}

impl ShellCommandTransformer {
    pub fn get_command<P: AsRef<Path>>(
        &self,
        repository_path: P,
        extension: Option<&str>,
    ) -> Command {
        /*
         * TODO: paths should be relative to repository root, need repository
         * root path to configure command.
         */
        match self {
            Self::Rustfmt => {
                let mut command = Command::new("rustfmt");
                command.args(["--emit", "stdout"]);
                command
            }
            Self::Gofmt => Command::new("gofmt"),
            Self::ClangFormat => {
                let mut command = Command::new("clang-format");
                if let Some(extension) = extension {
                    command.args(["--assume-filename", &format!("example.{}", extension)]);
                }
                command
            }
            Self::System { command, env, args } => {
                let mut command = Command::new(command);
                command.envs(env);
                command.args(args);
                command
            }
            Self::DenoFmt => {
                let mut command = Command::new("deno");
                command.arg("fmt");
                if let Some(extension) = extension {
                    command.args(["--ext", extension]);
                }
                command.arg("-");
                command
            }
            Self::Prettier {
                package_json_directory,
                package_manager_type,
            } => {
                let mut command = match package_manager_type {
                    Some(JavascriptPackageManagerType::Npm) => {
                        let mut command = Command::new("npx");
                        command.arg("prettier");
                        command
                    }
                    Some(JavascriptPackageManagerType::Yarn) => {
                        let mut command = Command::new("yarn");
                        /*
                         * "silent" option is required because otherwise yarn
                         * will print to stdout, ultimately ending up in the
                         * formatted file contents.
                         */
                        command.args(["--silent", "run", "prettier"]);
                        command
                    }
                    None => Command::new("prettier"),
                };
                if let Some(package_json_directory) = package_json_directory {
                    let prettier_path = repository_path.as_ref().join(package_json_directory);
                    command.current_dir(prettier_path);
                }
                if let Some(extension) = extension {
                    command.args(["--stdin-filepath", &format!("example.{}", extension)]);
                }
                command
            }
            Self::RuffFormat { venv_path } => {
                let mut command = match venv_path {
                    Some(venv_path) => {
                        /*
                         * This works even if path is absolute as join will
                         * replace the original path with the absolute one.
                         */
                        let python_path = repository_path
                            .as_ref()
                            .join(venv_path)
                            .join("bin")
                            .join("python");

                        let mut command = Command::new(python_path);
                        command.args(["-m", "ruff"]);
                        command
                    }
                    None => Command::new("ruff"),
                };
                command.arg("format");
                if let Some(extension) = extension {
                    command.args(["--stdin-filename", &format!("example.{}", extension)]);
                }
                command.args(["--quiet", "-"]);
                command
            }
        }
    }
}
