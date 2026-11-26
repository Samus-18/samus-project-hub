# Vanta FX V138 - UNSIGNED DEV

- Fix shared Vanta FX HUD and main Windows HWND title contract.
- Real RAWINPUT delta capture, opt-in per recording, bounded timeline,
  existing same-game focus and anti-cheat-heuristic protections retained.
- Restore the app's prior raw mouse device registration after recording.
- Replay is simulated via SendInput; not guaranteed to turn Roblox camera.
- 569 Rust tests, native GUI build, real hidden GUI/HUD smoke tests,
  read-only CLI, ZIP file manifest and checksum verification.
- Not certified: exclusive fullscreen in every game, Shift Lock replay,
  FG input lag or real game FPS improvements; unsigned executable.

## Source and verification
The new Raw Input listener uses a hidden message-only Windows HWND; it stores relative mouse deltas only during the explicitly approved recording. Restores prior mouse device registration on completion. For replay Vanta FX uses Win32 SendInput, not injected game raw HID. This has not been tested in Roblox Shift Lock or exclusive fullscreen. Windows code-signing certificate unavailable. V137 and earlier releases remain.
