mod code_reader;
mod llm;
mod scanner;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

use crate::code_reader::CodeReader;
use crate::llm::{LlmConfig, analyze_with_llm};

#[derive(Debug, Parser)]
#[command(name = "code-cohesion")]
#[command(about = "Find modules that mix too many responsibilities.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Scan(ScanArgs),
}

#[derive(Debug, Parser)]
struct ScanArgs {
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,

    #[arg(long)]
    llm: bool,

    #[arg(long, env = "OPENAI_MODEL", default_value = "gpt-4.1-mini")]
    model: String,

    #[arg(
        long,
        env = "OPENAI_BASE_URL",
        default_value = "https://api.openai.com/v1"
    )]
    base_url: String,

    #[arg(long, env = "OPENAI_API_KEY")]
    api_key: Option<String>,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Markdown,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Scan(args) => scan(args).await,
    }
}

async fn scan(args: ScanArgs) -> Result<()> {
    let report = scanner::scan(&args.path)?;

    if args.llm {
        let api_key = args
            .api_key
            .context("--llm requires OPENAI_API_KEY or --api-key")?;
        let reader = CodeReader::new(&args.path)?;
        let advice = analyze_with_llm(
            &report,
            &reader,
            LlmConfig {
                api_key,
                base_url: args.base_url,
                model: args.model,
            },
        )
        .await?;
        println!("{advice}");
        return Ok(());
    }

    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        OutputFormat::Markdown => print_markdown(&report),
    }

    Ok(())
}

fn print_markdown(report: &scanner::ScanReport) {
    println!("# Code Cohesion Scan\n");
    println!("- Root: `{}`", report.root);
    println!("- Files scanned: `{}`\n", report.files_scanned);

    for finding in report
        .findings
        .iter()
        .filter(|finding| finding.suspicion != scanner::Suspicion::Low)
        .take(20)
    {
        println!("## `{}`", finding.path);
        println!();
        println!("- Suspicion: `{:?}`", finding.suspicion);
        println!("- Lines: `{}`", finding.lines);
        println!("- Roles: `{:?}`", finding.likely_roles);
        if !finding.reasons.is_empty() {
            println!("- Reasons: {}", finding.reasons.join("; "));
        }
        println!();
    }
}
