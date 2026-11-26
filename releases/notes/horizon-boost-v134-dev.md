# Horizon Boost V134 — Monitor stability (DEVELOPER PRE-RELEASE)

Rust GPU utility using WGC→D3D11 HLSL→independent DXGI output. Fixes false auto-monitor mismatch causing repeated WGC rebuilds, adds debounced game-window monitor change detection for multi-monitor, negative coordinates, disconnect and resize; adds three regression tests.

**Verification:** 555/555 Rust tests, 2 integration tests ignored, Windows GUI EXE build, packaging self-test and three-command read-only CLI passed. Previous V133 had two isolated WGC GPU test sessions. **Not confirmed:** real Minecraft mouse mapping, all true-exclusive fullscreen, alt-tab on every engine, measured speed advantage over Magpie.

GPL-derived CuNNy/FSRCNNX shaders require the included GPLv3 license and corresponding modified HLSL plus full source. No Magpie.exe binary is shipped. This is an experimental, non-production build.
