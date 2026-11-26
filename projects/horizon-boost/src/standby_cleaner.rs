//! Manual Windows standby-page-list purge, never invoked on startup/timer.
//! Requires an explicit second-click UI confirmation; can worsen cold loads.
//! This uses an undocumented Windows NT interface: fail closed on any error.
use std::{ffi::c_void,sync::{Mutex,OnceLock},time::{Instant,Duration}};
#[repr(C)]
#[derive(Default)]
struct MemoryStatus{length:u32,load:u32,total_phys:u64,available_phys:u64,total_page:u64,available_page:u64,total_virt:u64,available_virt:u64,extended_virt:u64}
#[link(name="kernel32")]
unsafe extern "system"{
 fn GlobalMemoryStatusEx(status:*mut MemoryStatus)->i32;
 fn LoadLibraryExW(name:*const u16,file:*mut c_void,flags:u32)->*mut c_void;
 fn GetProcAddress(module:*mut c_void,name:*const u8)->*const c_void;
 fn FreeLibrary(module:*mut c_void)->i32;
}
const SYSTEM_MEMORY_LIST_INFORMATION:u32=80;
const MEMORY_PURGE_STANDBY_LIST:u32=4;
static LAST_SUCCESS:OnceLock<Mutex<Option<Instant>>>=OnceLock::new();
fn cooldown()->&'static Mutex<Option<Instant>>{LAST_SUCCESS.get_or_init(||Mutex::new(None))}
pub fn ram()->Result<(u64,u64,u32),String>{
 let mut s=MemoryStatus{length:std::mem::size_of::<MemoryStatus>() as u32,..Default::default()};
 if unsafe{GlobalMemoryStatusEx(&mut s)}==0{return Err("Windows RAM status unavailable".into());}
 if s.total_phys==0||s.available_phys>s.total_phys{return Err("Invalid Windows RAM counter".into());}
 Ok((s.total_phys,s.available_phys,s.load))
}
pub fn purge_confirmed()->Result<String,String>{
 let mut previous=cooldown().lock().map_err(|_|"Standby cleaner cooldown unavailable")?;
 if previous.is_some_and(|last|last.elapsed()<Duration::from_secs(90)){
  return Err("Standby purge was already used in the last 90 seconds".into());
 }
 let (_,before,load)=ram()?;
 // Avoid unnecessary purges when Windows already has abundant free memory.
 if before>=3*1024*1024*1024{return Err(format!(
   "Standby purge skipped: Windows reports {:.1} GiB available; purging cache would only harm loading",before as f64/1073741824.));}
 let module="ntdll.dll".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
 let dll=unsafe{LoadLibraryExW(module.as_ptr(),std::ptr::null_mut(),0x800)};
 if dll.is_null(){return Err("Windows NT memory API unavailable".into());}
 let entry=unsafe{GetProcAddress(dll,b"NtSetSystemInformation\0".as_ptr())};
 if entry.is_null(){unsafe{FreeLibrary(dll);}return Err("Windows NT standby interface unavailable".into());}
 let action=MEMORY_PURGE_STANDBY_LIST;
 type NtSet=unsafe extern "system" fn(u32,*const c_void,u32)->i32;
 let f:NtSet=unsafe{std::mem::transmute(entry)};
 let ntstatus=unsafe{f(SYSTEM_MEMORY_LIST_INFORMATION,(&action as *const u32).cast(),4)};
 unsafe{FreeLibrary(dll);}
 if ntstatus<0{return Err(format!("Windows declined standby purge NTSTATUS=0x{:08x}; elevated SeProfileSingleProcessPrivilege may be required. Nothing was forced.",ntstatus as u32));}
 *previous=Some(Instant::now());
 let after=ram().map(|(_,available,_)|available).unwrap_or(before);
 Ok(format!("User-requested standby purge completed. RAM available {:.2} -> {:.2} GiB (load {load}%). Windows can rebuild standby cache; do not run repeatedly.",
  before as f64/1073741824.,after as f64/1073741824.))
}
#[cfg(test)]mod tests{
 #[test]fn read_only_ram_probe_and_no_default_purge(){
  let (total,free,load)=super::ram().unwrap();
  assert!(total>free&&free>0&&load<=100);
  assert_eq!(super::MEMORY_PURGE_STANDBY_LIST,4);
  assert!(super::LAST_SUCCESS.get().is_none());
 }
}
