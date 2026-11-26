# Vanta FX V136 (formerly Horizon Boost)

Renamed public-facing native Rust gaming utility. Existing profile save locations, Rust internal package name and preference migration are intentionally preserved to avoid breaking user settings.

## New desktop interface
- Flat graphite/OLED surface; removed the old giant purple gradient/hero, duplicated shortcut cards and extra pop-up feature navigation.
- Direct game picker, one real start/stop output button, output mode, GPU presets and actual GPU output health.
- Dedicated visible pages for Upscaling, Filters, Game Tools, Macros, HUD, Recording, Games, Hardware, Preferences — no nested accordions on the main workflow.
- Original application icon and Windows executable VERSIONINFO.
- Real DXGI and WGC renderer preserved. This is a UI restructure, not a claim that all games or features are newly operational.

## Known verification limits
- No verified Windows code-signing certificate on the machine; production Authenticode signature is not available. The shipped developer ZIP must be labelled UNSIGNED.
- GPU overlay / exclusive fullscreen / game input coordinates must be tested directly in a game. Test fixture screenshots do not prove live game effects.
- Retained GPL notices and shader source for third-party derived filters. No Magpie executable.
