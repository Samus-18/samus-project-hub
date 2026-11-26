# Video test review — Aurora AI v0.4.1

Source reviewed: `Discord_8SnWDn4C7t.mp4` (about 110 seconds, 1426×860, 30 FPS).

## What the video confirms

- Around 0–21 s: the app opens, Ollama reports online with five models, and Czech chat messages are sent.
- Around 24–36 s: Agent Studio opens. The screen correctly states that command execution is not enabled and that a project folder must be selected before project-scoped work.
- Around 39–57 s: Model Hub and runtime pages open; the UI detects local models and shows approximate system memory.
- Around 60–61 s: the tools/integrations page displays service states and navigation to the relevant features.
- Around 63–78 s: a search is submitted, but the results area reports no results for the tested query. This is the clearest visible failure in the recording; it could be a brittle result parser, a search-provider response change, network filtering, or a query with no matches.
- Around 80–84 s: Image Lab reports that ComfyUI is not available at `127.0.0.1:8188`. This is a truthful dependency status, not proof that image generation is broken when ComfyUI is installed and running.
- Around 87–90 s: Projects shows a “Unity Companion” card describing the integration as future work even though static audit and log diagnostics already exist elsewhere in the UI. This wording is misleading.
- Around 93–102 s: Memory correctly says semantic long-term memory is not active; local conversation history is separate.
- Around 105–109 s: settings confirms Ollama connection and five models.

## Changes made after reviewing the video

1. Internet search now parses result anchors independently rather than relying on a brittle nested `<div>` expression, and tries DuckDuckGo's regular HTML endpoint followed by its lite endpoint.
2. Empty search results now display a diagnostic explaining whether the provider failed or returned markup the parser could not recognize, instead of only telling the user to try a different query.
3. The Projects screen now labels the existing Unity feature as “Unity diagnostics” and clearly lists the implemented static audit/log/export features and the limits (no Unity compilation or Editor launch).
4. Memory wording now distinguishes local conversation history from semantic long-term memory.
5. Agent Studio's introduction now emphasizes planning and approval-gated changes rather than implying unrestricted autonomous execution.

## Still requires live validation

This video alone cannot establish that a search result parser works against current live provider responses, that a selected web page can always be extracted, or that ComfyUI generation succeeds. Re-test search with multiple common queries while online; run ComfyUI before testing Image Lab; select a disposable Unity project to test audit and approved file edits. Do not use a valuable project for the first write test.

## Follow-up build v0.4.4

The video review's clearest workflow friction is now addressed in source: the main navigation no longer dedicates a primary slot to Image Lab. A natural-language image request routes directly to local ComfyUI, and generated images appear inside the chat. The separate visual-engine screen remains available from Tools / a guided chat action only for first-time workflow setup. The selected workflow path is stored under Electron userData and revalidated on the next launch.

Explicit web-research requests can now search, fetch up to three selected public HTTPS results, and ask the local model to compare them with citations. The main chat discloses that the search query goes to the search provider only when web research is requested. The UI gains animated holographic scan lines, orbiting rings, light sweeps and cyan/violet glass panels while respecting reduced-motion preferences.

A duplicate `web:open-result` IPC registration found during the source review was removed. Static validation verifies all IPC names are unique. v0.4.4 has 32 passing automated tests and passes `npm run check`; live ComfyUI and Windows Electron launch validation remain outstanding in this environment.


## v0.4.7 GUI layout pass

- Updated default/minimum Electron window size to 1360×860 / 900×600.
- Added responsive layout tokens for sidebar rail and content width.
- Rebalanced chat transcript and composer to a shared 1040px desktop measure.
- Changed welcome action cards to a centered 2×2 desktop layout, with narrower and short-height adaptations.
- Added condensed sidebar for narrower widths and stronger keyboard focus indicators.
- Automated tests verify responsive breakpoints, reduced-motion support and window metadata.
- Not yet visually verified in a live Windows Electron window.


## v0.4.8 GUI pass
- Assistant messages use a calmer reading column and clearer heading/paragraph rhythm.
- Tables and code blocks scroll within the message instead of expanding the whole chat width.
- Images are constrained to the conversation viewport.
- User message width and composer width are coordinated across desktop and narrow layouts.
- Static checks and tests should be run from the project root; live Windows rendering remains unverified here.


## v0.6.9 — user-provided Aurora background video
- Bundled the user-provided `292822.mp4` as a local H.264 background, 1920×1080, 24 fps, approx. 20.04 seconds, no audio.
- Added a still poster fallback and 0.82× playback. Playback pauses when the document is hidden, when background motion is disabled, when reduced motion is enabled, or when the UI uses a non-video theme.
- Static references and video metadata were checked. Actual Electron playback and decode load still need testing on Windows hardware.


## v0.6.9 renderer verification
- Local video is included as `src/aurora-background.mp4`; app uses muted, looping 1920×1080 H.264 at 0.82× playback.
- Isolated Chromium renderer smoke test decoded the actual bundled video (`readyState >= 3`, width 1920, height 1080, duration about 20.04 s, playback time advancing).
- The UI pauses playback under the reduced-motion preference and on hidden documents.
- The test used stubbed IPC and does not measure CPU/GPU load on the user's Windows machine.
