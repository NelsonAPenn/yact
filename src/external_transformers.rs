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
