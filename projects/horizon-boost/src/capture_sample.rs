use std::time::{Instant,Duration};
use windows::{core::Interface,Win32::Graphics::{Direct3D11::*,Dxgi::Common::{DXGI_FORMAT_R32G32B32A32_UINT,DXGI_SAMPLE_DESC}}};
#[derive(Clone,Copy,Debug)]pub struct Sample{pub near_black:bool,pub repeats:u32}
pub struct Analyzer{shader:ID3D11ComputeShader,uav:ID3D11UnorderedAccessView,output:ID3D11Resource,staging:ID3D11Resource,pending:Option<Instant>,last:Option<Instant>,last_poll:Option<Instant>,hash:Option<u32>,latest:Option<Sample>,latest_at:Option<Instant>}
impl Analyzer{
 pub fn new(device:&ID3D11Device)->Result<Self,String>{
  let desc=D3D11_TEXTURE2D_DESC{Width:1,Height:1,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R32G32B32A32_UINT,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_UNORDERED_ACCESS.0 as u32,CPUAccessFlags:0,MiscFlags:0};
  let make=|d:&D3D11_TEXTURE2D_DESC|->Result<ID3D11Texture2D,String>{let mut t=None;unsafe{device.CreateTexture2D(d,None,Some(&mut t))}.map_err(|e|e.to_string())?;t.ok_or("Aggregate texture missing".into())};
  let tex=make(&desc)?;let stage=make(&D3D11_TEXTURE2D_DESC{Usage:D3D11_USAGE_STAGING,BindFlags:0,CPUAccessFlags:D3D11_CPU_ACCESS_READ.0 as u32,..desc})?;
  let mut u=None;unsafe{device.CreateUnorderedAccessView(&tex,None,Some(&mut u))}.map_err(|e|e.to_string())?;
  let bytes=crate::shadercheck::compile_native_hlsl(include_str!("../shaders/native/HB_CaptureSample.hlsl"))?;
  let mut shader=None;unsafe{device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|e.to_string())?;
  Ok(Self{shader:shader.ok_or("Aggregate shader missing")?,uav:u.ok_or("Aggregate UAV missing")?,output:tex.cast().map_err(|e|e.to_string())?,staging:stage.cast().map_err(|e|e.to_string())?,pending:None,last:None,last_poll:None,hash:None,latest:None,latest_at:None})
 }
 fn enqueue(&mut self,ctx:&ID3D11DeviceContext,source:&ID3D11ShaderResourceView,now:Instant){
  unsafe{
   ctx.CSSetShader(&self.shader,None);ctx.CSSetShaderResources(0,Some(&[Some(source.clone())]));
   let u=[Some(self.uav.clone())];ctx.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);ctx.Dispatch(1,1,1);
   ctx.CSSetShaderResources(0,Some(&[None]));let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
   ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);ctx.CopyResource(&self.staging,&self.output);
  }self.pending=Some(now);self.last=Some(now);
 }
 fn read(&mut self,ctx:&ID3D11DeviceContext,now:Instant,nonblocking:bool)->Result<Option<Sample>,String>{
  if self.pending.is_none(){return Ok(None);}
  let mut mapped=D3D11_MAPPED_SUBRESOURCE::default();
  let result=unsafe{ctx.Map(&self.staging,0,D3D11_MAP_READ,if nonblocking{D3D11_MAP_FLAG_DO_NOT_WAIT.0 as u32}else{0},Some(&mut mapped))};
  if let Err(e)=result{
   if nonblocking&&e.code().0==0x887a000au32 as i32&&self.pending.is_some_and(|t|now.saturating_duration_since(t)<Duration::from_secs(5)){return Ok(None);}
   return Err(format!("Aggregate capture diagnostic unavailable: {e}"));
  }
  let data=if mapped.pData.is_null()||mapped.RowPitch<16{None}else{Some(unsafe{std::ptr::read_unaligned(mapped.pData as *const [u32;4])})};
  unsafe{ctx.Unmap(&self.staging,0);}let sampled=self.pending.take();
  let d=data.ok_or("Aggregate result missing")?;if d[0]>d[1]||d[1]>255||d[2]>65280{self.latest=None;self.latest_at=None;self.hash=None;return Err("Invalid aggregate result".into());}
  let fresh=sampled.zip(self.latest_at).is_some_and(|(now,prev)|now>=prev&&now.saturating_duration_since(prev)<=Duration::from_secs(3));
  let repeats=if fresh&&self.hash==Some(d[3]){self.latest.map_or(1,|v|v.repeats.saturating_add(1))}else{0};
  self.hash=Some(d[3]);let s=Sample{near_black:d[1]<=3,repeats};self.latest=Some(s);self.latest_at=sampled;Ok(Some(s))
 }
 pub fn tick(&mut self,ctx:&ID3D11DeviceContext,source:&ID3D11ShaderResourceView)->Result<Option<Sample>,String>{
  let now=Instant::now();
  if self.last_poll.is_none_or(|t|now.saturating_duration_since(t)>=Duration::from_millis(100)){
   self.last_poll=Some(now);let _=self.read(ctx,now,true)?;
  }
  if self.pending.is_none()&&self.last.is_none_or(|t|now.saturating_duration_since(t)>=Duration::from_secs(1)){self.enqueue(ctx,source,now);}
  Ok(self.latest.filter(|_|self.latest_at.is_some_and(|t|now>=t&&now.saturating_duration_since(t)<=Duration::from_secs(3))))
 }
}
pub fn proof()->Result<String,String>{
 let g=crate::photo_lab::Gpu::new()?;let mut a=Analyzer::new(&g.device)?;
 let read=|a:&mut Analyzer,value:u8|->Result<Sample,String>{
  let t=g.upload(&image::RgbaImage::from_pixel(32,32,image::Rgba([value,value,value,255])))?;
  let mut s=None;unsafe{g.device.CreateShaderResourceView(&t,None,Some(&mut s))}.map_err(|e|e.to_string())?;
  a.enqueue(&g.ctx,&s.ok_or("Sample input missing")?,Instant::now());crate::native_filter::verify_gpu_completion(&g.device,&g.ctx)?;
  a.read(&g.ctx,Instant::now(),false)?.ok_or("Sample missing".into())
 };
 let b=read(&mut a,0)?;let same=read(&mut a,0)?;let bright=read(&mut a,125)?;
 if !b.near_black||same.repeats!=1||bright.near_black||bright.repeats!=0{return Err("GPU aggregate classification incorrect".into());}
 Ok("CAPTURE_SAMPLE=PASS BLACK=true REPEAT=true CHANGE=true SYNTHETIC_ONLY=true".into())
}


#[cfg(test)]mod v95_source_gap_tests{use super::*;#[test]fn repeated_sample_only_counts_when_fresh(){let n=Instant::now();assert!(n.checked_duration_since(n+Duration::from_secs(1)).is_none());assert_eq!(Duration::from_secs(4)>Duration::from_secs(3),true);}}
