//! Optional two-scale external flow estimator. Source and intermediate frames stay on GPU.
//! Not AMD FidelityFX Optical Flow, no engine vectors or game injection.
use windows::Win32::Graphics::{Direct3D11::*,Dxgi::Common::*};
pub const DOWN:&str=include_str!("../shaders/native/HB_FlowDown4.hlsl");
pub const COARSE:&str=include_str!("../shaders/native/HB_FlowCoarse.hlsl");
pub const REFINE:&str=include_str!("../shaders/native/HB_FlowTile.hlsl");
fn shader(d:&ID3D11Device,code:&str)->Result<ID3D11ComputeShader,String>{
 let b=crate::shadercheck::compile_native_hlsl(code)?;let mut v=None;
 unsafe{d.CreateComputeShader(&b,None::<&ID3D11ClassLinkage>,Some(&mut v))}.map_err(|e|e.to_string())?;
 v.ok_or("Pyramid shader missing".into())
}
struct Surface{srv:ID3D11ShaderResourceView,uav:ID3D11UnorderedAccessView}
impl Surface{
 fn new(d:&ID3D11Device,w:u32,h:u32,format:DXGI_FORMAT)->Result<Self,String>{
  let desc=D3D11_TEXTURE2D_DESC{Width:w,Height:h,MipLevels:1,ArraySize:1,Format:format,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0)as u32,CPUAccessFlags:0,MiscFlags:0};
  let mut tex=None;unsafe{d.CreateTexture2D(&desc,None,Some(&mut tex))}.map_err(|e|e.to_string())?;let tex=tex.ok_or("Pyramid surface missing")?;
  let mut srv=None;let mut uav=None;unsafe{d.CreateShaderResourceView(&tex,None,Some(&mut srv))}.map_err(|e|e.to_string())?;unsafe{d.CreateUnorderedAccessView(&tex,None,Some(&mut uav))}.map_err(|e|e.to_string())?;
  Ok(Self{srv:srv.ok_or("Pyramid SRV missing")?,uav:uav.ok_or("Pyramid UAV missing")?})
 }
}
pub struct Pyramid{low_size:(u32,u32),flow_size:(u32,u32),down:ID3D11ComputeShader,coarse:ID3D11ComputeShader,pub refine:ID3D11ComputeShader,old:Surface,now:Surface,seed:Surface}
impl Pyramid{
 pub fn new(d:&ID3D11Device,w:u32,h:u32,tile:u32)->Result<Self,String>{
  if w<16||h<16||w>8192||h>8192||!matches!(tile,16|32){return Err("Pyramid input outside supported dimensions".into());}
  let low_size=(w.div_ceil(4),h.div_ceil(4));let flow_size=crate::framegen::field_dimensions(w,h,tile);
  Ok(Self{down:shader(d,DOWN)?,coarse:shader(d,&format!("#define FLOW_TILE {tile}\n{COARSE}"))?,refine:shader(d,&format!("#define FLOW_TILE {tile}\n#define FLOW_PYRAMID 1\n{REFINE}"))?,
   old:Surface::new(d,low_size.0,low_size.1,DXGI_FORMAT_R8G8B8A8_UNORM)?,
   now:Surface::new(d,low_size.0,low_size.1,DXGI_FORMAT_R8G8B8A8_UNORM)?,
   seed:Surface::new(d,flow_size.0,flow_size.1,DXGI_FORMAT_R32G32B32A32_FLOAT)?,
   low_size,flow_size})
 }
 pub fn prepare(&self,ctx:&ID3D11DeviceContext,old:&ID3D11ShaderResourceView,now:&ID3D11ShaderResourceView){
  unsafe{
   ctx.CSSetShader(&self.down,None);ctx.CSSetShaderResources(0,Some(&[Some(old.clone()),Some(now.clone())]));
   let u=[Some(self.old.uav.clone()),Some(self.now.uav.clone())];ctx.CSSetUnorderedAccessViews(0,2,Some(u.as_ptr()),None);
   ctx.Dispatch(self.low_size.0.div_ceil(8),self.low_size.1.div_ceil(8),1);
   let clear=[None,None];ctx.CSSetUnorderedAccessViews(0,2,Some(clear.as_ptr()),None);ctx.CSSetShaderResources(0,Some(&[None,None]));
   ctx.CSSetShader(&self.coarse,None);ctx.CSSetShaderResources(0,Some(&[Some(self.old.srv.clone()),Some(self.now.srv.clone())]));
   let u=[Some(self.seed.uav.clone())];ctx.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);
   ctx.Dispatch(self.flow_size.0.div_ceil(8),self.flow_size.1.div_ceil(8),1);
   let clear=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
   ctx.CSSetShaderResources(0,Some(&[None,None]));ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
  }
 }
 pub fn seed(&self)->ID3D11ShaderResourceView{self.seed.srv.clone()}
}
pub fn compile_all()->Result<(),String>{
 crate::shadercheck::compile_native_hlsl(DOWN)?;
 for tile in [16,32]{
  crate::shadercheck::compile_native_hlsl(&format!("#define FLOW_TILE {tile}\n{COARSE}"))?;
  crate::shadercheck::compile_native_hlsl(&format!("#define FLOW_TILE {tile}\n#define FLOW_PYRAMID 1\n{REFINE}"))?;
 }
 Ok(())
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn shader_variants_compile(){compile_all().unwrap();}
 #[test]fn actual_gpu_pyramid_estimates_and_warps_with_alpha(){
  let g=crate::photo_lab::Gpu::new().unwrap();
  let image=|shift:u32|image::RgbaImage::from_fn(323,183,|x,y|{
   let x=(x+323-shift)%323;let l=((x*23+y*13+((x/5)^(y/7))*17)%180+30)as u8;
   image::Rgba([l,240-l/2,(l/2)+20,255])
  });
  let previous=g.upload(&image(0)).unwrap();let current=g.upload(&image(12)).unwrap();
  for tile in [16,32]{
   let fg=crate::framegen::FlowInterpolator::new_with_pyramid(&g.device,323,183,tile,true).unwrap();
   assert!(fg.pyramid_active());fg.save_real_frame(&g.ctx,&previous);
   let pair=fg.estimate(&g.device,&g.ctx,&current).unwrap();
   let out=g.read(fg.synth(&g.ctx,&pair,2,1).unwrap()).unwrap();
   assert_eq!(out.dimensions(),(323,183));assert!(out.pixels().all(|v|v[3]==255));
   assert!(out.pixels().zip(image(12).pixels()).filter(|(a,b)|a!=b).count()>300);
  }
  assert!(!crate::framegen::FlowInterpolator::new_with_pyramid(&g.device,323,183,16,false).unwrap().pyramid_active());
  eprintln!("PYRAMID_FLOW_GPU=PASS TILES_16_32 ODD_SOURCE=323x183");
 }
}
