# code-cohesion

`code-cohesion` is an experimental CLI for detecting source files that may mix
too many responsibilities.

It is not a linter, a quality score, or a small clone of SonarQube. The core
idea is narrower:

> Metrics should not pretend to be judgment. Metrics should open an inquiry.

Large files, broad imports, churn, and mixed symbols are useful signals, but
they are not the diagnosis. `code-cohesion` uses them as a radar, then asks an
LLM to inspect the suspicious parts of the code through constrained tools and
return concrete refactoring advice.

The project is built for human and AI coding workflows where the main risk is
not "bad style" in the abstract, but quiet structural drift:

- UI code absorbing domain logic;
- route handlers becoming services;
- persistence leaking into presentation;
- one module becoming the default place where every new feature lands;
- coding agents extending already-fragile files because no guardrail says stop.

The static scan is only a radar. The interesting path is:

1. build a compact structural report from source files;
2. let an LLM inspect suspicious files through constrained read tools;
3. return concrete split advice instead of a vague quality score.

In other words:

```text
static metrics -> attention trigger
tool-guided LLM inspection -> semantic diagnosis
recommendation -> concrete split or explicit justification
```

## Status

Very early prototype.

This project started as a small vibe-coded experiment. That is part of the
point: it tries to make AI-assisted code growth inspectable, reviewable, and
able to criticize its own structure before it quietly turns into a pile.

Current capabilities:

- scan local source files;
- parse supported source files with tree-sitter;
- emit a structured JSON report;
- emit a compact Markdown report;
- call an OpenAI-compatible LLM;
- expose `list_files` and `read_file` tools to the model;
- require the LLM to inspect at least one source file before producing final
  advice.

The current AST support is still early: it extracts structural symbols with
tree-sitter, while role inference remains heuristic. The next milestone is to
derive richer signals from the AST and dependency graph instead of relying on
keyword matches.

## Language Support

`code-cohesion` is intentionally conservative about language support.

Currently supported:

- Rust
- TypeScript
- TSX
- JavaScript
- JSX
- Python
- Kotlin

Planned:

- More language-specific AST signals for the supported languages

Unsupported files are skipped for now. That is deliberate: a shallow,
overconfident analysis across every extension would be worse than a narrower
analysis that knows what it can parse.

## Usage

```bash
cargo run -- scan path/to/repo --format markdown
```

JSON output is the default:

```bash
cargo run -- scan path/to/repo
```

LLM-assisted scan:

```bash
OPENAI_API_KEY=... cargo run -- scan path/to/repo --llm
```

The LLM client is OpenAI-compatible:

```bash
OPENAI_API_KEY=... \
OPENAI_BASE_URL=https://api.openai.com/v1 \
OPENAI_MODEL=gpt-4.1-mini \
cargo run -- scan path/to/repo --llm
```

The model receives two tools:

- `list_files`: list files under the scan root;
- `read_file`: read one file relative to the scan root, capped at 32 KiB.

Reads are constrained to the scan root.

## Why This Exists

Most code health tools are good at identifying local issues: complexity,
duplication, formatting, unsafe patterns, missing tests. Fewer tools are good at
answering a more architectural question:

> Does this file still represent one coherent responsibility?

That question is fuzzy enough that pure metrics are brittle, but structured
enough that an LLM can help when it receives the right context.

`code-cohesion` tries to give the model a useful map instead of dumping a whole
repository into a prompt. The model can then ask to read the files it needs.

## Self-Improvement Loop

This repository is meant to be its own first case study.

The intended workflow is:

1. run `code-cohesion` on `code-cohesion`;
2. let the LLM identify mixed responsibilities;
3. apply the recommended split;
4. run the scan again;
5. keep the improvement visible in the commit history.

That makes the project a small experiment in tool-assisted refactoring, not just
a CLI that comments on other people's code.

The first self-analysis report is committed in
[`docs/self-analysis-2026-06-27.md`](docs/self-analysis-2026-06-27.md).
