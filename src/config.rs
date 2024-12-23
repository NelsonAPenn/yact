use crate::{
    builtin_transformers, create_shell_transformer, BuiltinTransformer, ShellCommandTransformer,
    Transformer,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum TransformerOptions {
    Builtin(BuiltinTransformer),
    RawCommand(ShellCommandTransformer),
}

impl TransformerOptions {
    pub fn transformer(&self) -> Box<dyn Transformer> {
        match self {
            Self::Builtin(BuiltinTransformer::TrailingWhitespace) => {
                Box::new(builtin_transformers::trailing_whitespace)
            }
            Self::RawCommand(command_type) => {
                let command_type = command_type.clone();
                Box::new(create_shell_transformer(move || {
                    let mut command = std::process::Command::new(command_type.command_str());
                    command_type.configure_command(&mut command);
                    command
                }))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationItem {
    pub pathspec: String,
    pub transformers: Vec<TransformerOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub items: Vec<ConfigurationItem>,
}
