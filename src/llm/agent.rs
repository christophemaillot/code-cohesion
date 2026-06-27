use anyhow::{Context, Result, bail};

use crate::code_reader::CodeReader;
use crate::scanner::ScanReport;

use super::client::LlmClient;
use super::tools::{run_tool, tools_schema};
use super::types::{LlmConfig, Message};

pub async fn analyze_with_llm(
    report: &ScanReport,
    reader: &CodeReader,
    config: LlmConfig,
) -> Result<String> {
    let client = LlmClient::new(config);
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
        let assistant = client.complete(&messages, tools_schema()).await?;
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
                serde_json::json!({
                    "error": error.to_string(),
                })
            });
            messages.push(Message::tool(call.id, output.to_string()));
        }
    }

    bail!("LLM did not finish within tool-call budget");
}
