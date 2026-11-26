# Third-party source notices — Horizon Boost v2.7-dev

## AMD FidelityFX FSR1
- Source: https://github.com/GPUOpen-Effects/FidelityFX-FSR
- Files: shaders/third_party/fsr1/ffx_a.h, ffx_fsr1.h
- Copyright (c) 2021 Advanced Micro Devices, Inc.
- MIT license: shaders/third_party/fsr1/LICENSE.txt
- Used by independent native D3D11 EASU and RCAS compute passes only on Horizon-owned pixel output. Does not integrate with or modify any game's GPU render loop.

## NVIDIA Image Scaling
- Source: https://github.com/NVIDIAGameWorks/NVIDIAImageScaling
- Files: shaders/third_party/nis/NIS_Main.hlsl, NIS_Scaler.h, NIS_Config.h, coef_scale.bin, coef_usm.bin.
- Copyright (c) 2022 NVIDIA CORPORATION & AFFILIATES.
- Original MIT text and permission notice in NIS_Main.hlsl and NIS_Scaler.h. LICENSE.txt alongside.
- Original 64-phase coefficient tables are reproduced as little-endian FP32 textures, 2x64xRGBA32F.
- Nvidia's open image-scaling compute algorithm runs on a Direct3D11 Radeon card without NVIDIA Control Panel.
- No assertion of DLSS/RTX capability or proprietary driver features.

All tested results refer to independent own DXGI output; external captured frame post-processing does NOT change game engine frames or its render resolution.
