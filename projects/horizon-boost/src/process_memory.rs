use std::ffi::c_void;
#[repr(C)]#[derive(Default)]
struct Counters{cb:u32,faults:u32,peak_ws:usize,ws:usize,peak_paged:usize,paged:usize,
 peak_nonpaged:usize,nonpaged:usize,pagefile:usize,peak_pagefile:usize,private:usize}
#[link(name="kernel32")]
unsafe extern "system"{
 fn GetCurrentProcess()->*mut c_void;
 fn K32GetProcessMemoryInfo(process:*mut c_void,counters:*mut Counters,size:u32)->i32;
}
#[derive(Clone,Copy,Debug)]
pub struct Memory{pub working_mib:f64,pub peak_mib:f64,pub private_mib:f64,pub faults:u32}
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum Budget { Under30MiB, Above30MiB }
impl Memory {
 pub fn classify(&self)->Budget{if self.working_mib.is_finite()&&self.working_mib>0.&&self.working_mib<30.{Budget::Under30MiB}else{Budget::Above30MiB}}
 pub fn report(&self)->String{format!("Own process measured: working {:.1} MiB, private {:.1} MiB, peak {:.1} MiB. Target <30 MiB: {} (not a memory cap).",
 self.working_mib,self.private_mib,self.peak_mib,if self.classify()==Budget::Under30MiB{"yes"}else{"no"})}
}
pub fn read()->Result<Memory,String>{
 let size=std::mem::size_of::<Counters>() as u32;
 let mut c=Counters{cb:size,..Default::default()};
 if unsafe{K32GetProcessMemoryInfo(GetCurrentProcess(),&mut c,size)}==0{return Err(std::io::Error::last_os_error().to_string());}
 let mib=|v:usize|v as f64/1048576.;
 Ok(Memory{working_mib:mib(c.ws),peak_mib:mib(c.peak_ws),private_mib:mib(c.private),faults:c.faults})
}
#[cfg(test)]mod tests{#[test]fn actual_self_memory(){let m=super::read().unwrap();assert!(m.working_mib>0.&&m.peak_mib>=m.working_mib&&m.private_mib>0.);}}

#[cfg(test)]mod v101_target_tests{
 use super::*;
 #[test]fn numeric_classification_does_not_invent_ram_savings(){
  let m=Memory{working_mib:29.9,peak_mib:50.,private_mib:45.,faults:0};
  assert_eq!(m.classify(),Budget::Under30MiB);
  assert_eq!(Memory{working_mib:30.,..m}.classify(),Budget::Above30MiB);
  assert_eq!(Memory{working_mib:f64::NAN,..m}.classify(),Budget::Above30MiB);
  assert!(m.report().contains("not a memory cap"));
 }
}
