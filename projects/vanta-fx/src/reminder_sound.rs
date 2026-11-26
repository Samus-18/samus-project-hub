use std::{path::{Path,PathBuf},fs,io::Read,ffi::c_void};
fn path()->PathBuf{crate::settings::folder().join("reminder.wav")}
fn validate(data:&[u8])->Result<(),String>{
 if data.len()<44||data.len()>1024*1024||&data[..4]!=b"RIFF"||&data[8..12]!=b"WAVE"{return Err("Choose PCM WAV under 1 MiB".into());}
 let n=|p:usize|u32::from_le_bytes(data[p..p+4].try_into().unwrap()) as usize;
 if n(4)+8!=data.len(){return Err("Invalid WAV length".into());}
 let mut p=12;let mut fmt=None;let mut samples=None;
 while p+8<=data.len(){
  let size=n(p+4);let start=p+8;let end=start.checked_add(size).ok_or("WAV overflow")?;
  if end>data.len(){return Err("Truncated WAV chunk".into());}
  match &data[p..p+4]{
   b"fmt " if size>=16&&fmt.is_none()=>{let v=&data[start..end];let word=|x|u16::from_le_bytes([v[x],v[x+1]]);
    let channels=word(2);let rate=u32::from_le_bytes(v[4..8].try_into().unwrap());let byte_rate=u32::from_le_bytes(v[8..12].try_into().unwrap());let bits=word(14);
    if word(0)!=1||!(1..=2).contains(&channels)||!(8000..=48000).contains(&rate)||!matches!(bits,8|16)||byte_rate!=rate*channels as u32*bits as u32/8||word(12)!=channels*bits/8{return Err("Only standard PCM mono/stereo 8/16-bit WAV".into());}
    fmt=Some((byte_rate as usize,word(12) as usize));
   },b"data" if samples.is_none()=>samples=Some(size),b"fmt "|b"data"=>return Err("Duplicate or invalid WAV chunk".into()),_=>{}
  }
  p=end+(size&1);if p>data.len(){return Err("Missing WAV padding".into());}
 }
 if p!=data.len(){return Err("Trailing incomplete WAV chunk".into());}
 let (rate,align)=fmt.ok_or("Missing PCM format")?;let bytes=samples.ok_or("Missing audio data")?;
 if bytes==0||bytes>rate*3||bytes%align!=0{return Err("Choose a sound up to three seconds".into());}Ok(())
}
pub fn import(source:&Path)->Result<(),String>{
 let mut data=Vec::new();fs::File::open(source).map_err(|e|e.to_string())?.take(1024*1024+1).read_to_end(&mut data).map_err(|e|e.to_string())?;
 validate(&data)?;let p=path();fs::create_dir_all(p.parent().ok_or("Missing parent")?).map_err(|e|e.to_string())?;crate::settings::atomic_json_replace(&p,&data)
}
#[link(name="kernel32")]unsafe extern "system"{fn LoadLibraryExW(n:*const u16,f:*mut c_void,flags:u32)->*mut c_void;fn GetProcAddress(m:*mut c_void,n:*const u8)->*const c_void;fn FreeLibrary(m:*mut c_void)->i32;}
static PLAYING:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
struct PlaybackGuard;impl Drop for PlaybackGuard{fn drop(&mut self){PLAYING.store(false,std::sync::atomic::Ordering::Release);}}
pub fn play()->Result<(),String>{
 let p=path();let mut raw=Vec::new();fs::File::open(&p).map_err(|e|e.to_string())?.take(1024*1024+1).read_to_end(&mut raw).map_err(|e|e.to_string())?;validate(&raw)?;
 // Worker uses synchronous PlaySound, bounded by the validated three-second PCM file. No persistent DLL.
 if PLAYING.compare_exchange(false,true,std::sync::atomic::Ordering::AcqRel,std::sync::atomic::Ordering::Acquire).is_err(){return Ok(());}
 let guard=PlaybackGuard;std::thread::Builder::new().name("horizon-reminder-sound".into()).spawn(move||{let _guard=guard;
  let wide="winmm.dll".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
  let lib=unsafe{LoadLibraryExW(wide.as_ptr(),std::ptr::null_mut(),0x800)};if lib.is_null(){return;}
  let f=unsafe{GetProcAddress(lib,b"PlaySoundW\0".as_ptr())};
  if !f.is_null(){let play:unsafe extern "system" fn(*const u16,*mut c_void,u32)->i32=unsafe{std::mem::transmute(f)};
   unsafe{play(raw.as_ptr().cast(),std::ptr::null_mut(),0x0004|0x0002|0x0010);}}
  unsafe{FreeLibrary(lib);}
 }).map_err(|e|e.to_string())?;Ok(())
}
#[cfg(test)]mod tests{
 #[test]fn validates_real_pcm_header_and_rejects_invalid(){let mut b=Vec::new();b.extend(b"RIFF");b.extend(38u32.to_le_bytes());b.extend(b"WAVEfmt ");b.extend(16u32.to_le_bytes());b.extend(1u16.to_le_bytes());b.extend(1u16.to_le_bytes());b.extend(8000u32.to_le_bytes());b.extend(8000u32.to_le_bytes());b.extend(1u16.to_le_bytes());b.extend(8u16.to_le_bytes());b.extend(b"data");b.extend(2u32.to_le_bytes());b.extend([128,128]);assert!(super::validate(&b).is_ok());b[20]=3;assert!(super::validate(&b).is_err());}
}
