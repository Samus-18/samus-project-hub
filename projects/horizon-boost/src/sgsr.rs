//! Qualcomm SGSR1 native D3D11 compute adapter, no Magpie runtime.
use windows::Win32::Graphics::Direct3D11::*;
pub struct Pass{shader:ID3D11ComputeShader,sampler:ID3D11SamplerState}
impl Pass{
 pub fn new(device:&ID3D11Device)->Result<Self,String>{
  let bytes=crate::shadercheck::compile_native_hlsl(include_str!("../shaders/native/HB_SGSR1.hlsl"))?;
  let mut shader=None;unsafe{device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|e.to_string())?;
  let mut sd=D3D11_SAMPLER_DESC::default();sd.Filter=D3D11_FILTER_MIN_MAG_MIP_LINEAR;sd.AddressU=D3D11_TEXTURE_ADDRESS_CLAMP;sd.AddressV=D3D11_TEXTURE_ADDRESS_CLAMP;sd.AddressW=D3D11_TEXTURE_ADDRESS_CLAMP;sd.MaxAnisotropy=1;sd.ComparisonFunc=D3D11_COMPARISON_NEVER;sd.MaxLOD=f32::MAX;
  let mut sampler=None;unsafe{device.CreateSamplerState(&sd,Some(&mut sampler))}.map_err(|e|e.to_string())?;
  Ok(Self{shader:shader.ok_or("SGSR1 shader missing")?,sampler:sampler.ok_or("SGSR1 sampler missing")?})
 }
 pub fn render(&self,ctx:&ID3D11DeviceContext,src:&ID3D11ShaderResourceView,dst:&ID3D11UnorderedAccessView,w:u32,h:u32){
  unsafe{ctx.CSSetShader(&self.shader,None);ctx.CSSetShaderResources(0,Some(&[Some(src.clone())]));ctx.CSSetSamplers(0,Some(&[Some(self.sampler.clone())]));
  let u=[Some(dst.clone())];ctx.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);
  ctx.Dispatch(w.div_ceil(8),h.div_ceil(8),1);
  ctx.CSSetShaderResources(0,Some(&[None]));ctx.CSSetSamplers(0,Some(&[None]));let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);}
 }
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn actual_sgsr_gpu_flat_edge_alpha_and_odd_sizes(){
  let g=crate::photo_lab::Gpu::new().unwrap();
  let flat=image::RgbaImage::from_pixel(37,19,image::Rgba([53,101,187,153]));
  let mut s=crate::spatial::NativeSpatialScaler::new(&g.device,73,37).unwrap();
  let tex=g.upload(&flat).unwrap();let out=g.read(&s.render(&g.device,&g.ctx,&tex,13).unwrap()).unwrap();assert_eq!(out.dimensions(),(73,37));assert!(out.pixels().all(|p|p.0==[53,101,187,153]));
  let edge=image::RgbaImage::from_fn(37,19,|x,y|{let v=if x>y+8{230}else{30};image::Rgba([v,v,v,153])});let tex=g.upload(&edge).unwrap();
  let base=g.read(&s.render(&g.device,&g.ctx,&tex,1).unwrap()).unwrap();
  let a=g.read(&s.render(&g.device,&g.ctx,&tex,13).unwrap()).unwrap();
  assert_ne!(a,base);assert!(a.pixels().all(|p|p[3]==153));assert_eq!(a.get_pixel(0,36).0,[30,30,30,153]);assert_eq!(a.get_pixel(72,0).0,[230,230,230,153]);
  let b=g.read(&s.render(&g.device,&g.ctx,&tex,13).unwrap()).unwrap();assert_eq!(a,b);
  eprintln!("SGSR1_GPU=PASS ODD_SIZE=73x37 ALPHA_EXACT=true FLAT_EXACT=true DISTINCT_FROM_BILINEAR=true");
 }
 #[test]fn sgsr_only_used_with_supported_upscale(){use crate::spatial::effective_mode;assert_eq!(effective_mode((1280,720),(1920,1080),13),13);assert_eq!(effective_mode((1280,720),(1280,720),13),0);assert_eq!(effective_mode((1280,720),(640,360),13),1);assert_eq!(effective_mode((640,360),(1920,1080),13),1);}
}
