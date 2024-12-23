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
    /*
     * TODO: support
     *
     * - prettier
     * - ruff
     */
}

impl ShellCommandTransformer {
    pub fn command_str(&self) -> &str {
        match self {
            Self::Rustfmt => "rustfmt",
            Self::ClangFormat => "clang-format",
            Self::System { command, .. } => command.as_str(),
        }
    }

    pub fn configure_command(&self, command: &mut std::process::Command) {
        match self {
            Self::Rustfmt => {
                command.args(["--emit", "stdout"]);
            }
            Self::System { env, args, .. } => {
                command.envs(env);
                command.args(args);
            }
            Self::ClangFormat => {
                /*
                 * clang-format operates with the desired interface out of the
                 * box. No action necessary.
                 */
            }
        }
    }
}
