use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum BuiltinTransformer {
    TrailingWhitespace,
}

pub fn trailing_whitespace(data: &[u8]) -> Result<Vec<u8>, String> {
    let str_data = std::str::from_utf8(data).map_err(|err| format!("{:?}", err))?;
    let mut out = String::with_capacity(data.len());
    for line in str_data.lines() {
        out.push_str(line.trim_end());
        out.push('\n');
    }
    Ok(out.into_bytes())
}
