//! NVIDIA Image Scaling vendor reference shader, compiled for generic AMD D3D11.
//! This is MIT-licensed NVIDIA SDK v1.0.3, not NVIDIA driver control panel.
//! Vendor sources are at shaders/third_party/nis/NIS_Main.hlsl,
//! NIS_Scaler.h and NIS_Config.h (with original MIT notices).
//! No game process code injection. Independent own output only.
const NIS_MAIN:&str=include_str!("../shaders/third_party/nis/NIS_Main.hlsl");
const NIS_SHADER:&str=include_str!("../shaders/third_party/nis/NIS_Scaler.h");
const NIS_COEF_SCALE:&[u8;2048]=include_bytes!("../shaders/third_party/nis/coef_scale.bin");
const NIS_COEF_USM:&[u8;2048]=include_bytes!("../shaders/third_party/nis/coef_usm.bin");
pub fn compute_source()->String{
    let replaced=NIS_MAIN.replace("#include \"NIS_Scaler.h\"",NIS_SHADER)
        .replace("void main(","void CSMain(");
    format!("#define NIS_THREAD_GROUP_SIZE 256\n#define NIS_BLOCK_WIDTH 32\n\
        #define NIS_BLOCK_HEIGHT 24\n#define NIS_CLAMP_OUTPUT 1\n{replaced}")
}
pub fn compile_vendor()->Result<Vec<u8>,String>{
    crate::shadercheck::compile_native_hlsl(&compute_source())
        .map_err(|e|format!("NVIDIA NIS vendor HLSL 5.0 compile failed on AMD: {e}"))
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn exact_vendor_nis_hlsl5_compiles_for_amd(){
        assert!(!compile_vendor().unwrap().is_empty());
        assert_eq!(NIS_COEF_SCALE.len(),64*8*4);
        assert_eq!(NIS_COEF_USM.len(),64*8*4);
    }
}


use windows::Win32::Graphics::{
    Direct3D11::{
        ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11Buffer,
        ID3D11ShaderResourceView,ID3D11UnorderedAccessView,ID3D11ComputeShader,
        ID3D11ClassLinkage,ID3D11SamplerState,D3D11_SAMPLER_DESC,
        D3D11_FILTER_MIN_MAG_MIP_LINEAR,D3D11_TEXTURE_ADDRESS_CLAMP,D3D11_COMPARISON_NEVER,
        D3D11_TEXTURE2D_DESC,D3D11_SUBRESOURCE_DATA,D3D11_BUFFER_DESC,
        D3D11_BIND_CONSTANT_BUFFER,D3D11_BIND_SHADER_RESOURCE,D3D11_BIND_UNORDERED_ACCESS,
        D3D11_USAGE_IMMUTABLE,D3D11_USAGE_DEFAULT,
    },
    Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_FORMAT_R32G32B32A32_FLOAT,
        DXGI_SAMPLE_DESC},
};
/// Direct Rust translation of NVIDIA NIS_Config.h NVScalerUpdateConfig for
/// SDR, with neutral 50% sharpness. Retains the original vendor scale and USM
/// coefficient tables (textures 2 x 64 RGBA32F). All values are 32-bit words.
fn vendor_config(src:(u32,u32),dst:(u32,u32))->Result<[u32;28],String>{
    let ((sw,sh),(dw,dh))=(src,dst);
    if sw==0||sh==0||dw==0||dh==0||sw>8192||sh>8192||dw>8192||dh>8192 {
        return Err("NIS invalid/unsafe capture/output dimensions".into());
    }
    let sx=sw as f32/dw as f32;
    let sy=sh as f32/dh as f32;
    if !(0.5..=1.0).contains(&sx)||!(0.5..=1.0).contains(&sy)
        ||sw>=dw||sh>=dh{
        return Err("NIS scaling only accepts genuine upscaling by <=2x on both axes".into());
    }
    let mut data=[0u32;28];
    let settings=[
        2.0*1127.0/1024.0, 64.0/1024.0, 2.0, 1.0/(10.0-2.0),
        1.0, 1.0/255.0, 0.45, 1.0/(0.9-0.45),
        0.4, 1.6-0.4, 0.14, 0.5-0.14,
        sx, sy, 1.0/dw as f32, 1.0/dh as f32,
        1.0/sw as f32, 1.0/sh as f32,
    ];
    for (i,v) in settings.iter().enumerate(){data[i]=(*v as f32).to_bits();}
    data[18]=0;data[19]=0;data[20]=sw;data[21]=sh;
    data[22]=0;data[23]=0;data[24]=dw;data[25]=dh;
    Ok(data)
}

pub struct NativeNisScaler{
    source:(u32,u32),target:(u32,u32),
    compute:ID3D11ComputeShader,
    config:ID3D11Buffer,
    coeff_scale:ID3D11ShaderResourceView,
    coeff_usm:ID3D11ShaderResourceView,
    sampler:ID3D11SamplerState,
    output:ID3D11Texture2D,
    output_uav:ID3D11UnorderedAccessView,
}
impl NativeNisScaler{
    pub fn new(device:&ID3D11Device,source:(u32,u32),target:(u32,u32))
        ->Result<Self,String>{
        let cfg=vendor_config(source,target)?;
        let binary=compile_vendor()?;
        let mut compute=None;
        unsafe{device.CreateComputeShader(&binary,None::<&ID3D11ClassLinkage>,Some(&mut compute))}
            .map_err(|e|format!("NVIDIA NIS compute on AMD RX580: {e}"))?;
        let compute=compute.ok_or("NIS compute shader unavailable")?;
        let mut bd=D3D11_BUFFER_DESC::default();
        bd.ByteWidth=(cfg.len()*4) as u32;
        bd.Usage=D3D11_USAGE_IMMUTABLE;
        bd.BindFlags=D3D11_BIND_CONSTANT_BUFFER.0 as u32;
        let init=D3D11_SUBRESOURCE_DATA{
            pSysMem:cfg.as_ptr() as *const _,SysMemPitch:0,SysMemSlicePitch:0,
        };
        let mut buffer=None;
        unsafe{device.CreateBuffer(&bd,Some(&init),Some(&mut buffer))}
            .map_err(|e|format!("NIS official SDR cbuffer: {e}"))?;
        let config=buffer.ok_or("NIS config constant buffer unavailable")?;
        let create_coeff=|bytes:&[u8;2048]|->Result<ID3D11ShaderResourceView,String>{
            let desc=D3D11_TEXTURE2D_DESC{
                Width:2,Height:64,MipLevels:1,ArraySize:1,
                Format:DXGI_FORMAT_R32G32B32A32_FLOAT,
                SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
                Usage:D3D11_USAGE_IMMUTABLE,
                BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,
                CPUAccessFlags:0,MiscFlags:0,
            };
            let init=D3D11_SUBRESOURCE_DATA{
                pSysMem:bytes.as_ptr() as *const _,
                SysMemPitch:2*16,SysMemSlicePitch:2048,
            };
            let mut texture=None;
            unsafe{device.CreateTexture2D(&desc,Some(&init),Some(&mut texture))}
                .map_err(|e|format!("NIS official coeff texture: {e}"))?;
            let texture=texture.ok_or("NIS coefficient texture unavailable")?;
            let mut view=None;
            unsafe{device.CreateShaderResourceView(&texture,None,Some(&mut view))}
                .map_err(|e|format!("NIS coefficient SRV: {e}"))?;
            view.ok_or("NIS coefficient view unavailable".into())
        };
        let coeff_scale=create_coeff(NIS_COEF_SCALE)?;
        let coeff_usm=create_coeff(NIS_COEF_USM)?;
        let mut sd=D3D11_SAMPLER_DESC::default();
        sd.Filter=D3D11_FILTER_MIN_MAG_MIP_LINEAR;
        sd.AddressU=D3D11_TEXTURE_ADDRESS_CLAMP;
        sd.AddressV=D3D11_TEXTURE_ADDRESS_CLAMP;
        sd.AddressW=D3D11_TEXTURE_ADDRESS_CLAMP;
        sd.MaxAnisotropy=1;
        sd.ComparisonFunc=D3D11_COMPARISON_NEVER;
        sd.MaxLOD=f32::MAX;
        let mut sampler=None;
        unsafe{device.CreateSamplerState(&sd,Some(&mut sampler))}
            .map_err(|e|format!("NIS clamp sampler: {e}"))?;
        let sampler=sampler.ok_or("NIS clamp sampler unavailable")?;
        let desc=D3D11_TEXTURE2D_DESC{
            Width:target.0,Height:target.1,MipLevels:1,ArraySize:1,
            Format:DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
            Usage:D3D11_USAGE_DEFAULT,
            BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,
            CPUAccessFlags:0,MiscFlags:0,
        };
        let mut output=None;
        unsafe{device.CreateTexture2D(&desc,None,Some(&mut output))}
            .map_err(|e|format!("NIS output: {e}"))?;
        let output=output.ok_or("NIS output surface unavailable")?;
        let mut output_uav=None;
        unsafe{device.CreateUnorderedAccessView(&output,None,Some(&mut output_uav))}
            .map_err(|e|format!("NIS output UAV: {e}"))?;
        Ok(Self{source,target,compute,config,coeff_scale,coeff_usm,sampler,output,
            output_uav:output_uav.ok_or("NIS output UAV unavailable")?})
    }
    pub fn render(&self,device:&ID3D11Device,ctx:&ID3D11DeviceContext,
        input:&ID3D11Texture2D)->Result<ID3D11Texture2D,String>{
        let mut desc=D3D11_TEXTURE2D_DESC::default();
        unsafe{input.GetDesc(&mut desc);}
        if (desc.Width,desc.Height)!=self.source {
            return Err("NIS input changed dimensions; rebuild GPU scaler".into());
        }
        let mut input_srv=None;
        unsafe{device.CreateShaderResourceView(input,None,Some(&mut input_srv))}
            .map_err(|e|format!("NIS input SRV: {e}"))?;
        let input_srv=input_srv.ok_or("NIS input SRV missing")?;
        let sources=[
            Some(input_srv),Some(self.coeff_scale.clone()),Some(self.coeff_usm.clone())
        ];
        let output=[Some(self.output_uav.clone())];
        unsafe{
            ctx.CSSetShader(&self.compute,None);
            ctx.CSSetConstantBuffers(0,Some(&[Some(self.config.clone())]));
            ctx.CSSetSamplers(0,Some(&[Some(self.sampler.clone())]));
            ctx.CSSetShaderResources(0,Some(&sources));
            ctx.CSSetUnorderedAccessViews(0,1,Some(output.as_ptr()),None);
            ctx.Dispatch(self.target.0.div_ceil(32),self.target.1.div_ceil(24),1);
            let empty=[None];
            ctx.CSSetUnorderedAccessViews(0,1,Some(empty.as_ptr()),None);
            ctx.CSSetShaderResources(0,Some(&[None,None,None]));
            ctx.CSSetSamplers(0,Some(&[None]));
            ctx.CSSetConstantBuffers(0,Some(&[None]));
            ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
        }
        Ok(self.output.clone())
    }
}
#[cfg(test)]
mod vendor_config_tests{
    use super::*;
    #[test]
    fn config_is_licensed_reference_and_rejects_bad_scale(){
        assert!(vendor_config((640,360),(1280,720)).is_ok());
        assert!(vendor_config((1280,720),(1280,720)).is_err());
        assert!(vendor_config((640,360),(2560,1440)).is_err());
        assert_eq!(vendor_config((640,360),(1280,720)).unwrap()[20],640);
        assert_eq!(vendor_config((640,360),(1280,720)).unwrap()[24],1280);
    }
}
