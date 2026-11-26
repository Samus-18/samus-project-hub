# Vanta FX V136 — Native Windows GPU and game tools

![Real compiled Windows UI](../../docs/assets/vanta-home.png)

Vanta FX (formerly **Horizon Boost**) is a native Rust gaming utility for Windows 10/11, designed to run an external GPU output using Windows.Graphics.Capture, D3D11 filters and independent DXGI presentation. No game DLL injection. **Existing Rust project package and user profile paths remain unchanged intentionally** so current settings are preserved.

## V136 direct-control desktop rebuild

- New graphite OLED window with original Vanta icon and Windows product metadata.
- Full-width panels, 11 clearly labeled destinations and a clean dashboard.
- Direct GPU game selection, output modes, start/stop, shader presets, live effect strength, independent capture/output status.
- Upscaling and frame-generation settings, game tools, editor for offline macros, click-through HUD, local recording, media and preferences.
- Direct timeline/keyboard/mouse steps for macros — no second nested Macro Studio and duplicate controls.
- Legacy technical functionality still available under Developer lab, clearly separated.

## Evidence and limits

**561/561 Rust unit tests pass** (2 isolated live GPU integration tests intentionally ignored in the last test run). Windows GUI EXE compiled, read-only CLI passed, actual hidden egui visual QA screenshots were captured in EN/CZ, and the source ZIP has SHA256/manifest verification. See the V136 release notes.

Screenshots are of the current real native application UI in deterministic smoke mode, **not of effects applied live in Minecraft**. Universal fullscreen, all game input, latency and FPS improvements have not been certified.

## Signing / release safety

The V136 Windows executable is **UNSIGNED**: this PC has no trusted Authenticode publisher certificate and private key available. GitHub SHA-256 validates bytes only. A real production Authenticode signing certificate and timestamped signing process are required for digital publisher identity and reducing signing-related Windows warnings; this cannot be faked. See [Code Signing](../../tools/WINDOWS_SIGNING.md).

GPL-derived FSRCNNX/CuNNy shader source and corresponding GNU GPLv3 licenses are included with the downloadable developer ZIP. Magpie.exe is not bundled.

[Latest Vanta FX V136 dev release](https://github.com/Samus-18/samus-project-hub/releases/tag/vanta-fx-v136-dev)
