# Aurora AI v0.6.1 — Response Performance Telemetry

- Added local-only rolling telemetry for up to 40 completed chat responses.
- Records time to first streamed output, total response duration, output characters, selected provider/model/profile and context-budget details.
- Added averages and median in the Performance & Runtime view, plus recent response history and a clear-history action.
- Labels throughput as characters/second rather than claiming token throughput; no message content is stored in telemetry.
- Added profile-aware limits for attached web research and project context, with a visible truncation notice.
- Added tests for timing summaries, malformed values, bounded history and external-context limits.

## Notes and limitations
- Measurements cover the renderer-observed request from immediately before chat streaming to completion; they can include provider/network and prompt processing latency.
- Character throughput is only a rough comparison and is not tokens/second. Ollama's existing benchmark remains the source for model token throughput.
- Telemetry is stored locally in the app's localStorage and can be cleared in the runtime view.
- No Windows packaged UI test was run in this release.
