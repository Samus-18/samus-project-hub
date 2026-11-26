//! Experimental real scRGB FP16 swapchain output for Horizon-owned preview.
//! Source capture is SDR; accurate linear 80-nit scRGB encoding is *not*
//! game HDR, physical monitor calibration or HDR content recovery.
use windows::{core::Interface,Win32::Graphics::{
 Direct3D11::{ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11ShaderResourceView,
   ID3D11UnorderedAccessView,ID3D11ComputeShader,ID3D11ClassLinkage,ID3D11Buffer,ID3D11Resource,
   D3D11_BIND_CONSTANT_BUFFER,D3D11_BUFFER_DESC,
   D3D11_TEXTURE2D_DESC,D3D11_BIND_UNORDERED_ACCESS,D3D11_USAGE_DEFAULT},
 Dxgi::{IDXGISwapChain1,IDXGISwapChain3,DXGI_SWAP_CHAIN_COLOR_SPACE_SUPPORT_FLAG_PRESENT},
 Dxgi::Common::{DXGI_COLOR_SPACE_RGB_FULL_G10_NONE_P709,DXGI_FORMAT_R16G16B16A16_FLOAT,DXGI_SAMPLE_DESC}
}};
pub const SHADER:&str=include_str!("../shaders/native/HB_SDRToScRGB.hlsl");
pub fn available_monitor(name:Option<&str>)->Result<bool,String>{
 let Some(name)=name else{return Ok(false)};
 let displays=crate::hdr_probe::probe()?;
 Ok(displays.iter().any(|d|d.name.eq_ignore_ascii_case(name)&&d.hdr10_active&&d.bits>=10))
}
pub fn set_color_space(chain:&IDXGISwapChain1)->Result<(),String>{
 let actual:IDXGISwapChain3=chain.cast().map_err(|e|format!("DXGI SwapChain3 for scRGB: {e}"))?;
 let support=unsafe{actual.CheckColorSpaceSupport(DXGI_COLOR_SPACE_RGB_FULL_G10_NONE_P709)}
  .map_err(|e|format!("DXGI scRGB support query: {e}"))?;
 if support&DXGI_SWAP_CHAIN_COLOR_SPACE_SUPPORT_FLAG_PRESENT.0 as u32==0{
  return Err("Monitor/driver cannot present FP16 scRGB in current Windows HDR mode".into());
 }
 unsafe{actual.SetColorSpace1(DXGI_COLOR_SPACE_RGB_FULL_G10_NONE_P709)}
  .map_err(|e|format!("DXGI SetColorSpace1 scRGB refused: {e}"))?;
 Ok(())
}
pub struct Processor{
 pub output:ID3D11Texture2D,
 target:ID3D11UnorderedAccessView,
 shader:ID3D11ComputeShader,
 constants:ID3D11Buffer,
 cache:crate::reconstruction::ViewCache,
 size:(u32,u32),
}
impl Processor{
 pub fn new(device:&ID3D11Device,w:u32,h:u32)->Result<Self,String>{
  if w==0||h==0||w>8192||h>8192{return Err("Invalid own scRGB output dimensions".into());}
  let supported=unsafe{device.CheckFormatSupport(DXGI_FORMAT_R16G16B16A16_FLOAT)}
   .map_err(|e|format!("GPU FP16 output support: {e}"))?;
  if supported&(D3D11_BIND_UNORDERED_ACCESS.0 as u32)==0{
   // CheckFormatSupport does not share D3D11_BIND flag values, so test the
   // *actual* resource creation below instead of trusting this flag value.
   let _=supported;
  }
  let desc=D3D11_TEXTURE2D_DESC{
   Width:w,Height:h,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R16G16B16A16_FLOAT,
   SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,
   BindFlags:D3D11_BIND_UNORDERED_ACCESS.0 as u32,CPUAccessFlags:0,MiscFlags:0,
  };
  let mut texture=None;
  unsafe{device.CreateTexture2D(&desc,None,Some(&mut texture))}
   .map_err(|e|format!("Own FP16 output unavailable on adapter: {e}"))?;
  let output=texture.ok_or("Own FP16 output texture missing")?;
  let mut uav=None;
  unsafe{device.CreateUnorderedAccessView(&output,None,Some(&mut uav))}
   .map_err(|e|format!("Own scRGB GPU UAV incompatible: {e}"))?;
  let target=uav.ok_or("Own scRGB output UAV missing")?;
  let blob=crate::shadercheck::compile_native_hlsl(SHADER)?;
  let mut shader=None;
  unsafe{device.CreateComputeShader(&blob,None::<&ID3D11ClassLinkage>,Some(&mut shader))}
   .map_err(|e|format!("scRGB GPU shader unavailable: {e}"))?;
  let mut constants=None;
  let constant_desc=D3D11_BUFFER_DESC{
     ByteWidth:16,Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,
     CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0,
  };
  unsafe{device.CreateBuffer(&constant_desc,None,Some(&mut constants))}
      .map_err(|e|format!("scRGB calibration cbuffer: {e}"))?;
  Ok(Self{output,target,shader:shader.ok_or("scRGB GPU shader missing")?,
     constants:constants.ok_or("No scRGB calibration buffer")?,
     cache:Default::default(),size:(w,h)})
 }
 pub fn render(&mut self,device:&ID3D11Device,ctx:&ID3D11DeviceContext,input:&ID3D11Texture2D,white_nits:f32)->Result<(),String>{self.render_with(device,ctx,input,white_nits,0,400.,1.)}
 pub fn render_configured(&mut self,device:&ID3D11Device,ctx:&ID3D11DeviceContext,input:&ID3D11Texture2D,white_nits:f32)->Result<(),String>{
  let packed=CURVE.load(std::sync::atomic::Ordering::Relaxed);
  self.render_with(device,ctx,input,white_nits,(packed&3)as u8,((packed>>2)&0x3fff)as f32,((packed>>16)&0xff)as f32/10.)
 }
 fn render_with(&mut self,device:&ID3D11Device,ctx:&ID3D11DeviceContext,input:&ID3D11Texture2D,white_nits:f32,mode:u8,peak:f32,exposure:f32)->Result<(),String>{
  let mut d=D3D11_TEXTURE2D_DESC::default();unsafe{input.GetDesc(&mut d);}
  if (d.Width,d.Height)!=self.size{return Err("Output resized; rebuild own scRGB GPU surface".into());}
  let srv:ID3D11ShaderResourceView=self.cache.get(device,input)?;
  // WhiteScale constant feeds HLSL and is bounded 80–400 nits user reference.
  let scale=if white_nits.is_finite(){white_nits.clamp(80.,400.)/80.}else{1.25};
  let values=[scale,f32::from(mode.min(2)),peak.clamp(80.,10000.)/80.,exposure.clamp(0.5,4.)];
  let constants:ID3D11Resource=self.constants.cast().map_err(|e|e.to_string())?;
  unsafe{
   ctx.UpdateSubresource(&constants,0,None,values.as_ptr().cast(),0,0);
   ctx.CSSetConstantBuffers(0,Some(&[Some(self.constants.clone())]));
   ctx.CSSetShader(&self.shader,None);
   ctx.CSSetShaderResources(0,Some(&[Some(srv)]));
   let views=[Some(self.target.clone())];
   ctx.CSSetUnorderedAccessViews(0,1,Some(views.as_ptr()),None);
   ctx.Dispatch(self.size.0.div_ceil(8),self.size.1.div_ceil(8),1);
   ctx.CSSetShaderResources(0,Some(&[None]));
   ctx.CSSetConstantBuffers(0,Some(&[None]));
   ctx.CSSetUnorderedAccessViews(0,1,Some([None].as_ptr()),None);
   ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
  }
  Ok(())
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn actual_gpu_converts_sdr_into_linear_fp16_sc_rgb_texture(){
  let gpu=crate::photo_lab::Gpu::new().unwrap();
  let img=image::RgbaImage::from_fn(23,19,|x,_y|image::Rgba([if x<11{128}else{255},128,128,255]));
  let source=gpu.upload(&img).unwrap();
  let mut p=Processor::new(&gpu.device,23,19).unwrap();
  p.render(&gpu.device,&gpu.ctx,&source,100.).unwrap();
  crate::native_filter::verify_gpu_completion(&gpu.device,&gpu.ctx).unwrap();
  use windows::Win32::Graphics::Direct3D11::{
    D3D11_USAGE_STAGING,D3D11_CPU_ACCESS_READ,ID3D11Resource,D3D11_MAPPED_SUBRESOURCE,D3D11_MAP_READ};
  let mut d=D3D11_TEXTURE2D_DESC::default();unsafe{p.output.GetDesc(&mut d);}
  assert_eq!(d.Format,DXGI_FORMAT_R16G16B16A16_FLOAT);
  let desc=D3D11_TEXTURE2D_DESC{Usage:D3D11_USAGE_STAGING,BindFlags:0,
     CPUAccessFlags:D3D11_CPU_ACCESS_READ.0 as u32,..d};
  let mut read=None;unsafe{gpu.device.CreateTexture2D(&desc,None,Some(&mut read))}.unwrap();
  let staged=read.unwrap();
  let to:ID3D11Resource=staged.cast().unwrap();
  let from:ID3D11Resource=p.output.cast().unwrap();
  unsafe{gpu.ctx.CopyResource(&to,&from);}
  crate::native_filter::verify_gpu_completion(&gpu.device,&gpu.ctx).unwrap();
  let mut mapped=D3D11_MAPPED_SUBRESOURCE::default();
  unsafe{gpu.ctx.Map(&to,0,D3D11_MAP_READ,0,Some(&mut mapped))}.unwrap();
  let half=|x:u16|{
   let exp=(x>>10)&31;let frac=x&1023;
   if exp==0{(frac as f32/1024.)*2f32.powi(-14)}
   else{(1.+frac as f32/1024.)*2f32.powi(exp as i32-15)}
  };
  let col=|x:usize|half(unsafe{*(mapped.pData.cast::<u16>().add(x*4))});
  let dark=col(5);let bright=col(16);
  unsafe{gpu.ctx.Unmap(&to,0);}
  assert!((dark-0.270).abs()<0.035,"sRGB 0.5 must map near linear scRGB .27, observed {dark}");
  assert!((bright-1.25).abs()<0.035,"SDR 100nit white should be 1.25x scRGB 80 nit reference, got {bright}");
 }
}

static CURVE:std::sync::atomic::AtomicU32=std::sync::atomic::AtomicU32::new(0);
pub fn configure(mode:u8,peak:u32,exposure:u8){CURVE.store(u32::from(mode.min(2))|(peak.clamp(80,10000)<<2)|(u32::from(exposure.clamp(5,40))<<16),std::sync::atomic::Ordering::Relaxed);}
