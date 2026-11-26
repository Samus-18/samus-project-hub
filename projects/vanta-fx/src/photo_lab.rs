use std::{path::Path,ffi::c_void};
use windows::{core::Interface,Win32::{Foundation::HMODULE,Graphics::{
 Direct3D::{D3D_DRIVER_TYPE_HARDWARE,D3D_FEATURE_LEVEL_11_0},
 Direct3D11::{D3D11CreateDevice,D3D11_SDK_VERSION,D3D11_CREATE_DEVICE_BGRA_SUPPORT,D3D11_TEXTURE2D_DESC,D3D11_SUBRESOURCE_DATA,D3D11_MAPPED_SUBRESOURCE,
 D3D11_USAGE_DEFAULT,D3D11_USAGE_STAGING,D3D11_CPU_ACCESS_READ,D3D11_BIND_SHADER_RESOURCE,D3D11_BIND_UNORDERED_ACCESS,D3D11_MAP_READ,
 ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11Resource,ID3D11ComputeShader,ID3D11ClassLinkage,ID3D11UnorderedAccessView},
 Dxgi::{IDXGIAdapter,Common::{DXGI_FORMAT,DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_FORMAT_R32G32B32A32_FLOAT,DXGI_SAMPLE_DESC}}}
}};
#[derive(Clone,Debug,serde::Serialize)]
pub struct ScaleResult{pub name:String,pub psnr:Option<f64>,pub gpu_ms:Option<f64>}
pub struct ResultImage{pub before:image::RgbaImage,pub after:image::RgbaImage,pub histogram:[u32;32],pub changed:u64,pub mean_delta:f64,pub scales:Vec<ScaleResult>}
pub(crate) struct Gpu{pub(crate) device:ID3D11Device,pub(crate) ctx:ID3D11DeviceContext}
fn desc(w:u32,h:u32,format:DXGI_FORMAT)->D3D11_TEXTURE2D_DESC{
 D3D11_TEXTURE2D_DESC{Width:w,Height:h,MipLevels:1,ArraySize:1,Format:format,
 SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,
 BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,CPUAccessFlags:0,MiscFlags:0}
}
impl Gpu{
 pub(crate) fn new()->Result<Self,String>{
  let(mut device,mut ctx)=(None,None);
  unsafe{D3D11CreateDevice(None::<&IDXGIAdapter>,D3D_DRIVER_TYPE_HARDWARE,HMODULE::default(),D3D11_CREATE_DEVICE_BGRA_SUPPORT,
   Some(&[D3D_FEATURE_LEVEL_11_0]),D3D11_SDK_VERSION,Some(&mut device),None,Some(&mut ctx))}.map_err(|e|e.to_string())?;
  Ok(Self{device:device.ok_or("GPU missing")?,ctx:ctx.ok_or("GPU context missing")?})
 }
 fn texture(&self,d:&D3D11_TEXTURE2D_DESC,init:Option<&D3D11_SUBRESOURCE_DATA>)->Result<ID3D11Texture2D,String>{
  let mut t=None;unsafe{self.device.CreateTexture2D(d,init.map(|v|v as *const _),Some(&mut t))}.map_err(|e|e.to_string())?;
  t.ok_or("Texture missing".into())
 }
 pub(crate) fn upload(&self,img:&image::RgbaImage)->Result<ID3D11Texture2D,String>{
  let(w,h)=img.dimensions();self.texture(&desc(w,h,DXGI_FORMAT_R8G8B8A8_UNORM),Some(&D3D11_SUBRESOURCE_DATA{
   pSysMem:img.as_raw().as_ptr().cast(),SysMemPitch:w*4,SysMemSlicePitch:w*h*4}))
 }
 pub(crate) fn read(&self,texture:&ID3D11Texture2D)->Result<image::RgbaImage,String>{
  let mut d=D3D11_TEXTURE2D_DESC::default();unsafe{texture.GetDesc(&mut d);}
  let(w,h)=(d.Width,d.Height);
  if d.Format!=DXGI_FORMAT_R8G8B8A8_UNORM||w==0||h==0||w>1024||h>1024{return Err("Unsupported offline readback dimensions or format".into());}
  let staging=self.texture(&D3D11_TEXTURE2D_DESC{Usage:D3D11_USAGE_STAGING,BindFlags:0,CPUAccessFlags:D3D11_CPU_ACCESS_READ.0 as u32,..d},None)?;
  let a:ID3D11Resource=staging.cast().map_err(|e|e.to_string())?;let b:ID3D11Resource=texture.cast().map_err(|e|e.to_string())?;
  unsafe{self.ctx.CopyResource(&a,&b);}crate::native_filter::verify_gpu_completion(&self.device,&self.ctx)?;
  let mut mapped=D3D11_MAPPED_SUBRESOURCE::default();
  unsafe{self.ctx.Map(&a,0,D3D11_MAP_READ,0,Some(&mut mapped))}.map_err(|e|e.to_string())?;
  let data=if mapped.pData.is_null()||mapped.RowPitch<w*4{None}else{
   let mut v=vec![0u8;(w*h*4) as usize];
   for y in 0..h as usize{let row=unsafe{std::slice::from_raw_parts((mapped.pData as *const u8).add(y*mapped.RowPitch as usize),(w*4) as usize)};
   v[y*w as usize*4..(y+1)*w as usize*4].copy_from_slice(row);}Some(v)
  };
  unsafe{self.ctx.Unmap(&a,0);}
  image::RgbaImage::from_raw(w,h,data.ok_or("Invalid GPU readback")?).ok_or("Image shape mismatch".into())
 }
 fn lut(&self,input:&ID3D11Texture2D,cube:&crate::cube_lut::Cube)->Result<ID3D11Texture2D,String>{
  let n=cube.size;let data=cube.packed();let lut=self.texture(&desc(n*n,n,DXGI_FORMAT_R32G32B32A32_FLOAT),Some(&D3D11_SUBRESOURCE_DATA{
   pSysMem:data.as_ptr().cast(),SysMemPitch:n*n*16,SysMemSlicePitch:n*n*n*16}))?;
  let mut d=D3D11_TEXTURE2D_DESC::default();unsafe{input.GetDesc(&mut d);}
  d.BindFlags=(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32;
  let output=self.texture(&d,None)?;
  let(mut a,mut b,mut u)=(None,None,None);
  unsafe{self.device.CreateShaderResourceView(input,None,Some(&mut a))}.map_err(|e|e.to_string())?;
  unsafe{self.device.CreateShaderResourceView(&lut,None,Some(&mut b))}.map_err(|e|e.to_string())?;
  unsafe{self.device.CreateUnorderedAccessView(&output,None,Some(&mut u))}.map_err(|e|e.to_string())?;
  let bytes=crate::shadercheck::compile_native_hlsl(include_str!("../shaders/native/HB_CubePreview.hlsl"))?;
  let mut shader=None;unsafe{self.device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|e.to_string())?;
  let shader:ID3D11ComputeShader=shader.ok_or("LUT shader missing")?;
  if a.is_none()||b.is_none()||u.is_none(){return Err("LUT resource view missing".into());}
  unsafe{
   self.ctx.CSSetShader(&shader,None);self.ctx.CSSetShaderResources(0,Some(&[a,b]));
   self.ctx.CSSetUnorderedAccessViews(0,1,Some([u].as_ptr()),None);
   self.ctx.Dispatch(d.Width.div_ceil(8),d.Height.div_ceil(8),1);
   self.ctx.CSSetShaderResources(0,Some(&[None,None]));
   let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];self.ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
   self.ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
  }Ok(output)
 }
}
fn load(path:Option<&Path>)->Result<image::RgbaImage,String>{
 let img=if let Some(path)=path{
  if std::fs::metadata(path).map_err(|e|e.to_string())?.len()>32*1024*1024{return Err("Reference image exceeds 32 MiB".into());}
  let mut reader=image::ImageReader::open(path).map_err(|e|e.to_string())?.with_guessed_format().map_err(|e|e.to_string())?;
  let mut limits=image::Limits::default();limits.max_image_width=Some(4096);limits.max_image_height=Some(4096);limits.max_alloc=Some(96*1024*1024);
  reader.limits(limits);reader.decode().map_err(|e|e.to_string())?
 }else{image::load_from_memory(include_bytes!("../assets/hero_gameplay_v10.png")).map_err(|e|e.to_string())?};
 Ok(img.resize(768,768,image::imageops::FilterType::Lanczos3).to_rgba8())
}
pub fn histogram(img:&image::RgbaImage)->[u32;32]{
 let mut bins=[0;32];for p in img.pixels(){let y=(54*p[0] as u32+183*p[1] as u32+19*p[2] as u32)/256;bins[(y/8).min(31) as usize]+=1;}bins
}
fn difference(a:&image::RgbaImage,b:&image::RgbaImage)->(u64,f64,Option<f64>){
 let mut changed=0;let mut abs=0u64;let mut sq=0u64;let mut n=0u64;
 for (a,b) in a.pixels().zip(b.pixels()){let mut diff=false;for k in 0..3{let d=a[k].abs_diff(b[k]) as u64;diff|=d!=0;abs+=d;sq+=d*d;n+=1;}changed+=u64::from(diff);}
 let mse=sq as f64/n.max(1) as f64;
 (changed,abs as f64/n.max(1) as f64,if mse>0.{Some(10.*(65025./mse).log10())}else{None})
}
pub fn render(path:Option<&Path>,stages:&[usize],lighting:&crate::lighting::LightTuning,lut:Option<&crate::cube_lut::Cube>,benchmark:bool)->Result<ResultImage,String>{
 crate::native_stack::validate(stages)?;
 let original=load(path)?;let(w,h)=original.dimensions();
 let gpu=Gpu::new()?;let tex=gpu.upload(&original)?;
 let pipe=crate::native_pipeline::NativeGpuPipeline::new(&gpu.device,w,h)?;
 let mut srv=None;unsafe{gpu.device.CreateShaderResourceView(&tex,None,Some(&mut srv))}.map_err(|e|e.to_string())?;
 let mut tuning=lighting.clone();if lut.is_some(){tuning.lut.name=None;}
 let filtered=pipe.dispatch_with_cached_srv_tuned(&gpu.ctx,&tex,&srv.ok_or("Reference SRV missing")?,stages,&tuning)?;
 let final_tex=if let Some(cube)=lut{gpu.lut(&filtered,cube)?}else{filtered};
 let after=gpu.read(&final_tex)?;
 let (changed,mean_delta,_)=difference(&original,&after);
 let mut scales=Vec::new();
 if benchmark&&w>=32&&h>=32{
  // Same reference and 2:1 source for every algorithm; PSNR is a reference error metric, not visual ranking.
  let (ow,oh)=(w/2*2,h/2*2);
  let reference=image::imageops::resize(&original,ow,oh,image::imageops::FilterType::Lanczos3);
  let full=gpu.upload(&reference)?;
  let mut down=crate::spatial::NativeSpatialScaler::new(&gpu.device,ow/2,oh/2)?;
  let reduced=down.render_area(&gpu.device,&gpu.ctx,&full)?;
  let mut scaler=crate::spatial::NativeSpatialScaler::new(&gpu.device,ow,oh)?;
  for mode in 1..crate::spatial::MODES.len() as u8{
   let output=scaler.render(&gpu.device,&gpu.ctx,&reduced,mode)?;
   let reconstructed=gpu.read(&output)?;
   let (_,_,psnr)=difference(&reference,&reconstructed);
   let gpu_ms=crate::effect_bench::measure(&gpu.device,&gpu.ctx,||scaler.render(&gpu.device,&gpu.ctx,&reduced,mode).map(|_|()))?;
   scales.push(ScaleResult{name:crate::spatial::MODES[mode as usize].into(),psnr,gpu_ms});
  }
 }
 Ok(ResultImage{histogram:histogram(&after),before:original,after,changed,mean_delta,scales})
}
pub fn layer_proof()->Result<String,String>{
 let gpu=Gpu::new()?;
 let img=image::RgbaImage::from_fn(16,8,|x,y|image::Rgba([(x*17) as u8,(y*35) as u8,((x+y)*11) as u8,255]));
 let src=gpu.upload(&img)?;
 let pipe=crate::native_pipeline::NativeGpuPipeline::new(&gpu.device,16,8)?;
 let mut srv=None;unsafe{gpu.device.CreateShaderResourceView(&src,None,Some(&mut srv))}.map_err(|e|e.to_string())?;
 let srv=srv.ok_or("Proof SRV missing")?;
 let mut t=crate::lighting::LightTuning::default();
 let render=|s:&[usize],l:&crate::lighting::LightTuning|gpu.read(&pipe.dispatch_with_cached_srv_tuned(&gpu.ctx,&src,&srv,s,l)?);
 let full=render(&[7],&t)?;
 if full==img{return Err("Baseline shader inactive".into());}
 t.layers.strength.insert(7,0.);
 if render(&[7],&t)?!=img{return Err("Bypass not exact".into());}
 t.layers.strength.insert(7,0.5);
 let half=render(&[7],&t)?;
 for ((a,b),c) in img.pixels().zip(full.pixels()).zip(half.pixels()){
  for k in 0..3{if (c[k] as f32-(a[k] as f32+b[k] as f32)*0.5).abs()>1.01{return Err("Mix incorrect".into());}}
 }
 t.layers.strength.clear();t.layers.split=Some(0.5);
 let split=render(&[7],&t)?;
 for (x,y,p) in split.enumerate_pixels(){if p!=if x<8{img.get_pixel(x,y)}else{full.get_pixel(x,y)}{return Err("Split incorrect".into());}}
 t.layers.split=None;t.layers.strength.insert(7,0.);
 if render(&[7,2,7],&t)?!=render(&[2],&t)?{return Err("Mixed chain bypass incorrect".into());}
 t.layers=Default::default();t.layers.set_at(&[7,2,7],0,0.);
 if render(&[7,2,7],&t)?!=render(&[2,7],&Default::default())?{return Err("First duplicate bypass incorrect".into());}
 t.layers.set_at(&[7,2,7],0,1.);t.layers.set_at(&[7,2,7],2,0.);
 if render(&[7,2,7],&t)?!=render(&[7,2],&Default::default())?{return Err("Last duplicate bypass incorrect".into());}
 Ok("GPU_LAYER_MIX=PASS BYPASS_EXACT=true HALF_MIX=true SPLIT_EXACT=true ORDERED_BYPASS=true INSTANCE_EXACT=true".into())
}
pub fn proof()->Result<String,String>{
 let gpu=Gpu::new()?;
 let img=image::RgbaImage::from_fn(16,8,|x,y|image::Rgba([(x*17) as u8,(y*35) as u8,((x+y)*11) as u8,255]));
 let src=gpu.upload(&img)?;
 let identity=gpu.read(&gpu.lut(&src,&crate::cube_lut::fixture(false))?)?;
 if difference(&img,&identity).0!=0{return Err("GPU identity LUT differs".into());}
 let negative=gpu.read(&gpu.lut(&src,&crate::cube_lut::fixture(true))?)?;
 for (a,b) in img.pixels().zip(negative.pixels()){if (0..3).any(|k|a[k].abs_diff(255-b[k])>1){return Err("GPU inverse LUT inaccurate".into());}}
 let pipe=crate::native_pipeline::NativeGpuPipeline::new(&gpu.device,16,8)?;
 let mut srv=None;unsafe{gpu.device.CreateShaderResourceView(&src,None,Some(&mut srv))}.map_err(|e|e.to_string())?;
 let srv=srv.ok_or("Proof SRV missing")?;
 let mut light=crate::lighting::LightTuning::default();
 // Compare neutral and changed grade on exactly the same base stack.
 let base=gpu.read(&pipe.dispatch_with_cached_srv_tuned(&gpu.ctx,&src,&srv,&[2],&light)?)?;
 light.grade.enabled=true;light.grade.mix=0.;
 let zero=gpu.read(&pipe.dispatch_with_cached_srv_tuned(&gpu.ctx,&src,&srv,&[2],&light)?)?;
 if base!=zero{return Err("Zero grade not exact".into());}
 light.grade.mix=1.;light.grade.exposure=0.5;light.grade.temperature=0.4;light.grade.selective_sat=0.5;light.grade.red=0.08;
 let graded=gpu.read(&pipe.dispatch_with_cached_srv_tuned(&gpu.ctx,&src,&srv,&[2],&light)?)?;
 if difference(&base,&graded).0<32{return Err("Color grade inactive".into());}
 let reference=render(None,&[52,54,57],&light,None,true)?;
 if reference.scales.len()!=crate::spatial::MODES.len()-1||reference.scales.iter().any(|r|r.gpu_ms.is_none()){return Err("Reference scaler benchmark incomplete".into());}
 Ok(format!("PHOTO_LAB=PASS LUT_IDENTITY_EXACT=true LUT_INVERSE=true GRADE_ZERO_EXACT=true GRADE_CHANGED={} SCALERS=12 PIXELS_CHANGED={}\n{}",difference(&base,&graded).0,reference.changed,serde_json::to_string(&reference.scales).map_err(|e|e.to_string())?))
}
#[cfg(test)]mod tests{
 #[test]fn histogram_and_identity_reference(){
  let img=image::RgbaImage::from_pixel(4,4,image::Rgba([128,128,128,255]));
  assert_eq!(super::histogram(&img)[16],16);assert_eq!(super::difference(&img,&img),(0,0.,None));
 }
}
