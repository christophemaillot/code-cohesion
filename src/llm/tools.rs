use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use crate::code_reader::CodeReader;

use super::types::ToolCall;

pub(crate) fn run_tool(reader: &CodeReader, call: &ToolCall) -> Result<Value> {
    let args: Value = serde_json::from_str(&call.function.arguments)
        .with_context(|| format!("invalid tool arguments for {}", call.function.name))?;

    match call.function.name.as_str() {
        "list_files" => {
            let max_files = args
                .get("max_files")
                .and_then(Value::as_u64)
                .unwrap_or(200)
                .min(1000) as usize;
            Ok(json!({ "files": reader.list_files(max_files) }))
        }
        "read_file" => {
            let path = args
                .get("path")
                .and_then(Value::as_str)
                .context("read_file requires a string path")?;
            Ok(json!({
                "path": path,
                "content": reader.read_file(path)?,
            }))
        }
        other => bail!("unknown tool: {other}"),
    }
}

pub(crate) fn tools_schema() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": "list_files",
                "description": "List source files under the scan root.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "max_files": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 1000
                        }
                    }
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "Read one file relative to the scan root. Output may be truncated.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative file path from the scan root."
                        }
                    },
                    "required": ["path"]
                }
            }
        }
    ])
}
