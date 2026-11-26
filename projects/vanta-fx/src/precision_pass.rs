//! One optional linear-light finish with real FP16/FP32 temporary texture.
use serde::{Serialize,Deserialize};
use windows::Win32::Graphics::{Direct3D11::*,Dxgi::Common::*};
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]pub struct Options{pub enabled:bool,pub force_fp32:bool,pub exposure:f32,pub shoulder:f32,pub contrast:f32,pub saturation:f32,pub mix:f32,pub quant10:bool,pub dither:f32}
impl Default for Options{fn default()->Self{Self{enabled:false,force_fp32:false,exposure:0.,shoulder:0.35,contrast:1.,saturation:1.,mix:1.,quant10:false,dither:0.}}}
impl Options{
 pub fn clamp(&mut self){let d=Self::default();for (v,lo,hi,default) in [(&mut self.exposure,-2.,2.,d.exposure),(&mut self.shoulder,0.,1.,d.shoulder),(&mut self.contrast,0.5,1.5,d.contrast),(&mut self.saturation,0.,2.,d.saturation),(&mut self.mix,0.,1.,d.mix),(&mut self.dither,0.,1.,d.dither)]{*v=if v.is_finite(){v.clamp(lo,hi)}else{default};}}
 pub fn active(&self)->bool{self.enabled&&self.mix>0.}
 pub fn args(&self)->[f32;8]{[self.exposure,self.shoulder,self.contrast,self.saturation,self.mix,if self.quant10{1.}else{0.},self.dither,0.]}
 pub fn decode(s:&str)->Result<Self,String>{if s.len()>4096{return Err("Precision preset exceeds 4 KiB".into());}let mut v:Self=serde_json::from_str(s).map_err(|e|e.to_string())?;v.clamp();Ok(v)}
}
pub struct Pass{_temp:ID3D11Texture2D,srv:ID3D11ShaderResourceView,uav:ID3D11UnorderedAccessView,encode:ID3D11ComputeShader,resolve:ID3D11ComputeShader,constants:ID3D11Buffer,pub fp16:bool,forced:bool,w:u32,h:u32}
impl Pass{
 pub fn compatible(&self,forced:bool)->bool{self.forced==forced}
 pub fn new(d:&ID3D11Device,w:u32,h:u32,forced:bool)->Result<Self,String>{
  if w==0||h==0||u64::from(w)*u64::from(h)>8_294_400{return Err("Precision finish limited to 8.3 million pixels".into());}
  let make=|format:DXGI_FORMAT|->Result<(ID3D11Texture2D,ID3D11ShaderResourceView,ID3D11UnorderedAccessView),String>{
   let flags=unsafe{d.CheckFormatSupport(format)}.map_err(|e|e.to_string())?;
   if flags&(D3D11_FORMAT_SUPPORT_TYPED_UNORDERED_ACCESS_VIEW.0 as u32)==0{return Err("Typed UAV unsupported".into());}
   let desc=D3D11_TEXTURE2D_DESC{Width:w,Height:h,MipLevels:1,ArraySize:1,Format:format,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,CPUAccessFlags:0,MiscFlags:0};
   let mut t=None;unsafe{d.CreateTexture2D(&desc,None,Some(&mut t))}.map_err(|e|e.to_string())?;let t=t.ok_or("Precision texture missing")?;
   let(mut srv,mut uav)=(None,None);
   unsafe{d.CreateShaderResourceView(&t,None,Some(&mut srv))}.map_err(|e|e.to_string())?;unsafe{d.CreateUnorderedAccessView(&t,None,Some(&mut uav))}.map_err(|e|e.to_string())?;
   Ok((t,srv.ok_or("Precision SRV missing")?,uav.ok_or("Precision UAV missing")?))
  };
  let ((temp,srv,uav),fp16)=if !forced{match make(DXGI_FORMAT_R16G16B16A16_FLOAT){Ok(v)=>(v,true),Err(_)=>(make(DXGI_FORMAT_R32G32B32A32_FLOAT)?,false)}}else{(make(DXGI_FORMAT_R32G32B32A32_FLOAT)?,false)};
  let shader=|s:&str|->Result<ID3D11ComputeShader,String>{let bytes=crate::shadercheck::compile_native_hlsl(s)?;let mut sh=None;unsafe{d.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut sh))}.map_err(|e|e.to_string())?;sh.ok_or("Precision shader missing".into())};
  let mut constants=None;unsafe{d.CreateBuffer(&D3D11_BUFFER_DESC{ByteWidth:32,Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0},None,Some(&mut constants))}.map_err(|e|e.to_string())?;
  Ok(Self{_temp:temp,srv,uav,encode:shader(include_str!("../shaders/native/HB_PrecisionEncode.hlsl"))?,resolve:shader(include_str!("../shaders/native/HB_PrecisionResolve.hlsl"))?,constants:constants.ok_or("Precision constants missing")?,fp16,forced,w,h})
 }
 pub fn run(&self,ctx:&ID3D11DeviceContext,input:&ID3D11ShaderResourceView,target:&ID3D11UnorderedAccessView,o:&Options)->Result<(),String>{
  use windows::core::Interface;let cb:ID3D11Resource=self.constants.cast().map_err(|e|e.to_string())?;let args=o.args();
  let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];
  unsafe{
   ctx.UpdateSubresource(&cb,0,None,args.as_ptr().cast(),0,0);ctx.CSSetConstantBuffers(0,Some(&[Some(self.constants.clone())]));
   ctx.CSSetShader(&self.encode,None);ctx.CSSetShaderResources(0,Some(&[Some(input.clone())]));ctx.CSSetUnorderedAccessViews(0,1,Some([Some(self.uav.clone())].as_ptr()),None);ctx.Dispatch(self.w.div_ceil(8),self.h.div_ceil(8),1);
   ctx.CSSetShaderResources(0,Some(&[None,None]));ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
   ctx.CSSetShader(&self.resolve,None);ctx.CSSetShaderResources(0,Some(&[Some(self.srv.clone()),Some(input.clone())]));ctx.CSSetUnorderedAccessViews(0,1,Some([Some(target.clone())].as_ptr()),None);ctx.Dispatch(self.w.div_ceil(8),self.h.div_ceil(8),1);
   ctx.CSSetShaderResources(0,Some(&[None,None]));ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);ctx.CSSetConstantBuffers(0,Some(&[None]));
  }Ok(())
 }
}
pub fn proof()->Result<String,String>{
 let g=crate::photo_lab::Gpu::new()?;let im=image::RgbaImage::from_fn(64,32,|x,y|image::Rgba([(x*4) as u8,(y*8) as u8,((x+y)*2) as u8,255]));let source=g.upload(&im)?;
 let pipe=crate::native_pipeline::NativeGpuPipeline::new(&g.device,64,32)?;
 let mut view=None;unsafe{g.device.CreateShaderResourceView(&source,None,Some(&mut view))}.map_err(|e|e.to_string())?;let view=view.ok_or("Proof input SRV")?;
 let render=|o:&Options|->Result<image::RgbaImage,String>{let mut t=crate::lighting::LightTuning::default();t.layers.strength.insert(7,0.);t.precision=o.clone();g.read(&pipe.dispatch_with_cached_srv_tuned(&g.ctx,&source,&view,&[7],&t)?)};
 if render(&Options::default())?!=im{return Err("Precision off not exact".into());}
 let mut opt=Options{enabled:true,exposure:0.4,shoulder:0.3,contrast:1.05,saturation:0.9,..Default::default()};
 let a=render(&opt)?;let half=pipe.precision_fp16().ok_or("Precision backend missing")?;
 opt.force_fp32=true;let b=render(&opt)?;if pipe.precision_fp16()!=Some(false){return Err("FP32 forced backend ignored".into());}
 let diff=a.as_raw().iter().zip(b.as_raw()).map(|(a,b)|a.abs_diff(*b)).max().unwrap_or(0);
 if diff>1||a==im{return Err(format!("Precision comparison failed: delta {diff}"));}
 opt.quant10=true;opt.dither=1.;let d=render(&opt)?;let changed=d.pixels().zip(b.pixels()).filter(|(a,b)|a!=b).count();if changed==0{return Err("Quantize/dither inactive".into());}
 opt.mix=0.;if render(&opt)?!=im{return Err("Zero mix not exact".into());}
 Ok(format!("PRECISION_PASS=PASS AUTO_FP16={half} FORCED_FP32=true MAX_BYTE_DELTA={diff} DITHER_CHANGED={changed} OFF_ZERO_EXACT=true SDR_OUTPUT=true"))
}
#[cfg(test)]mod tests{use super::*;#[test]fn safe_preset_and_bounds(){let mut o=Options{exposure:99.,dither:f32::NAN,..Default::default()};o.clamp();assert_eq!(o.exposure,2.);assert_eq!(o.dither,0.);assert!(!o.active());let raw=serde_json::to_string(&o).unwrap();assert_eq!(Options::decode(&raw).unwrap(),o);assert!(Options::decode(&"x".repeat(4097)).is_err());}}
