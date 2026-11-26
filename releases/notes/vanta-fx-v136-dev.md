# Vanta FX V136 — Complete native GUI rebuild (UNSIGNED DEV)

Renamed from Horizon Boost to Vanta FX without breaking existing user profiles or internal Rust compatibility.

## Real changes
- Full native egui UI replacement: graphite OLED window, original vector V logo/icon, simpler navigation, full-width panels, no giant gradient banner, duplicate home shortcuts or page-wide pop-up menus.
- Direct GPU settings, shader presets and strength, capture modes, independent FPS diagnostics, manual scaling, frame generation controls.
- One macros screen with direct Record/Play/Stop, speed/repeat, saved profiles and a real editable event timeline (no duplicate Macro Studio); reset consent when target window changes.
- Game tools, crosshair, click-through HUD and recording/replay use existing actions instead of disconnected mock controls.
- Windows executable ProductName and VersionInfo for Vanta FX.
- Preserved native WGC/D3D11/DXGI engine and all V135 DXGI Present status correctness changes.

## Verified and not verified
- 561/561 Rust tests passed, 2 GPU integration test fixtures ignored.
- Windows GUI PE compiled, nine actual native egui hidden screenshot smoke tests and three read-only CLI operations passed.
- ZIP integrity, SHA-256 sidecar, matching GPL shader sources/licenses verified. No Magpie executable or private AI memory is packaged.
- **UNSIGNED:** no trusted Authenticode certificate/private key available on the developer PC. Checksums do NOT constitute digital signing.
- **Not live-game certified:** full Minecraft mouse input, every exclusive-fullscreen engine, performance gains, frame-generation latency, anti-cheat acceptance and SmartScreen reputation.

Original older Horizon Boost releases remain available for rollback. This release is a developer prerelease, not an automatic upgrade.
