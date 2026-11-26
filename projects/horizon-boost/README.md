# Horizon Boost — Rust GPU utilities

![Horizon Boost UI](../../docs/assets/horizon-home.png)

Windows-native GPU effect suite designed for low-overhead capture and custom image processing without game executable injection. Uses WGC (Windows.Graphics.Capture), D3D11 compute filters and its own DXGI presentation. Options include spatial upscaling, experimental frame generation, filter stacks, OSD, crosshair, macro tools and profiles.

## V134 developer state

- 555/555 Rust unit tests passed; two integration tests were excluded from the unit pass.
- Windows GUI EXE built, native packager self-test and read-only three-command CLI test passed.
- Fixed incorrect monitor topology comparison triggering unnecessary WGC/DXGI rebuilds, and added debounced source-game monitor tracking and three regressions for negative coordinates, resize and disconnect.
- V133 had two controlled WGC/HLSL/DXGI capture test sessions and ten EN/CZ screenshots.

**Not verified:** Minecraft mouse coordinate remapping after upscaling, universal exclusive fullscreen, speed advantage over Magpie, real-game long-session stability.

## Public source snapshot

Includes selected Rust modules, shaders and manifests and attribution files. Machine-local configuration, third-party engines (including Magpie), test videos, original user memory and local binary tooling are excluded. A complete historical distribution may require resources available only in its dedicated release packages. Do not claim that this is a production-certified installer.

[Development release](https://github.com/Samus-18/samus-project-hub/releases/tag/horizon-boost-v134-dev)
