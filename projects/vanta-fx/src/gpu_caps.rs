//! Read-only RX580 hardware capability probe using actual Direct3D11 driver.
 //! This probes the real GPU adapter instead of guessing from vendor support lists.
 //! No driver setting, registry, executable, clock or game memory changes.
use windows::Win32::{
    Foundation::HMODULE,
    Graphics::{
        Direct3D::{D3D_DRIVER_TYPE_HARDWARE,D3D_FEATURE_LEVEL_11_0},
        Direct3D11::{
            D3D11CreateDevice,D3D11_CREATE_DEVICE_BGRA_SUPPORT,D3D11_SDK_VERSION,
            D3D11_FORMAT_SUPPORT_TYPED_UNORDERED_ACCESS_VIEW,ID3D11Device,ID3D11DeviceContext,
        },
        Dxgi::{
            IDXGIAdapter,
            Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_FORMAT_R16G16B16A16_FLOAT,
                DXGI_FORMAT_R32_FLOAT},
        },
    },
};
#[derive(Debug,Clone)]
pub struct NativeCapabilities {
    pub adapter:String,
    pub feature_level_11:bool,
    pub rgba8_typed_uav:bool,
    pub rgba16f_typed_uav:bool,
    pub r32f_typed_uav:bool,
    pub original_cs_5_0_compiles:bool,
}
const UAV_FLAG:u32=D3D11_FORMAT_SUPPORT_TYPED_UNORDERED_ACCESS_VIEW.0 as u32;
fn supports_typed_uav(flags:u32)->bool{flags&UAV_FLAG==UAV_FLAG}
impl NativeCapabilities {
    pub fn summary(&self)->String{
        format!("RX580_NATIVE_D3D11_CAPS=OK ADAPTER={:?} FEATURE_LEVEL_11_0={} \
                UAV_RGBA8={} UAV_RGBA16F={} UAV_R32F={} HLSL_CS_5_0_COMPILED={} \
                READ_ONLY=true IN_ENGINE_FRAMEGEN_NOT_PROVEN=true",
            self.adapter,self.feature_level_11,self.rgba8_typed_uav,
            self.rgba16f_typed_uav,self.r32f_typed_uav,self.original_cs_5_0_compiles)
    }
}
pub fn read_native_capabilities()->Result<NativeCapabilities,String>{
    let mut device:Option<ID3D11Device>=None;
    let mut context:Option<ID3D11DeviceContext>=None;
    unsafe{
        D3D11CreateDevice(None::<&IDXGIAdapter>,D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&[D3D_FEATURE_LEVEL_11_0]),D3D11_SDK_VERSION,
            Some(&mut device),None,Some(&mut context))
    }.map_err(|e|format!("Hardware D3D11 feature-level 11 device refused: {e}"))?;
    let device=device.ok_or("Driver did not create D3D11 hardware device")?;
    let _context=context.ok_or("D3D11 hardware device lacks immediate context")?;
    let rgba8=unsafe{device.CheckFormatSupport(DXGI_FORMAT_R8G8B8A8_UNORM)}
        .map_err(|e|format!("RGBA8 capability query failed: {e}"))?;
    let rgba16f=unsafe{device.CheckFormatSupport(DXGI_FORMAT_R16G16B16A16_FLOAT)}
        .map_err(|e|format!("RGBA16F capability query failed: {e}"))?;
    let r32=unsafe{device.CheckFormatSupport(DXGI_FORMAT_R32_FLOAT)}
        .map_err(|e|format!("R32F capability query failed: {e}"))?;
    // Shader compilation is required but not sufficient for execution. Our
    // separate native spatial proof dispatches CS_5_0 on the actual RX580.
    let shader="RWTexture2D<float4> Result:register(u0);\n\
        [numthreads(8,8,1)] void CSMain(uint3 p:SV_DispatchThreadID){\
        if(p.x<8&&p.y<8) Result[p.xy]=float4(.25,.5,.75,1.0);}";
    let compiled=crate::shadercheck::compile_native_hlsl(shader).is_ok();
    Ok(NativeCapabilities {
        adapter:crate::win::gpu_name(),
        feature_level_11:true,
        rgba8_typed_uav:supports_typed_uav(rgba8),
        rgba16f_typed_uav:supports_typed_uav(rgba16f),
        r32f_typed_uav:supports_typed_uav(r32),
        original_cs_5_0_compiles:compiled
    })
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn flags_are_interpreted_without_invented_capability(){
        assert!(!supports_typed_uav(0));
        assert!(supports_typed_uav(UAV_FLAG));
        assert!(supports_typed_uav(u32::MAX));
        assert!(!supports_typed_uav(1));
    }
}
