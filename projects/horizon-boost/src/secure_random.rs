//! Windows CSPRNG for the optional LAN dashboard access token.
use std::ffi::c_void;
type H=*mut c_void;
#[link(name="kernel32")]unsafe extern "system"{
 fn LoadLibraryExW(filename:*const u16,parent:H,flags:u32)->H;
 fn GetProcAddress(module:H,name:*const u8)->*const c_void;
 fn FreeLibrary(module:H)->i32;
}
pub fn bytes<const N:usize>()->Result<[u8;N],String>{
 if N==0||N>1024{return Err("Invalid token length".into());}
 let name="bcrypt.dll".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
 let lib=unsafe{LoadLibraryExW(name.as_ptr(),std::ptr::null_mut(),0x800)};
 if lib.is_null(){return Err("Windows BCrypt random provider unavailable".into());}
 struct Lib(H);impl Drop for Lib{fn drop(&mut self){unsafe{FreeLibrary(self.0);}}}
 let _lib=Lib(lib);
 let addr=unsafe{GetProcAddress(lib,b"BCryptGenRandom\0".as_ptr())};
 if addr.is_null(){return Err("BCryptGenRandom API missing".into());}
 let generate:unsafe extern "system" fn(H,*mut u8,u32,u32)->i32=unsafe{std::mem::transmute(addr)};
 let mut out=[0u8;N];let ntstatus=unsafe{generate(std::ptr::null_mut(),out.as_mut_ptr(),N as u32,2)};
 if ntstatus<0{return Err(format!("Windows CSPRNG failed: 0x{:08X}",ntstatus as u32));}
 Ok(out)
}
pub fn token()->Result<String,String>{
 let mut s=String::with_capacity(40);
 for byte in bytes::<20>()?{use std::fmt::Write;write!(s,"{byte:02x}").map_err(|e|e.to_string())?;}
 Ok(s)
}
#[cfg(test)]mod tests{
 #[test]fn tokens_are_os_random_160bit_and_distinct(){let a=super::token().unwrap();let b=super::token().unwrap();assert_eq!(a.len(),40);assert!(a.bytes().all(|c|c.is_ascii_hexdigit()));assert_ne!(a,b);}
}
