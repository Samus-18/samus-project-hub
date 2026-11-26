# Horizon Boost V135 — DXGI output accuracy (developer prerelease)

An independent native Rust Windows GPU utility: WGC, D3D11 HLSL, DXGI output; no injected game DLL and no Magpie executable.

## Changes
- DXGI Desktop Duplication fallback now counts **only S_OK Present** as own-output frames.
- A successful API call can return DXGI_STATUS_OCCLUDED, so those frames are excluded from output FPS measurements. Other non-submitted states reset pacing and increment diagnostics instead of generating optimistic FPS counters.
- DXGI_ERROR_DEVICE_REMOVED and other failures continue through existing recovery.
- Three new regression tests for success, occlusion/busy and GPU failure.
- All V134 multi-monitor fixes retained.

## Evidence and limits
558/558 Rust unit tests passed; 2 GPU WGC integration tests ignored in this round. Windows GUI EXE compiled, portable packager self-test, and 3 read-only CLI commands passed. SHA-256 checksum attached.

Not certified: live Minecraft cursor mapping, universal exclusive fullscreen, uninterrupted long sessions, or Magpie A/B performance. DXGI Present indicates submission, **not guaranteed physical display scanout**. No game FPS gain claimed.

The ZIP includes rebuild source, modified HLSL and GPLv3 notices for GPL-derived neural shaders. No Magpie.exe is packaged. Developer usage only.
