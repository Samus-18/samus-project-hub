//! Optional sparse D3D11 timestamp sampling. No live CPU wait, flush, image readback or FPS estimates.
use std::time::{Duration,Instant};
use windows::Win32::Graphics::Direct3D11::{ID3D11Device,ID3D11DeviceContext,ID3D11Query,D3D11_QUERY_DESC,D3D11_QUERY_TIMESTAMP,D3D11_QUERY_TIMESTAMP_DISJOINT,D3D11_QUERY_DATA_TIMESTAMP_DISJOINT};
#[derive(Clone,Copy,Debug,PartialEq)]pub struct Reading{pub ms:f64,pub when:Instant}
pub struct Sampler{queries:Option<(ID3D11Query,ID3D11Query,ID3D11Query)>,pending:Option<Instant>,begun:bool,last_start:Option<Instant>,result:Option<Reading>,failed:bool}
impl Default for Sampler{fn default()->Self{Self{queries:None,pending:None,begun:false,last_start:None,result:None,failed:false}}}
fn duration_ms(freq:u64,a:u64,b:u64,disjoint:bool)->Option<f64>{
 if disjoint||freq==0||a==0||b<=a{return None;}
 let v=(b-a) as f64*1000./freq as f64;
 (v.is_finite()&&v>0.&&v<1000.).then_some(v)
}
impl Sampler{
 pub fn clear(&mut self){*self=Self::default();}
 pub fn reading(&self,now:Instant)->Option<Reading>{self.result.filter(|r|now.saturating_duration_since(r.when)<Duration::from_secs(4))}
 pub fn value(&self,now:Instant)->Option<f64>{self.reading(now).map(|r|r.ms)}
 pub fn failed(&self)->bool{self.failed}
 pub fn begin(&mut self,d:&ID3D11Device,ctx:&ID3D11DeviceContext,now:Instant){
  if self.failed||self.begun||self.pending.is_some()||self.last_start.is_some_and(|t|now.saturating_duration_since(t)<Duration::from_secs(1)){return;}
  if self.queries.is_none(){
   let create=|kind|->Result<ID3D11Query,String>{let mut q=None;unsafe{d.CreateQuery(&D3D11_QUERY_DESC{Query:kind,MiscFlags:0},Some(&mut q))}.map_err(|e|e.to_string())?;q.ok_or("Timestamp query unavailable".into())};
   let result=(||Ok::<_,String>((create(D3D11_QUERY_TIMESTAMP_DISJOINT)?,create(D3D11_QUERY_TIMESTAMP)?,create(D3D11_QUERY_TIMESTAMP)?)))();
   match result{Ok(q)=>self.queries=Some(q),Err(_)=>{self.failed=true;return;}}
  }
  if let Some((d,a,_))=&self.queries{unsafe{ctx.Begin(d);ctx.End(a);}self.begun=true;self.last_start=Some(now);}
 }
 pub fn end(&mut self,ctx:&ID3D11DeviceContext,now:Instant){
  if !self.begun{return;}
  if let Some((d,_,b))=&self.queries{unsafe{ctx.End(b);ctx.End(d);}self.pending=Some(now);}
  self.begun=false;
 }
 pub fn poll(&mut self,ctx:&ID3D11DeviceContext,now:Instant){
  let Some(start)=self.pending else{return;};
  if now.saturating_duration_since(start)>Duration::from_secs(3){self.queries=None;self.pending=None;self.result=None;return;}
  let Some((d,a,b))=&self.queries else{return;};
  let mut info=D3D11_QUERY_DATA_TIMESTAMP_DISJOINT::default();let(mut t0,mut t1)=(0u64,0u64);
  // GetData may map S_FALSE to Ok in windows-rs, so zero sentinels are essential.
  // DONOTFLUSH: never force driver execution just to obtain measurements.
  let read=|q:&ID3D11Query,p:*mut std::ffi::c_void,n:u32|unsafe{ctx.GetData(q,Some(p),n,1)};
  if read(d,(&mut info as *mut D3D11_QUERY_DATA_TIMESTAMP_DISJOINT).cast(),std::mem::size_of_val(&info) as u32).is_err()||
     read(a,(&mut t0 as *mut u64).cast(),8).is_err()||read(b,(&mut t1 as *mut u64).cast(),8).is_err(){self.failed=true;self.pending=None;self.result=None;return;}
  if info.Frequency==0||t0==0||t1==0{return;}
  self.pending=None;self.result=duration_ms(info.Frequency,t0,t1,info.Disjoint.as_bool()).map(|ms|Reading{ms,when:now});
 }
}
pub fn proof()->Result<String,String>{
 let g=crate::photo_lab::Gpu::new()?;let img=image::RgbaImage::from_fn(320,180,|x,y|image::Rgba([(x%256)as u8,(y%256)as u8,90,255]));
 let tex=g.upload(&img)?;let pipe=crate::native_pipeline::NativeGpuPipeline::new(&g.device,320,180)?;
 let mut p=Sampler::default();let n=Instant::now();p.begin(&g.device,&g.ctx,n);
 for _ in 0..8{pipe.dispatch(&g.device,&g.ctx,&tex,&[7,2])?;}
 p.end(&g.ctx,Instant::now());
 // Explicit offline proof may wait; the live sampler never calls this.
 crate::native_filter::verify_gpu_completion(&g.device,&g.ctx)?;p.poll(&g.ctx,Instant::now());
 let ms=p.value(Instant::now()).ok_or("Real asynchronous timestamp unavailable")?;
 if p.value(Instant::now()+Duration::from_secs(5)).is_some(){return Err("Stale GPU timing exposed".into());}
 Ok(format!("LIVE_GPU_TIMING=PASS GPU_MS={ms:.5} STALE_HIDDEN=true LIVE_NO_WAIT=true"))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn invalid_and_disjoint_timestamps_never_invent_values(){assert_eq!(duration_ms(1000,100,102,false),Some(2.));assert!(duration_ms(0,100,102,false).is_none());assert!(duration_ms(1000,100,100,false).is_none());assert!(duration_ms(1000,100,102,true).is_none());assert!(Sampler::default().value(Instant::now()).is_none());}
}
