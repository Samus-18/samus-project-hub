use serde::{Serialize,Deserialize};
use windows::{core::Interface,Win32::Graphics::{Direct3D11::*,Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_SAMPLE_DESC}}};
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]pub struct Options{pub mip_prefilter:bool,pub sharpness:f32,pub capture_diagnostic:bool,pub live_gpu_timing:bool,pub fused_color:bool,pub animated_grain:bool}
impl Default for Options{fn default()->Self{Self{mip_prefilter:false,sharpness:0.,capture_diagnostic:false,live_gpu_timing:false,fused_color:true,animated_grain:false}}}
impl Options{pub fn clamp(&mut self){self.sharpness=if self.sharpness.is_finite(){self.sharpness.clamp(0.,1.)}else{0.};}}
// Retain the underlying texture together with its view, preventing address reuse.
#[derive(Default)]pub struct ViewCache{entry:Option<(ID3D11Texture2D,ID3D11ShaderResourceView)>}
impl ViewCache{
 pub fn get(&mut self,device:&ID3D11Device,texture:&ID3D11Texture2D)->Result<ID3D11ShaderResourceView,String>{
  if self.entry.as_ref().is_none_or(|(t,_)|t.as_raw()!=texture.as_raw()){
   let mut view=None;unsafe{device.CreateShaderResourceView(texture,None,Some(&mut view))}.map_err(|e|e.to_string())?;
   self.entry=Some((texture.clone(),view.ok_or("Cached source view missing")?));
  }Ok(self.entry.as_ref().unwrap().1.clone())
 }
}
pub struct MipChain{resource:ID3D11Resource,srv:ID3D11ShaderResourceView,sampler:ID3D11SamplerState,shader:ID3D11ComputeShader,constants:ID3D11Buffer,desc:D3D11_TEXTURE2D_DESC}
impl MipChain{
 pub fn new(device:&ID3D11Device,input:&ID3D11Texture2D)->Result<Self,String>{
  let mut d=D3D11_TEXTURE2D_DESC::default();unsafe{input.GetDesc(&mut d);}
  if d.Width==0||d.Height==0||d.ArraySize!=1||d.SampleDesc.Count!=1{return Err("Unsupported mip input".into());}
  let flags=unsafe{device.CheckFormatSupport(d.Format)}.map_err(|e|e.to_string())?;
  if flags&D3D11_FORMAT_SUPPORT_MIP_AUTOGEN.0 as u32==0{return Err("Driver does not support automatic mipmaps for capture format".into());}
  let actual=d;
  d.MipLevels=0;d.Usage=D3D11_USAGE_DEFAULT;d.BindFlags=(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_RENDER_TARGET.0) as u32;
  d.CPUAccessFlags=0;d.MiscFlags=D3D11_RESOURCE_MISC_GENERATE_MIPS.0 as u32;
  let mut tex=None;unsafe{device.CreateTexture2D(&d,None,Some(&mut tex))}.map_err(|e|e.to_string())?;
  let tex=tex.ok_or("Mip texture missing")?;let resource=tex.cast().map_err(|e|e.to_string())?;
  let mut srv=None;unsafe{device.CreateShaderResourceView(&tex,None,Some(&mut srv))}.map_err(|e|e.to_string())?;
  let mut sampler=None;unsafe{device.CreateSamplerState(&D3D11_SAMPLER_DESC{Filter:D3D11_FILTER_MIN_MAG_MIP_LINEAR,
   AddressU:D3D11_TEXTURE_ADDRESS_CLAMP,AddressV:D3D11_TEXTURE_ADDRESS_CLAMP,AddressW:D3D11_TEXTURE_ADDRESS_CLAMP,
   MipLODBias:0.,MaxAnisotropy:1,ComparisonFunc:D3D11_COMPARISON_NEVER,BorderColor:[0.;4],MinLOD:0.,MaxLOD:16.},Some(&mut sampler))}.map_err(|e|e.to_string())?;
  Ok(Self{resource,srv:srv.ok_or("Mip SRV missing")?,sampler:sampler.ok_or("Mip sampler missing")?,shader:shader(device,include_str!("../shaders/native/HB_MipPrefilter.hlsl"))?,constants:constant(device)?,desc:actual})
 }
 pub fn compatible(&self,t:&ID3D11Texture2D)->bool{let mut d=D3D11_TEXTURE2D_DESC::default();unsafe{t.GetDesc(&mut d);}d.Width==self.desc.Width&&d.Height==self.desc.Height&&d.Format==self.desc.Format}
 pub fn render(&self,ctx:&ID3D11DeviceContext,input:&ID3D11Texture2D,out:&ID3D11UnorderedAccessView,w:u32,h:u32)->Result<(),String>{
  if !self.compatible(input){return Err("Mip input changed".into());}
  let source:ID3D11Resource=input.cast().map_err(|e|e.to_string())?;
  let lod=(self.desc.Width as f32/w.max(1) as f32).max(self.desc.Height as f32/h.max(1) as f32).max(1.).log2();
  let p=[lod,0.,0.,0.];let cb:ID3D11Resource=self.constants.cast().map_err(|e|e.to_string())?;
  unsafe{
   ctx.CopySubresourceRegion(&self.resource,0,0,0,0,&source,0,None);
   ctx.GenerateMips(&self.srv);
   ctx.UpdateSubresource(&cb,0,None,p.as_ptr().cast(),0,0);
   ctx.CSSetConstantBuffers(0,Some(&[Some(self.constants.clone())]));ctx.CSSetShader(&self.shader,None);
   ctx.CSSetShaderResources(0,Some(&[Some(self.srv.clone())]));ctx.CSSetSamplers(0,Some(&[Some(self.sampler.clone())]));
   let u=[Some(out.clone())];ctx.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);ctx.Dispatch(w.div_ceil(8),h.div_ceil(8),1);
   ctx.CSSetShaderResources(0,Some(&[None]));ctx.CSSetSamplers(0,Some(&[None]));ctx.CSSetConstantBuffers(0,Some(&[None]));
   let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
   ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
  }Ok(())
 }
}
fn shader(d:&ID3D11Device,code:&str)->Result<ID3D11ComputeShader,String>{
 let b=crate::shadercheck::compile_native_hlsl(code)?;let mut s=None;unsafe{d.CreateComputeShader(&b,None::<&ID3D11ClassLinkage>,Some(&mut s))}.map_err(|e|e.to_string())?;s.ok_or("Compute shader missing".into())
}
fn constant(d:&ID3D11Device)->Result<ID3D11Buffer,String>{
 let mut c=None;unsafe{d.CreateBuffer(&D3D11_BUFFER_DESC{ByteWidth:16,Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0},None,Some(&mut c))}.map_err(|e|e.to_string())?;c.ok_or("Constant buffer missing".into())
}
pub struct Sharpen{shader:ID3D11ComputeShader,constants:ID3D11Buffer,texture:ID3D11Texture2D,uav:ID3D11UnorderedAccessView,cache:ViewCache,size:(u32,u32)}
impl Sharpen{
 pub fn new(d:&ID3D11Device,w:u32,h:u32)->Result<Self,String>{
  let desc=D3D11_TEXTURE2D_DESC{Width:w,Height:h,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R8G8B8A8_UNORM,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,CPUAccessFlags:0,MiscFlags:0};
  let mut t=None;unsafe{d.CreateTexture2D(&desc,None,Some(&mut t))}.map_err(|e|e.to_string())?;let texture=t.ok_or("Sharpen output missing")?;
  let mut u=None;unsafe{d.CreateUnorderedAccessView(&texture,None,Some(&mut u))}.map_err(|e|e.to_string())?;
  Ok(Self{shader:shader(d,include_str!("../shaders/native/HB_ReconstructionSharpen.hlsl"))?,constants:constant(d)?,texture,uav:u.ok_or("Sharpen UAV missing")?,cache:Default::default(),size:(w,h)})
 }
 pub fn render(&mut self,d:&ID3D11Device,ctx:&ID3D11DeviceContext,input:&ID3D11Texture2D,strength:f32)->Result<ID3D11Texture2D,String>{
  if strength<=0.{return Ok(input.clone());}
  let src=self.cache.get(d,input)?;let c:ID3D11Resource=self.constants.cast().map_err(|e|e.to_string())?;
  let p=[strength.clamp(0.,1.),0.,0.,0.];unsafe{
   ctx.UpdateSubresource(&c,0,None,p.as_ptr().cast(),0,0);ctx.CSSetConstantBuffers(0,Some(&[Some(self.constants.clone())]));
   ctx.CSSetShader(&self.shader,None);ctx.CSSetShaderResources(0,Some(&[Some(src)]));
   let u=[Some(self.uav.clone())];ctx.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);ctx.Dispatch(self.size.0.div_ceil(8),self.size.1.div_ceil(8),1);
   ctx.CSSetShaderResources(0,Some(&[None]));ctx.CSSetConstantBuffers(0,Some(&[None]));
   let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
  }Ok(self.texture.clone())
 }
}
#[cfg(test)]mod tests{
 #[test]fn safe_off_defaults(){let mut o:super::Options=serde_json::from_str("{}").unwrap();assert!(!o.mip_prefilter&&!o.capture_diagnostic);o.sharpness=2.;o.clamp();assert_eq!(o.sharpness,1.);}
}
