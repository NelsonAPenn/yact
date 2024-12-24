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

#[derive(Debug, Serialize, Deserialize)]
pub enum BuiltinTransformer {
    TrailingWhitespace,
}

pub fn trailing_whitespace(data: &[u8], _extension: Option<&str>) -> Result<Vec<u8>, String> {
    let str_data = std::str::from_utf8(data).map_err(|err| format!("{:?}", err))?;
    let mut out = String::with_capacity(data.len());
    for line in str_data.lines() {
        out.push_str(line.trim_end());
        out.push('\n');
    }
    Ok(out.into_bytes())
}
