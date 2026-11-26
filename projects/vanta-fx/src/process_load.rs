use std::{ffi::c_void,time::{Instant,Duration}};
type Handle=*mut c_void;
#[repr(C)]#[derive(Default)]struct FileTime{low:u32,high:u32}
impl FileTime{fn ticks(&self)->u64{(self.high as u64)<<32|self.low as u64}}
#[repr(C)]#[derive(Default)]struct Counters{cb:u32,faults:u32,peak_ws:usize,ws:usize,peak_paged:usize,paged:usize,peak_nonpaged:usize,nonpaged:usize,pagefile:usize,peak_pagefile:usize,private:usize}
#[link(name="kernel32")]unsafe extern "system"{
 fn OpenProcess(access:u32,inherit:i32,pid:u32)->Handle;
 fn CloseHandle(h:Handle)->i32;
 fn GetProcessTimes(h:Handle,creation:*mut FileTime,exit:*mut FileTime,kernel:*mut FileTime,user:*mut FileTime)->i32;
 fn K32GetProcessMemoryInfo(h:Handle,c:*mut Counters,size:u32)->i32;
}
struct Process(Handle);impl Drop for Process{fn drop(&mut self){unsafe{CloseHandle(self.0);}}}
#[derive(Clone,Copy)]struct Raw{creation:u64,ticks:u64,at:Instant,ram:Option<f64>}
fn read(pid:u32)->Option<Raw>{
 if pid==0{return None;}let p=Process(unsafe{OpenProcess(0x1000,0,pid)});if p.0.is_null(){std::mem::forget(p);return None;}
 let(mut c,mut e,mut k,mut u)=(FileTime::default(),FileTime::default(),FileTime::default(),FileTime::default());
 if unsafe{GetProcessTimes(p.0,&mut c,&mut e,&mut k,&mut u)}==0{return None;}
 let size=std::mem::size_of::<Counters>() as u32;let mut mem=Counters{cb:size,..Default::default()};
 let ram=if unsafe{K32GetProcessMemoryInfo(p.0,&mut mem,size)}!=0{Some(mem.ws as f64/1048576.)}else{None};
 Some(Raw{creation:c.ticks(),ticks:k.ticks().saturating_add(u.ticks()),at:Instant::now(),ram})
}
#[derive(Clone,Copy,Debug)]pub struct Sample{pub cpu:Option<f64>,pub ram:Option<f64>}
fn delta(old:Raw,new:Raw,cores:usize)->Option<f64>{
 if old.creation!=new.creation{return None;}
 let dt=new.at.checked_duration_since(old.at)?.as_secs_f64();
 if !(0.25..=10.).contains(&dt){return None;}
 let used=new.ticks.checked_sub(old.ticks)? as f64/10_000_000.;
 let p=100.*used/(dt*cores.max(1) as f64);
 (p.is_finite()&&p<=105.).then_some(p.clamp(0.,100.))
}
#[derive(Default)]pub struct Meter{
 own:Option<Raw>,game:Option<(u32,Raw)>,last:Option<Instant>,
 pub own_sample:Option<Sample>,pub game_sample:Option<Sample>
}
impl Meter{
 pub fn poll(&mut self,pid:Option<u32>,now:Instant){
  if self.game.as_ref().map(|v|v.0)!=pid{self.game=None;self.game_sample=None;}
  if self.last.is_some_and(|t|now.saturating_duration_since(t)<Duration::from_secs(1)){return;}
  self.last=Some(now);
  let cores=std::thread::available_parallelism().map(|v|v.get()).unwrap_or(1);
  let own=read(std::process::id());
  self.own_sample=own.map(|x|Sample{cpu:self.own.and_then(|o|delta(o,x,cores)),ram:x.ram});self.own=own;
  let game=pid.filter(|p|*p!=std::process::id()).and_then(|p|read(p).map(|r|(p,r)));
  self.game_sample=game.map(|(p,r)|Sample{cpu:self.game.filter(|x|x.0==p).and_then(|x|delta(x.1,r,cores)),ram:r.ram});
  self.game=game;
 }
}
pub fn proof()->Result<String,String>{
 let first=read(std::process::id()).ok_or("Self process query failed")?;
 std::thread::sleep(Duration::from_millis(300));
 let second=read(std::process::id()).ok_or("Second process query failed")?;
 let cpu=delta(first,second,std::thread::available_parallelism().map(|v|v.get()).unwrap_or(1)).ok_or("Invalid CPU delta")?;
 Ok(format!("PROCESS_LOAD=PASS CPU_PERCENT={cpu:.3} RAM_QUERY={} NO_GAME_ACCESS=true",second.ram.is_some()))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn process_delta_identity_and_bounds(){let t=Instant::now();let a=Raw{creation:1,ticks:0,at:t,ram:None};let b=Raw{ticks:5_000_000,at:t+Duration::from_secs(1),..a};assert_eq!(delta(a,b,2),Some(25.));assert!(delta(a,Raw{creation:2,..b},2).is_none());assert!(delta(a,Raw{at:t,..b},2).is_none());}
}
