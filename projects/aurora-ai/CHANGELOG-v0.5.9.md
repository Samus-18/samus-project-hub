# Aurora AI v0.5.9 — Adaptive Performance

- Added performance profiles: Adaptive, Quick, Balanced, Deep and Manual.
- Adaptive mode classifies the prompt and selects a bounded context/output budget for simple versus complex tasks.
- Manual mode preserves the user's saved generation settings; profile choice persists across restarts.
- CPU thread selection is bounded by detected core count when available.
- The assistant labels the profile used in each completed answer and shows the effective budget in response settings.
- Added unit tests for classification, deterministic presets, manual bounds and CPU thread limits.

## Limitations
- Performance profiles configure request budgets; they do not change model weights or guarantee faster tokens/second.
- Ollama honors context/output/thread options. The current OpenAI-compatible adapter maps temperature and maximum output tokens; remote providers may ignore local-only context/thread controls.
- No live Ollama hardware benchmark or packaged Windows UI test was run for this release.
