//! Keep a stable GDI overlay idle unless its geometry or drawing changes.
use std::time::{Duration,Instant};
use crate::crosshair::CrosshairOptions;
#[derive(Default)]pub struct State{last:Option<((i32,i32),CrosshairOptions)>,z:Option<Instant>,visible:bool}
#[derive(Debug,PartialEq,Eq)]pub struct Update{pub alpha:bool,pub position:bool,pub paint:bool}
impl State{
 pub fn clear(&mut self){*self=Self::default();}
 pub fn observe(&mut self,pos:(i32,i32),opt:&CrosshairOptions,visible:bool,now:Instant)->Update{
  let reappeared=visible&&!self.visible;
  let alpha=self.last.as_ref().is_none_or(|(_,old)|old.opacity!=opt.opacity)||!visible||reappeared;
  let paint=self.last.as_ref().is_none_or(|(_,old)|old!=opt)||!visible||reappeared;
  let position=!visible||reappeared||self.last.as_ref().is_none_or(|(p,_)|*p!=pos)||self.z.is_none_or(|t|now.saturating_duration_since(t)>=Duration::from_secs(1));
  if position{self.z=Some(now);}
  self.last=Some((pos,opt.clone()));self.visible=visible;Update{alpha,position,paint}
 }
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn static_reticle_does_not_repaint_or_reposition(){let n=Instant::now();let mut s=State::default();let mut opt=CrosshairOptions::default();assert_eq!(s.observe((0,0),&opt,false,n),Update{alpha:true,position:true,paint:true});assert_eq!(s.observe((0,0),&opt,true,n+Duration::from_millis(100)),Update{alpha:true,position:true,paint:true});
  assert_eq!(s.observe((0,0),&opt,true,n+Duration::from_millis(150)),Update{alpha:false,position:false,paint:false});assert!(s.observe((0,0),&opt,true,n+Duration::from_millis(1100)).position);opt.size+=1;assert!(s.observe((0,0),&opt,true,n+Duration::from_millis(1150)).paint);assert!(s.observe((10,0),&opt,true,n+Duration::from_millis(1200)).position);s.clear();assert!(s.observe((10,0),&opt,true,n).paint);}
}
