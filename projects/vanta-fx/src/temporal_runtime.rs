//! Lazy optional capture-only temporal effect, no synthetic presents.
use std::time::{Duration,Instant};
use windows::Win32::Graphics::Direct3D11::{ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D};
use crate::{framegen::FlowInterpolator,lighting::LightTuning};
#[derive(Default)]pub struct Runtime{engine:Option<FlowInterpolator>,previous:Option<(Instant,LightTuning,usize)>,failed:bool}
impl Runtime{
 pub fn pause(&mut self){self.previous=None;}
 pub fn clear(&mut self){*self=Self::default();}
 pub fn render(&mut self,device:&ID3D11Device,ctx:&ID3D11DeviceContext,input:&ID3D11Texture2D,w:u32,h:u32,t:&LightTuning,passes:usize,now:Instant)->Result<ID3D11Texture2D,String>{
  if t.motion.temporal_mode==0||t.motion.temporal_strength<=0.||t.layers.split.is_some(){self.clear();return Ok(input.clone());}
  if self.failed{return Ok(input.clone());}
  let result=(||{
   if self.engine.is_none(){self.engine=Some(FlowInterpolator::new(device,w,h)?);}
   let e=self.engine.as_ref().ok_or("Temporal GPU resources unavailable")?;
   let ready=self.previous.as_ref().is_some_and(|(last,old,p)|now.saturating_duration_since(*last)<=Duration::from_millis(55)&&old==t&&*p==passes);
   let output=if ready{
    let pair=e.estimate(device,ctx,input)?;
    e.temporal_tuned(ctx,&pair,&t.motion)?.clone()
   }else{input.clone()};
   // Save unprocessed current source, never recursively filtered history.
   e.save_real_frame(ctx,input);self.previous=Some((now,t.clone(),passes));Ok(output)
  })();
  if result.is_err(){self.clear();self.failed=true;}
  result
 }
}
