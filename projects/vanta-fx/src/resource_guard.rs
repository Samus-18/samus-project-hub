//! Read-only Windows physical-memory diagnostics; no process priority, cleanup, or OS changes.
use std::mem::size_of;
#[repr(C)]
#[derive(Clone,Copy)]
struct MemoryStatusEx{
 length:u32, load:u32, total_phys:u64, available_phys:u64,
 total_page:u64, available_page:u64, total_virtual:u64,
 available_virtual:u64, extended:u64
}
#[link(name="kernel32")]
unsafe extern "system"{fn GlobalMemoryStatusEx(status:*mut MemoryStatusEx)->i32;}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct PhysicalMemory{pub free_mib:u64,pub total_mib:u64,pub system_load:u32}
pub fn read()->Option<PhysicalMemory>{
 let mut info=MemoryStatusEx{length:size_of::<MemoryStatusEx>()as u32,load:0,total_phys:0,
  available_phys:0,total_page:0,available_page:0,total_virtual:0,available_virtual:0,extended:0};
 let ok=unsafe{GlobalMemoryStatusEx(&mut info)};
 if ok==0||info.total_phys==0||info.available_phys>info.total_phys{return None;}
 Some(PhysicalMemory{free_mib:info.available_phys/1048576,total_mib:info.total_phys/1048576,system_load:info.load.min(100)})
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Risk{Unknown,LowCritical,LowWarning,Ready}
pub fn risk(value:Option<PhysicalMemory>)->Risk{
 match value{
  None=>Risk::Unknown,
  Some(m)if m.free_mib<512=>Risk::LowCritical,
  Some(m)if m.free_mib<1536=>Risk::LowWarning,
  Some(_)=>Risk::Ready
 }
}
pub fn allowed_to_start(value:Option<PhysicalMemory>)->Result<(),String>{
 match risk(value){
 Risk::LowCritical=>Err("Less than 512 MiB of physical RAM is free. Native GPU capture was not started; close background programs or restart later.".into()),
 _=>Ok(())
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn sample(mib:u64)->Option<PhysicalMemory>{Some(PhysicalMemory{free_mib:mib,total_mib:16384,system_load:50})}
 #[test]fn low_memory_is_rejected_without_changing_system(){
  assert_eq!(risk(sample(250)),Risk::LowCritical);
  assert!(allowed_to_start(sample(511)).is_err());
  assert_eq!(risk(sample(512)),Risk::LowWarning);
  assert_eq!(risk(sample(1536)),Risk::Ready);
  assert!(allowed_to_start(None).is_ok());
 }
 #[test]fn read_is_readonly_and_bounded(){
  if let Some(m)=read(){assert!(m.total_mib>=m.free_mib);assert!(m.system_load<=100);}
 }
}
