//! Color-only multiscale light transport approximation on Horizon-owned D3D11 textures.
use serde::{Serialize,Deserialize};
use windows::{core::Interface,Win32::Graphics::{Direct3D11::*,Dxgi::Common::*}};
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]pub struct Options{
 pub enabled:bool,pub quality:u8,pub shadow:f32,pub bounce:f32,pub bloom:f32,pub sheen:f32,pub radius:f32,pub exposure:f32,pub shoulder:f32,pub mix:f32,pub protect_hud:bool
}
impl Default for Options{fn default()->Self{Self{enabled:false,quality:0,shadow:0.55,bounce:0.48,bloom:0.36,sheen:0.25,radius:0.55,exposure:0.25,shoulder:0.40,mix:0.78,protect_hud:true}}}
impl Options{
 pub fn clamp(&mut self){let d=Self::default();self.quality=self.quality.min(2);
  for(v,default)in[(&mut self.shadow,d.shadow),(&mut self.bounce,d.bounce),(&mut self.bloom,d.bloom),(&mut self.sheen,d.sheen),(&mut self.radius,d.radius),(&mut self.exposure,d.exposure),(&mut self.shoulder,d.shoulder),(&mut self.mix,d.mix)]{*v=if v.is_finite(){v.clamp(0.,1.)}else{default};}
 }
 pub fn active(&self)->bool{self.enabled&&self.mix>0.&&(self.shadow+self.bounce+self.bloom+self.sheen+self.exposure+self.shoulder)>0.}
 pub fn divisor(&self,w:u32,h:u32)->u32{let d=[8,4,2][self.quality.min(2) as usize];if u64::from(w)*u64::from(h)>2_073_600{d.max(4)}else{d}}
 pub fn preset(i:u8)->Self{let mut o=Self{enabled:true,..Default::default()};match i{
  1=>{o.shadow=0.78;o.bounce=0.40;o.bloom=0.16;o.exposure=0.12;o.mix=0.90;},
  2=>{o.shadow=0.32;o.bounce=0.64;o.bloom=0.65;o.sheen=0.42;o.radius=0.78;o.mix=0.84;},
  3=>{o.shadow=0.28;o.bounce=0.26;o.bloom=0.18;o.sheen=0.08;o.exposure=0.13;o.mix=0.52;},_=>{}}
  o
 }
 fn constants(&self,m:&crate::motion_options::Options,d:u32)->[f32;16]{let mut o=self.clone();o.clamp();let mut m=m.clone();m.clamp();
  [o.shadow,o.bounce,o.bloom,o.sheen,o.radius,o.exposure,o.shoulder,o.mix,m.left,m.top,m.right,m.bottom,if o.protect_hud&&m.mask{1.}else{0.},d as f32,0.,0.]
 }
}
struct Surface{_image:ID3D11Texture2D,srv:ID3D11ShaderResourceView,uav:ID3D11UnorderedAccessView}
pub struct Pass{field:Surface,transport:Surface,shaders:[ID3D11ComputeShader;3],cb:ID3D11Buffer,w:u32,h:u32,fw:u32,fh:u32,d:u32}
impl Pass{
 pub fn compatible(&self,d:u32)->bool{self.d==d}
 pub fn new(dev:&ID3D11Device,w:u32,h:u32,d:u32)->Result<Self,String>{
  if w==0||h==0||!matches!(d,2|4|8)||u64::from(w)*u64::from(h)>8_294_400{return Err("Cinema lighting resolution unsupported".into());}
  let fw=w.div_ceil(d);let fh=h.div_ceil(d);
  let surface=||->Result<Surface,String>{
   // FP32 field is only a fraction of output resolution, portable on FL11 hardware.
   let desc=D3D11_TEXTURE2D_DESC{Width:fw,Height:fh,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R32G32B32A32_FLOAT,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,CPUAccessFlags:0,MiscFlags:0};
   let mut image=None;unsafe{dev.CreateTexture2D(&desc,None,Some(&mut image))}.map_err(|e|e.to_string())?;let image=image.ok_or("Cinema field missing")?;
   let(mut srv,mut uav)=(None,None);unsafe{dev.CreateShaderResourceView(&image,None,Some(&mut srv))}.map_err(|e|e.to_string())?;unsafe{dev.CreateUnorderedAccessView(&image,None,Some(&mut uav))}.map_err(|e|e.to_string())?;
   Ok(Surface{_image:image,srv:srv.ok_or("Cinema SRV missing")?,uav:uav.ok_or("Cinema UAV missing")?})
  };
  let shader=|s:&str|->Result<ID3D11ComputeShader,String>{let bytes=crate::shadercheck::compile_native_hlsl(s)?;let mut sh=None;unsafe{dev.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut sh))}.map_err(|e|e.to_string())?;sh.ok_or("Cinema compute shader missing".into())};
  let mut cb=None;unsafe{dev.CreateBuffer(&D3D11_BUFFER_DESC{ByteWidth:64,Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0},None,Some(&mut cb))}.map_err(|e|e.to_string())?;
  Ok(Self{field:surface()?,transport:surface()?,shaders:[shader(include_str!("../shaders/native/HB_CinemaField.hlsl"))?,shader(include_str!("../shaders/native/HB_CinemaTransport.hlsl"))?,shader(include_str!("../shaders/native/HB_CinemaResolve.hlsl"))?],cb:cb.ok_or("Cinema constants missing")?,w,h,fw,fh,d})
 }
 pub fn run(&self,ctx:&ID3D11DeviceContext,input:&ID3D11ShaderResourceView,target:&ID3D11UnorderedAccessView,o:&Options,m:&crate::motion_options::Options)->Result<(),String>{
  let cb:ID3D11Resource=self.cb.cast().map_err(|e|e.to_string())?;let args=o.constants(m,self.d);
  unsafe{ctx.UpdateSubresource(&cb,0,None,args.as_ptr().cast(),0,0);ctx.CSSetConstantBuffers(0,Some(&[Some(self.cb.clone())]));}
  let dispatch=|sh:&ID3D11ComputeShader,srvs:&[Option<ID3D11ShaderResourceView>],uav:&ID3D11UnorderedAccessView,w:u32,h:u32|{
   unsafe{ctx.CSSetShader(sh,None);ctx.CSSetShaderResources(0,Some(srvs));ctx.CSSetUnorderedAccessViews(0,1,Some([Some(uav.clone())].as_ptr()),None);
    ctx.Dispatch(w.div_ceil(8),h.div_ceil(8),1);ctx.CSSetShaderResources(0,Some(&[None,None,None]));let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);}
  };
  dispatch(&self.shaders[0],&[Some(input.clone())],&self.field.uav,self.fw,self.fh);
  dispatch(&self.shaders[1],&[Some(self.field.srv.clone())],&self.transport.uav,self.fw,self.fh);
  dispatch(&self.shaders[2],&[Some(input.clone()),Some(self.field.srv.clone()),Some(self.transport.srv.clone())],target,self.w,self.h);
  unsafe{ctx.CSSetConstantBuffers(0,Some(&[None]));ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);}
  Ok(())
 }
}
pub fn proof()->Result<String,String>{
 let g=crate::photo_lab::Gpu::new()?;let (w,h)=(257,145);
 let im=image::RgbaImage::from_fn(w,h,|x,y|{
  if y>=116{return image::Rgba([200,(x%255)as u8,20,255]);}
  if (110..134).contains(&x)&&(28..61).contains(&y){return image::Rgba([252,250,246,255]);}
  if (40..95).contains(&x)&&(28..85).contains(&y){return image::Rgba([238,205,135,255]);}
  if (158..205).contains(&x)&&(20..90).contains(&y){return image::Rgba([60,210,240,255]);}
  let k=if x%48<8||y%40<8{45}else{92+((x+y)%38) as u8};image::Rgba([k,k,k,255])
 });
 let tex=g.upload(&im)?;let mut view=None;unsafe{g.device.CreateShaderResourceView(&tex,None,Some(&mut view))}.map_err(|e|e.to_string())?;let view=view.ok_or("Cinema proof SRV")?;
 let pipe=crate::native_pipeline::NativeGpuPipeline::new(&g.device,w,h)?;
 let mut t=crate::lighting::LightTuning::default();t.layers.strength.insert(7,0.);t.motion.mask=true;t.motion.top=0.8;
 let render=|t:&crate::lighting::LightTuning|g.read(&pipe.dispatch_with_cached_srv_tuned(&g.ctx,&tex,&view,&[7],t)?);
 if render(&t)?!=im{return Err("Cinema off not exact".into());}
 t.cinema=Options::preset(0);let a=render(&t)?;let changed=a.pixels().zip(im.pixels()).filter(|(a,b)|a!=b).count();
 if changed<500||a.enumerate_pixels().any(|(x,y,p)|y>=116&&p!=im.get_pixel(x,y)){return Err("Cinema output or HUD protection failed".into());}
 let mut checks=Vec::new();
 for q in 0..=2{t.cinema.quality=q;let b=render(&t)?;if b==im{return Err("Cinema quality inert".into());}checks.push(b);}
 if checks[0]==checks[1]||checks[1]==checks[2]{return Err("Cinema quality outputs duplicate".into());}
 let mut single=Options{enabled:true,mix:1.,shadow:0.,bounce:0.,bloom:0.,sheen:0.,exposure:0.,shoulder:0.,..Default::default()};
 for i in 0..6{let mut o=single.clone();match i{0=>o.shadow=1.,1=>o.bounce=1.,2=>o.bloom=1.,3=>o.sheen=1.,4=>o.exposure=1.,_=>o.shoulder=1.};t.cinema=o;if render(&t)?==im{return Err(format!("Cinema channel {i} inert"));}}
 single.mix=0.;t.cinema=single;if render(&t)?!=im{return Err("Cinema zero not exact".into());}
 t.cinema=Options::preset(0);let gpu=crate::effect_bench::measure(&g.device,&g.ctx,||pipe.dispatch_with_cached_srv_tuned(&g.ctx,&tex,&view,&[7],&t).map(|_|()))?;
 let mut times=Vec::new();
 for (bw,bh) in [(1280,720),(1920,1080)]{
  let fixture=image::RgbaImage::from_fn(bw,bh,|x,y|image::Rgba([(x%256)as u8,(y%256)as u8,160,255]));
  let texture=g.upload(&fixture)?;let mut view=None;unsafe{g.device.CreateShaderResourceView(&texture,None,Some(&mut view))}.map_err(|e|e.to_string())?;let view=view.ok_or("Cinema benchmark SRV")?;
  let p=crate::native_pipeline::NativeGpuPipeline::new(&g.device,bw,bh)?;
  for q in 0..=2{t.cinema.quality=q;let _=p.dispatch_with_cached_srv_tuned(&g.ctx,&texture,&view,&[7],&t)?;
   let ms=crate::effect_bench::measure(&g.device,&g.ctx,||p.dispatch_with_cached_srv_tuned(&g.ctx,&texture,&view,&[7],&t).map(|_|()))?;
   times.push((bw,bh,q,ms));}
 }
 Ok(format!("CINEMA_LIGHT=PASS CHANGED={changed} HUD_EXACT=true OFF_ZERO_EXACT=true CHANNELS=6 QUALITY_LEVELS=3 GPU_MS={gpu:?} SOURCE_COLOR_ONLY=true BENCHMARKS={times:?}"))
}
#[cfg(test)]mod tests{use super::*;#[test]fn defaults_and_bounded_quality(){let mut o=Options::default();assert!(!o.active());o.enabled=true;o.shadow=f32::NAN;o.quality=9;o.clamp();assert!(o.active());assert_eq!(o.quality,2);assert_eq!(o.divisor(1920,1080),2);assert_eq!(o.divisor(3840,2160),4);let a:Options=serde_json::from_str(&serde_json::to_string(&o).unwrap()).unwrap();assert_eq!(o,a);}}

pub fn reference_proof()->Result<String,String>{
 // The existing packaged game reference, never a fresh desktop/game capture.
 let mut t=crate::lighting::LightTuning::default();t.bypass_look=true;t.cinema=Options::preset(0);
 let res=crate::photo_lab::render(None,&[7],&t,None,false)?;
 if res.changed==0||res.before==res.after{return Err("Reference lighting inert".into());}
 let out=std::path::PathBuf::from("build-tests").join("cinema-radiance-v64");
 std::fs::create_dir_all(&out).map_err(|e|e.to_string())?;
 res.before.save(out.join("reference-before.png")).map_err(|e|e.to_string())?;
 res.after.save(out.join("reference-after.png")).map_err(|e|e.to_string())?;
 let rows=crate::effect_bench::run_lighting(&[7],&t)?;
 if rows.len()!=6||rows.iter().any(|r|r.gpu_ms.is_none()){return Err("Lighting benchmark unavailable".into());}
 Ok(format!("CINEMA_REFERENCE=PASS CHANGED={} MEAN_DELTA={:.3} TIMING_ROWS=6 BUNDLED_REFERENCE=true\n{}",res.changed,res.mean_delta,serde_json::to_string(&rows).map_err(|e|e.to_string())?))
}
