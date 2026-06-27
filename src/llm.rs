use anyhow::{Context, Result, bail};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::code_reader::CodeReader;
use crate::scanner::ScanReport;

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

pub async fn analyze_with_llm(
    report: &ScanReport,
    reader: &CodeReader,
    config: LlmConfig,
) -> Result<String> {
    let client = reqwest::Client::new();
    let endpoint = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));

    let report_json = serde_json::to_string_pretty(report)?;
    let mut saw_read_file = false;
    let mut messages = vec![
        Message::system(
            "You are code-cohesion, a structural code cohesion reviewer. \
             Use the provided tools to inspect the repository before giving advice. \
             Focus on mixed responsibilities, module boundaries, and concrete split recommendations. \
             Do not invent files you did not inspect.",
        ),
        Message::user(format!(
            "Here is the static scan report. Inspect the most suspicious files with tools, then produce a concise Markdown report.\n\n```json\n{report_json}\n```"
        )),
    ];

    for _ in 0..6 {
        let body = json!({
            "model": config.model,
            "messages": messages,
            "tools": tools_schema(),
            "tool_choice": "auto",
        });

        let response: ChatResponse = client
            .post(&endpoint)
            .header(AUTHORIZATION, format!("Bearer {}", config.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .context("LLM request failed")?
            .error_for_status()
            .context("LLM returned an error status")?
            .json()
            .await
            .context("cannot decode LLM response")?;

        let Some(choice) = response.choices.into_iter().next() else {
            bail!("LLM returned no choices");
        };

        let assistant = choice.message;
        let tool_calls = assistant.tool_calls.clone().unwrap_or_default();
        messages.push(assistant);

        if tool_calls.is_empty() {
            if !saw_read_file {
                messages.push(Message::user(
                    "Before the final report, call read_file on at least one suspicious source file.",
                ));
                continue;
            }
            return messages
                .last()
                .and_then(|message| message.content.clone())
                .context("LLM finished without content");
        }

        for call in tool_calls {
            if call.function.name == "read_file" {
                saw_read_file = true;
            }
            let output = run_tool(reader, &call).unwrap_or_else(|error| {
                json!({
                    "error": error.to_string(),
                })
            });
            messages.push(Message::tool(call.id, output.to_string()));
        }
    }

    bail!("LLM did not finish within tool-call budget");
}

fn run_tool(reader: &CodeReader, call: &ToolCall) -> Result<Value> {
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

fn tools_schema() -> Value {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

impl Message {
    fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    fn tool(tool_call_id: String, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolCall {
    id: String,
    function: ToolFunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolFunctionCall {
    name: String,
    arguments: String,
}
