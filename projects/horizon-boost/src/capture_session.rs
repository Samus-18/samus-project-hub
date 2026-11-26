//! Explicit recording session policy and local bookmarks.
use std::{path::{Path,PathBuf},time::{Instant,Duration}};
#[link(name="kernel32")]unsafe extern "system"{
 fn OpenProcess(access:u32,inherit:i32,pid:u32)->*mut std::ffi::c_void;
 fn WaitForSingleObject(handle:*mut std::ffi::c_void,ms:u32)->u32;
 fn CloseHandle(handle:*mut std::ffi::c_void)->i32;
}
struct GameHandle(isize);
impl GameHandle{
 fn open(pid:u32)->Result<Self,String>{let h=unsafe{OpenProcess(0x00100000,0,pid)};if h.is_null(){Err("Cannot observe selected game exit".into())}else{Ok(Self(h as isize))}}
 fn exited(&self)->Option<bool>{match unsafe{WaitForSingleObject(self.0 as *mut std::ffi::c_void,0)}{0=>Some(true),258=>Some(false),_=>None}}
}
impl Drop for GameHandle{fn drop(&mut self){unsafe{CloseHandle(self.0 as *mut std::ffi::c_void);}}}
#[derive(Default)]
pub struct Session{game:Option<GameHandle>,path:Option<PathBuf>,clip:crate::clip_book::Clip,limit:u64,last:Option<Instant>,pub bytes:u64,pub reason:Option<String>}
pub fn limit_mib(v:u32)->u32{v.clamp(256,16384)}
fn stop_reason(bytes:u64,limit:u64,exit:Option<bool>,elapsed:f64)->Option<&'static str>{
 if exit==Some(true){Some("Selected game exited")}else if bytes>=limit{Some("Recording video size threshold reached")}else if elapsed>=86400.{Some("24-hour session limit reached")}else{None}
}
impl Session{
 pub fn prepare(pid:Option<u32>,stop_on_exit:bool,limit:u32,game:String)->Result<Self,String>{
  let handle=if stop_on_exit{Some(GameHandle::open(pid.ok_or("Select a running game for stop-on-exit")?)?)}else{None};
  Ok(Self{game:handle,limit:u64::from(limit_mib(limit))*1048576,clip:crate::clip_book::Clip{game,..Default::default()},..Default::default()})
 }
 pub fn begin(&mut self,p:PathBuf){self.path=Some(p);}
 pub fn add_marker(&mut self,elapsed:f64,label:&str)->bool{self.path.is_some()&&self.clip.add(elapsed,elapsed,label)}
 pub fn markers(&self)->&[crate::clip_book::Marker]{&self.clip.markers}
 pub fn poll(&mut self,now:Instant,elapsed:f64)->Option<&'static str>{
  let p=self.path.as_ref()?;if self.last.is_some_and(|t|now.saturating_duration_since(t)<Duration::from_secs(1)){return None;}
  self.last=Some(now);if let Ok(v)=p.metadata(){self.bytes=v.len();}
  if crate::suite_finish::free_bytes(p).is_some_and(|n|n<512*1024*1024){return Some("Capture disk below 512 MiB free; finalizing");}
  let exit=self.game.as_ref().and_then(GameHandle::exited);
  stop_reason(self.bytes,self.limit,exit,elapsed)
 }
 pub fn metadata(&self,result:Option<&Path>)->Vec<(String,crate::clip_book::Clip)>{
  let mut out=Vec::new();
  for p in self.path.iter().map(|p|p.as_path()).chain(result){
   if p.is_file(){if let Some(k)=crate::clip_book::key(p){if !out.iter().any(|(v,_)|v==&k){out.push((k,self.clip.clone()));}}}
  }out
 }
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn sampled_recording_policy(){assert_eq!(limit_mib(1),256);assert_eq!(stop_reason(100,100,None,3.),Some("Recording video size threshold reached"));assert!(stop_reason(99,100,Some(false),3.).is_none());assert_eq!(stop_reason(1,100,Some(true),1.),Some("Selected game exited"));assert!(stop_reason(1,100,None,86400.).is_some());}
 #[test]fn manual_elapsed_markers_require_session(){let mut s=Session::prepare(None,false,256,"game".into()).unwrap();assert!(!s.add_marker(1.,"x"));s.begin("test.mkv".into());assert!(s.add_marker(1.5,"boss"));assert_eq!(s.markers()[0].ms,1500);}
 #[test]fn exit_watch_requires_selected_game(){assert!(Session::prepare(None,true,256,"g".into()).is_err());}
}
