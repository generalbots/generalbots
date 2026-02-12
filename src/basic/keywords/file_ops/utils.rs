/*****************************************************************************\
|  █████  █████ ██    █ █████ █████   ████  ██      ████   █████ █████  ███ ® |
| ██      █     ███   █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █   █      |
| ██  ███ ████  █ ██  █ ████  █████  ██████ ██      ████   █   █   █    ██    |
| ██   ██ █     █  ██ █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █      █   |
|  █████  █████ █   ███ █████ ██  ██ ██  ██ █████   ████   █████   █   ███    |
|                                                                             |
| General Bots Copyright (c) pragmatismo.com.br. All rights reserved.         |
| Licensed under the AGPL-3.0.                                                |
|                                                                             |
| According to our dual licensing model, this program can be used either      |
| under the terms of the GNU Affero General Public License, version 3,        |
| or under a proprietary license.                                             |
|                                                                             |
| The texts of the GNU Affero General Public License with an additional       |
| permission and of our proprietary license can be found at and               |
| in the LICENSE file you have received along with this program.              |
|                                                                             |
| This program is distributed in the hope that it will be useful,             |
| but WITHOUT ANY WARRANTY, without even the implied warranty of              |
| MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the                |
| GNU Affero General Public License for more details.                         |
|                                                                             |
| "General Bots" is a registered trademark of pragmatismo.com.br.             |
| The licensing of the program under the AGPLv3 does not imply a              |
| trademark license. Therefore any rights, title and interest in              |
| our trademarks remain entirely with us.                                     |
|                                                                             |
\*****************************************************************************/

use rhai::{Dynamic, Map};
use serde_json::Value;

use super::transfer::FileData;

pub fn dynamic_to_json(value: &Dynamic) -> Value {
    if value.is_unit() {
        Value::Null
    } else if value.is_bool() {
        Value::Bool(value.as_bool().unwrap_or(false))
    } else if value.is_int() {
        Value::Number(value.as_int().unwrap_or(0).into())
    } else if value.is_float() {
        if let Ok(f) = value.as_float() {
            serde_json::Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        } else {
            Value::Null
        }
    } else if value.is_string() {
        Value::String(value.to_string())
    } else if value.is_array() {
        let arr = value.clone().into_array().unwrap_or_default();
        Value::Array(arr.iter().map(dynamic_to_json).collect())
    } else if value.is_map() {
        let map = value.clone().try_cast::<Map>().unwrap_or_default();
        let obj: serde_json::Map<String, Value> = map
            .iter()
            .map(|(k, v)| (k.to_string(), dynamic_to_json(v)))
            .collect();
        Value::Object(obj)
    } else {
        Value::String(value.to_string())
    }
}

pub fn dynamic_to_file_data(value: &Dynamic) -> FileData {
    if value.is_map() {
        let map = value.clone().try_cast::<Map>().unwrap_or_default();
        let content = map
            .get("data")
            .map(|v| v.to_string().into_bytes())
            .unwrap_or_default();
        let filename = map
            .get("filename")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "file".to_string());

        FileData { content, filename }
    } else {
        FileData {
            content: value.to_string().into_bytes(),
            filename: "file".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Dynamic;
    use serde_json::Value;

    #[test]
    fn test_dynamic_to_json() {
        let dynamic = Dynamic::from("hello");
        let json = dynamic_to_json(&dynamic);
        assert_eq!(json, Value::String("hello".to_string()));
    }

    #[test]
    fn test_dynamic_to_file_data() {
        let dynamic = Dynamic::from("test content");
        let file_data = dynamic_to_file_data(&dynamic);
        assert_eq!(file_data.filename, "file");
        assert!(!file_data.content.is_empty());
    }
}
