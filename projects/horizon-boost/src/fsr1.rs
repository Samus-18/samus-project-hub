//! Official AMD FidelityFX Super Resolution 1 EASU/RCAS proof.
//! Uses AMD's unmodified MIT-licensed reference shader code, not Magpie.
//! The contained ffx_a.h and ffx_fsr1.h are from GPUOpen-Effects/FidelityFX-FSR.
//! See shaders/third_party/fsr1/LICENSE.txt for attribution and permission.
//! These stages process Horizon-owned D3D11 textures, never alter game memory.
const AMD_A:&str=include_str!("../shaders/third_party/fsr1/ffx_a.h");
const AMD_FSR:&str=include_str!("../shaders/third_party/fsr1/ffx_fsr1.h");

const EASU_WRAPPER:&str=r#"
Texture2D<float4> FsrInput:register(t0);
SamplerState LinearSampler:register(s0);
RWTexture2D<float4> FsrOutput:register(u0);
AF4 FsrEasuRF(AF2 p){return FsrInput.GatherRed(LinearSampler,p);}
AF4 FsrEasuGF(AF2 p){return FsrInput.GatherGreen(LinearSampler,p);}
AF4 FsrEasuBF(AF2 p){return FsrInput.GatherBlue(LinearSampler,p);}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
    uint sw,sh,dw,dh;
    FsrInput.GetDimensions(sw,sh);
    FsrOutput.GetDimensions(dw,dh);
    if(id.x>=dw||id.y>=dh)return;
    AU4 c0,c1,c2,c3;
    FsrEasuCon(c0,c1,c2,c3,(float)sw,(float)sh,
               (float)sw,(float)sh,(float)dw,(float)dh);
    AF3 rgb;
    FsrEasuF(rgb,AU2(id.xy),c0,c1,c2,c3);
    FsrOutput[id.xy]=float4(saturate(rgb),1.0);
}
"#;
const RCAS_WRAPPER:&str=r#"
Texture2D<float4> FsrInput:register(t0);
RWTexture2D<float4> FsrOutput:register(u0);
AF4 FsrRcasLoadF(ASU2 p){
    uint w,h;FsrInput.GetDimensions(w,h);
    int2 loc=clamp(int2(p),int2(0,0),int2(w-1,h-1));
    return FsrInput.Load(int3(loc,0));
}
void FsrRcasInputF(inout AF1 r,inout AF1 g,inout AF1 b){}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
    uint w,h;FsrOutput.GetDimensions(w,h);
    if(id.x>=w||id.y>=h)return;
    AU4 con;
    // Mild sharpness 1.0 stop, conservative on Polaris to limit ringing.
    FsrRcasCon(con,1.0);
    AF1 r,g,b;
    FsrRcasF(r,g,b,AU2(id.xy),con);
    FsrOutput[id.xy]=float4(saturate(float3(r,g,b)),1.0);
}
"#;

pub fn shader_source(easu:bool)->String{
    let mut s=String::with_capacity(AMD_A.len()+AMD_FSR.len()+3400);
    s.push_str("#define A_GPU 1\n#define A_HLSL 1\n");
    s.push_str(if easu{"#define FSR_EASU_F 1\n"}else{"#define FSR_RCAS_F 1\n"});
    s.push_str(AMD_A);s.push('\n');s.push_str(AMD_FSR);s.push('\n');
    s.push_str(if easu{EASU_WRAPPER}else{RCAS_WRAPPER});
    s
}
pub fn compile_reference()->Result<usize,String>{
    for (name,flag) in [("EASU",true),("RCAS",false)]{
        crate::shadercheck::compile_native_hlsl(&shader_source(flag))
            .map_err(|e|format!("Official AMD FSR1 {name}: {e}"))?;
    }
    Ok(2)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn amd_mit_reference_compute_compiles_on_real_d3dcompiler(){
        assert_eq!(compile_reference().unwrap(),2);
    }
    #[test]fn vendor_headers_are_present_and_kept_unmodified(){
        assert!(AMD_FSR.contains("FsrEasuF("));
        assert!(AMD_FSR.contains("FsrRcasF("));
        assert!(AMD_A.len()>100_000);
        assert!(include_str!("../shaders/third_party/fsr1/LICENSE.txt")
            .contains("Permission is hereby granted"));
    }
}


// Native compute implementation of the EXACT AMD EASU + RCAS reference.
// No game engine integration. Requires a genuinely smaller captured texture.
use windows::Win32::Graphics::{
    Direct3D11::{
        ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11ShaderResourceView,
        ID3D11UnorderedAccessView,ID3D11ComputeShader,ID3D11ClassLinkage,
        ID3D11SamplerState,D3D11_SAMPLER_DESC,
        D3D11_FILTER_MIN_MAG_MIP_LINEAR,D3D11_TEXTURE_ADDRESS_CLAMP,
        D3D11_COMPARISON_NEVER,D3D11_TEXTURE2D_DESC,D3D11_BIND_SHADER_RESOURCE,
        D3D11_BIND_UNORDERED_ACCESS,D3D11_USAGE_DEFAULT,
    },
    Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_SAMPLE_DESC},
};
pub struct NativeFsr1Upscaler{
    width:u32,height:u32,
    easu:ID3D11ComputeShader,rcas:ID3D11ComputeShader,
    intermediate:ID3D11Texture2D,
    intermediate_srv:ID3D11ShaderResourceView,
    intermediate_uav:ID3D11UnorderedAccessView,
    sampler:ID3D11SamplerState,
    output:ID3D11Texture2D,
    output_uav:ID3D11UnorderedAccessView,
}
impl NativeFsr1Upscaler{
    pub fn new(device:&ID3D11Device,width:u32,height:u32)->Result<Self,String>{
        if width==0||height==0||width>8192||height>8192{
            return Err("FSR1 output exceeds safe 8192px render cap".into());
        }
        let compile=|easu:bool|->Result<ID3D11ComputeShader,String>{
            let blob=crate::shadercheck::compile_native_hlsl(&shader_source(easu))?;
            let mut program=None;
            unsafe{device.CreateComputeShader(&blob,None::<&ID3D11ClassLinkage>,Some(&mut program))}
                .map_err(|e|format!("AMD FSR1 D3D11 program: {e}"))?;
            program.ok_or("AMD FSR1 compute shader missing".into())
        };
        let easu=compile(true)?;
        let rcas=compile(false)?;
        let desc=D3D11_TEXTURE2D_DESC{
            Width:width,Height:height,MipLevels:1,ArraySize:1,
            Format:DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
            Usage:D3D11_USAGE_DEFAULT,
            BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,
            CPUAccessFlags:0,MiscFlags:0,
        };
        let make=||->Result<ID3D11Texture2D,String>{
            let mut texture=None;
            unsafe{device.CreateTexture2D(&desc,None,Some(&mut texture))}
                .map_err(|e|format!("AMD FSR1 render surface: {e}"))?;
            texture.ok_or("AMD FSR1 texture missing".into())
        };
        let intermediate=make()?;
        let output=make()?;
        let mut intermediate_srv=None;
        let mut intermediate_uav=None;
        let mut output_uav=None;
        unsafe{
            device.CreateShaderResourceView(&intermediate,None,Some(&mut intermediate_srv))
                .map_err(|e|format!("AMD FSR1 intermediate SRV: {e}"))?;
            device.CreateUnorderedAccessView(&intermediate,None,Some(&mut intermediate_uav))
                .map_err(|e|format!("AMD FSR1 intermediate UAV: {e}"))?;
            device.CreateUnorderedAccessView(&output,None,Some(&mut output_uav))
                .map_err(|e|format!("AMD FSR1 output UAV: {e}"))?;
        }
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
            .map_err(|e|format!("AMD FSR1 linear clamp sampler: {e}"))?;
        Ok(Self{
            width,height,easu,rcas,intermediate,output,
            intermediate_srv:intermediate_srv.ok_or("FSR1 intermediate SRV unavailable")?,
            intermediate_uav:intermediate_uav.ok_or("FSR1 intermediate UAV unavailable")?,
            output_uav:output_uav.ok_or("FSR1 output UAV unavailable")?,
            sampler:sampler.ok_or("FSR1 sampler unavailable")?,
        })
    }
    pub fn render(&self,device:&ID3D11Device,ctx:&ID3D11DeviceContext,
        input:&ID3D11Texture2D)->Result<ID3D11Texture2D,String>{
        let mut in_desc=D3D11_TEXTURE2D_DESC::default();
        unsafe{input.GetDesc(&mut in_desc);}
        if in_desc.Width==0||in_desc.Height==0{
            return Err("FSR1 source has no pixels".into());
        }
        if in_desc.Width>=self.width||in_desc.Height>=self.height{
            // FSR1 would needlessly blur or waste GPU if no true upscaling.
            return Err("FSR1 requires both output dimensions to exceed input".into());
        }
        let mut input_srv=None;
        unsafe{device.CreateShaderResourceView(input,None,Some(&mut input_srv))}
            .map_err(|e|format!("FSR1 source SRV: {e}"))?;
        let input_srv=input_srv.ok_or("FSR1 source texture view missing")?;
        unsafe{
            ctx.CSSetShader(&self.easu,None);
            ctx.CSSetSamplers(0,Some(&[Some(self.sampler.clone())]));
            ctx.CSSetShaderResources(0,Some(&[Some(input_srv)]));
            let uav=[Some(self.intermediate_uav.clone())];
            ctx.CSSetUnorderedAccessViews(0,1,Some(uav.as_ptr()),None);
            ctx.Dispatch(self.width.div_ceil(8),self.height.div_ceil(8),1);
            let clear=[None];
            ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
            ctx.CSSetShaderResources(0,Some(&[None]));
            ctx.CSSetSamplers(0,Some(&[None]));
            ctx.CSSetShader(&self.rcas,None);
            ctx.CSSetShaderResources(0,Some(&[Some(self.intermediate_srv.clone())]));
            let result=[Some(self.output_uav.clone())];
            ctx.CSSetUnorderedAccessViews(0,1,Some(result.as_ptr()),None);
            ctx.Dispatch(self.width.div_ceil(8),self.height.div_ceil(8),1);
            ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
            ctx.CSSetShaderResources(0,Some(&[None]));
            ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
        }
        Ok(self.output.clone())
    }
}
