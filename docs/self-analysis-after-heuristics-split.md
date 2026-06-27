# Self-Analysis After Heuristics Split

This report was generated after splitting the scanner heuristic engine into
smaller modules.

- Model: `gemma4:31b`
- Provider: Ollama Cloud, OpenAI-compatible API
- Command shape: `code-cohesion scan . --llm`

## Summary

The static scan flagged `src/scanner/role_mapper.rs`,
`src/scanner/extractors.rs`, and `src/llm/client.rs`.

The LLM review judged these findings as mostly false positives:

- `role_mapper.rs` is healthy. It contains the mapping rules for roles; it is
  not itself performing all those roles.
- `extractors.rs` is healthy. It handles string extraction and basic text
  cleanup.
- `llm/client.rs` is acceptable. `LlmConfig` and `LlmClient` are tightly coupled
  and still small.

## Lesson

This is a useful failure mode for the project:

```text
the scanner can mistake analysis vocabulary for actual responsibility mixing
```

That is exactly why `code-cohesion` should treat static heuristics as attention
triggers, not final judgment.

The next improvement should make role inference aware of meta-code: files whose
purpose is to define detectors, roles, rules, or test fixtures should not be
penalized merely because they contain the words they detect.
