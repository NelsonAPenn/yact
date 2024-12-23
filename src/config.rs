use crate::TransformerOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationItem {
    pub pathspec: String,
    pub transformers: Vec<TransformerOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub items: Vec<ConfigurationItem>,
}
