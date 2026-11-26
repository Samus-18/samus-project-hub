//! Own capture worker MMCSS scheduling (AvRT), never touches the game thread.
//! Only attempted after user explicitly requests low-latency own output.
//! Drop always reverts task classification, even if WGC fails or panics.
use std::ffi::c_void;
type H=*mut c_void;
const SEARCH_SYSTEM32:u32=0x800;
#[link(name="kernel32")]unsafe extern "system"{
 fn LoadLibraryExW(file:*const u16,parent:H,flags:u32)->H;
 fn GetProcAddress(lib:H,name:*const u8)->*const c_void;
 fn FreeLibrary(lib:H)->i32;
}
type Register=unsafe extern "system" fn(*const u16,*mut u32)->H;
type Revert=unsafe extern "system" fn(H)->i32;
pub struct Guard{library:H,token:H,revert:Revert}
impl Drop for Guard{
 fn drop(&mut self){unsafe{(self.revert)(self.token);FreeLibrary(self.library);}}
}
fn entry(module:H,name:&[u8])->Result<*const c_void,String>{
 let p=unsafe{GetProcAddress(module,name.as_ptr())};
 if p.is_null(){Err("MMCSS AvRT entry point is missing".into())}else{Ok(p)}
}
pub fn install()->Result<Guard,String>{
 let name="avrt.dll".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
 let library=unsafe{LoadLibraryExW(name.as_ptr(),std::ptr::null_mut(),SEARCH_SYSTEM32)};
 if library.is_null(){return Err("AvRT system DLL unavailable".into());}
 let result=(||{
  let register: Register=unsafe{std::mem::transmute(entry(library,b"AvSetMmThreadCharacteristicsW\0")?)};
  let revert: Revert=unsafe{std::mem::transmute(entry(library,b"AvRevertMmThreadCharacteristics\0")?)};
  let task="Games".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
  let mut index=0;
  let token=unsafe{register(task.as_ptr(),&mut index)};
  if token.is_null(){return Err("Windows rejected MMCSS Games task for own capture worker".into());}
  Ok(Guard{library,token,revert})
 })();
 if result.is_err(){unsafe{FreeLibrary(library);}}
 result
}
pub fn capability()->bool{
 let name="avrt.dll".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
 let library=unsafe{LoadLibraryExW(name.as_ptr(),std::ptr::null_mut(),SEARCH_SYSTEM32)};
 if library.is_null(){return false;}
 let ok=entry(library,b"AvSetMmThreadCharacteristicsW\0").is_ok()&&
  entry(library,b"AvRevertMmThreadCharacteristics\0").is_ok();
 unsafe{FreeLibrary(library);}
 ok
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn windows_mmcss_abi_detected_without_changing_priority(){assert!(capability());}
}
