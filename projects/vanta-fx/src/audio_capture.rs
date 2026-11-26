//! Explicit recording-only WASAPI endpoints, bounded PCM16 files, separate tracks.
use std::{fs::{self,File,OpenOptions},io::{Write,Seek,SeekFrom},path::{Path,PathBuf},sync::{Arc,atomic::{AtomicBool,Ordering}},thread,time::{Duration,Instant}};
use windows::core::GUID;
use std::ffi::c_void;
type Ptr=*mut c_void;
#[link(name="ole32")]
unsafe extern "system"{
 fn CoInitializeEx(reserved:Ptr,flags:u32)->i32;
 fn CoUninitialize();
 fn CoCreateInstance(cls:*const GUID,outer:Ptr,ctx:u32,iid:*const GUID,out:*mut Ptr)->i32;
 fn CoTaskMemFree(p:Ptr);
}
#[repr(C,packed(1))]#[derive(Clone,Copy)]
struct WAVEFORMATEX{wFormatTag:u16,nChannels:u16,nSamplesPerSec:u32,nAvgBytesPerSec:u32,nBlockAlign:u16,wBitsPerSample:u16,cbSize:u16}
#[repr(C,packed(1))]#[derive(Clone,Copy)]
struct WAVEFORMATEXTENSIBLE{format:WAVEFORMATEX,valid_bits:u16,mask:u32,SubFormat:GUID}
struct Object(Ptr);
impl Object{fn entry(&self,n:usize)->*const c_void{unsafe{*(*(self.0 as *const *const *const c_void)).add(n)}}}
impl Drop for Object{fn drop(&mut self){let f:unsafe extern "system" fn(Ptr)->u32=unsafe{std::mem::transmute(self.entry(2))};unsafe{f(self.0);}}}
fn hr(v:i32,what:&str)->Result<(),String>{if v<0{Err(format!("{what}: HRESULT 0x{:08x}",v as u32))}else{Ok(())}}
fn obj(v:i32,p:Ptr,what:&str)->Result<Object,String>{hr(v,what)?;if p.is_null(){Err(format!("{what}: null interface"))}else{Ok(Object(p))}}
const NULL:Ptr=std::ptr::null_mut();
#[derive(Clone,Copy,Default)]pub struct Options{pub system:bool,pub microphone:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Kind{System,Microphone}
impl Kind{fn label(self)->&'static str{match self{Self::System=>"System audio",Self::Microphone=>"Microphone"}}fn suffix(self)->&'static str{match self{Self::System=>"system",Self::Microphone=>"mic"}}}
#[derive(Debug)]pub struct Track{pub kind:Kind,pub path:PathBuf,pub offset:f64,pub frames:u64,pub rate:u32,pub discontinuities:u32}
const MAX_BYTES:u64=512*1024*1024;
struct Wav{f:File,rate:u32,channels:u16,bytes:u64}
fn header(rate:u32,ch:u16,n:u32)->[u8;44]{
 let mut b=[0;44];b[0..4].copy_from_slice(b"RIFF");b[4..8].copy_from_slice(&(36+n).to_le_bytes());b[8..16].copy_from_slice(b"WAVEfmt ");b[16..20].copy_from_slice(&16u32.to_le_bytes());b[20..22].copy_from_slice(&1u16.to_le_bytes());b[22..24].copy_from_slice(&ch.to_le_bytes());b[24..28].copy_from_slice(&rate.to_le_bytes());b[28..32].copy_from_slice(&(rate*ch as u32*2).to_le_bytes());b[32..34].copy_from_slice(&(ch*2).to_le_bytes());b[34..36].copy_from_slice(&16u16.to_le_bytes());b[36..40].copy_from_slice(b"data");b[40..44].copy_from_slice(&n.to_le_bytes());b
}
impl Wav{
 fn new(p:&Path,rate:u32,ch:u16)->Result<Self,String>{if !(8000..=192000).contains(&rate)||!(1..=2).contains(&ch){return Err("Unsupported WAV shape".into());}let mut f=OpenOptions::new().write(true).create_new(true).open(p).map_err(|e|e.to_string())?;f.write_all(&header(rate,ch,0)).map_err(|e|e.to_string())?;Ok(Self{f,rate,channels:ch,bytes:0})}
 fn write(&mut self,b:&[u8])->Result<(),String>{if b.len()%(self.channels as usize*2)!=0||self.bytes+b.len() as u64>MAX_BYTES{return Err("Audio track reached bounded 512 MiB limit".into());}self.f.write_all(b).map_err(|e|e.to_string())?;self.bytes+=b.len() as u64;Ok(())}
 fn silence(&mut self,frames:u64)->Result<(),String>{let mut left=frames*self.channels as u64*2;let zeros=[0u8;8192];while left>0{let n=left.min(zeros.len() as u64) as usize;self.write(&zeros[..n])?;left-=n as u64;}Ok(())}
 fn frames(&self)->u64{self.bytes/(self.channels as u64*2)}
 fn finish(&mut self)->Result<(),String>{self.f.seek(SeekFrom::Start(0)).map_err(|e|e.to_string())?;self.f.write_all(&header(self.rate,self.channels,self.bytes as u32)).map_err(|e|e.to_string())?;self.f.sync_all().map_err(|e|e.to_string())}
}
impl Drop for Wav{fn drop(&mut self){let _=self.finish();}}
#[derive(Clone,Copy)]struct Format{rate:u32,input_ch:u16,out_ch:u16,bits:u16,float:bool,align:u16}
impl Format{
 unsafe fn read(p:*const WAVEFORMATEX)->Result<Self,String>{
  if p.is_null(){return Err("WASAPI returned null format".into());}let f=unsafe{std::ptr::read_unaligned(p)};
  let tag=f.wFormatTag;let bits=f.wBitsPerSample;let ch=f.nChannels;let rate=f.nSamplesPerSec;let align=f.nBlockAlign;
  let actual=if tag==0xfffe&&f.cbSize>=22{let ext=unsafe{std::ptr::read_unaligned(p.cast::<WAVEFORMATEXTENSIBLE>())};let sub=ext.SubFormat;
   if sub==GUID::from_u128(0x00000003_0000_0010_8000_00aa00389b71){3}else if sub==GUID::from_u128(0x00000001_0000_0010_8000_00aa00389b71){1}else{0}
  }else{tag};
  if !(8000..=192000).contains(&rate)||!(1..=8).contains(&ch)||align!=ch*(bits/8)||!((actual==3&&bits==32)||(actual==1&&matches!(bits,16|24|32))){return Err("Unsupported shared audio endpoint PCM/float format".into());}
  Ok(Self{rate,input_ch:ch,out_ch:ch.min(2),bits,float:actual==3,align})
 }
 fn sample(&self,b:&[u8])->i16{
  match (self.float,self.bits){(true,32)=>{let v=f32::from_le_bytes(b[..4].try_into().unwrap());if v.is_finite(){(v.clamp(-1.,1.)*32767.).round() as i16}else{0}},(false,16)=>i16::from_le_bytes(b[..2].try_into().unwrap()),(false,24)=>i16::from_le_bytes([b[1],b[2]]),(false,32)=>i16::from_le_bytes([b[2],b[3]]),_=>0}
 }
 fn convert(&self,raw:&[u8],frames:usize,out:&mut Vec<u8>)->Result<(),String>{
  if raw.len()!=frames*self.align as usize{return Err("Audio packet size mismatch".into());}out.clear();out.reserve(frames*self.out_ch as usize*2);let stride=self.bits as usize/8;
  for frame in raw.chunks_exact(self.align as usize){for c in 0..self.out_ch as usize{out.extend_from_slice(&self.sample(&frame[c*stride..c*stride+stride]).to_le_bytes());}}Ok(())
 }
}
struct Com;impl Drop for Com{fn drop(&mut self){unsafe{CoUninitialize();}}}
struct Mix(*mut WAVEFORMATEX);impl Drop for Mix{fn drop(&mut self){unsafe{CoTaskMemFree(self.0.cast());}}}
struct Active<'a>(&'a Object);impl Drop for Active<'_>{fn drop(&mut self){let stop:unsafe extern "system" fn(Ptr)->i32=unsafe{std::mem::transmute(self.0.entry(11))};unsafe{stop(self.0.0);}}}
fn capture(path:PathBuf,kind:Kind,run:Arc<AtomicBool>,video_start:Instant,gain:f32)->Result<Track,String>{
 if !run.load(Ordering::Acquire){return Err("Audio start cancelled".into());}
 hr(unsafe{CoInitializeEx(NULL,0)},"COM initialization")?;let _com=Com;
 let cls=GUID::from_u128(0xbcde0395_e52f_467c_8e3d_c4579291692e);
 let en_iid=GUID::from_u128(0xa95664d2_9614_4f35_a746_de8db63617e6);
 let client_iid=GUID::from_u128(0x1cb9ad4c_dbfa_4c32_b178_c2f568a703b2);
 let reader_iid=GUID::from_u128(0xc8adbd64_e71e_48a0_a4de_185c395cd317);
 let mut p=NULL;let en=obj(unsafe{CoCreateInstance(&cls,NULL,23,&en_iid,&mut p)},p,"IMMDeviceEnumerator")?;
 let get:unsafe extern "system" fn(Ptr,i32,i32,*mut Ptr)->i32=unsafe{std::mem::transmute(en.entry(4))};
 let mut p=NULL;let device=obj(unsafe{get(en.0,if kind==Kind::System{0}else{1},0,&mut p)},p,"Default audio endpoint")?;
 let activate:unsafe extern "system" fn(Ptr,*const GUID,u32,Ptr,*mut Ptr)->i32=unsafe{std::mem::transmute(device.entry(3))};
 let mut p=NULL;let client=obj(unsafe{activate(device.0,&client_iid,23,NULL,&mut p)},p,"IAudioClient")?;
 let get_mix:unsafe extern "system" fn(Ptr,*mut *mut WAVEFORMATEX)->i32=unsafe{std::mem::transmute(client.entry(8))};
 let mut p=std::ptr::null_mut();hr(unsafe{get_mix(client.0,&mut p)},"GetMixFormat")?;let mix=Mix(p);let format=unsafe{Format::read(mix.0)}?;
 let flags=if kind==Kind::System{0x20000}else{0};
 let init:unsafe extern "system" fn(Ptr,i32,u32,i64,i64,*const WAVEFORMATEX,*const GUID)->i32=unsafe{std::mem::transmute(client.entry(3))};
 hr(unsafe{init(client.0,0,flags,1_000_000,0,mix.0,std::ptr::null())},"IAudioClient Initialize")?;
 let service:unsafe extern "system" fn(Ptr,*const GUID,*mut Ptr)->i32=unsafe{std::mem::transmute(client.entry(14))};
 let mut p=NULL;let reader=obj(unsafe{service(client.0,&reader_iid,&mut p)},p,"IAudioCaptureClient")?;
 let next:unsafe extern "system" fn(Ptr,*mut u32)->i32=unsafe{std::mem::transmute(reader.entry(5))};
 let buffer:unsafe extern "system" fn(Ptr,*mut *mut u8,*mut u32,*mut u32,*mut u64,*mut u64)->i32=unsafe{std::mem::transmute(reader.entry(3))};
 let release_buffer:unsafe extern "system" fn(Ptr,u32)->i32=unsafe{std::mem::transmute(reader.entry(4))};
 if !run.load(Ordering::Acquire){return Err("Audio start cancelled".into());}
 let start:unsafe extern "system" fn(Ptr)->i32=unsafe{std::mem::transmute(client.entry(10))};
 hr(unsafe{start(client.0)},"Audio Start")?;let _active=Active(&client);
 let mut wav=Wav::new(&path,format.rate,format.out_ch)?;let mut out=Vec::new();let mut first_pos=None;let mut expected=None;let mut offset=0.;let mut gaps=0u32;
 let result=(||->Result<(),String>{
  while run.load(Ordering::Acquire)&&video_start.elapsed()<Duration::from_secs(7200){
   for _ in 0..32{
    if !run.load(Ordering::Acquire){break;}
    let mut packet_size=0;hr(unsafe{next(reader.0,&mut packet_size)},"GetNextPacketSize")?;if packet_size==0{break;}
    let(mut data,mut frames,mut flags,mut pos)=(std::ptr::null_mut(),0u32,0u32,0u64);
    hr(unsafe{buffer(reader.0,&mut data,&mut frames,&mut flags,&mut pos,std::ptr::null_mut())},"GetBuffer")?;
    let packet=(||->Result<(),String>{
     if frames>format.rate*2{return Err("Unexpected audio packet length".into());}if frames==0{return Ok(());}
     if first_pos.is_none(){first_pos=Some(pos);offset=(video_start.elapsed().as_secs_f64()-frames as f64/format.rate as f64).max(0.);}
     if let Some(e)=expected{if pos>e{let delta=pos-e;if delta>format.rate as u64*5{return Err("Large audio discontinuity; source retained".into());}wav.silence(delta)?;gaps=gaps.saturating_add(1);}else if pos<e{return Err("Audio device timestamp reset; source retained".into());}}
     if flags&2!=0{wav.silence(frames as u64)?;}else{
      if data.is_null(){return Err("Null WASAPI audio packet".into());}
      let raw=unsafe{std::slice::from_raw_parts(data,frames as usize*format.align as usize)};
      format.convert(raw,frames as usize,&mut out)?;apply_gain(&mut out,gain);wav.write(&out)?;
     }
     expected=Some(pos.saturating_add(frames as u64));Ok(())
    })();
    let release=hr(unsafe{release_buffer(reader.0,frames)},"ReleaseBuffer");
    packet?;release?;
   }thread::sleep(Duration::from_millis(8));
  }Ok(())
 })();
 let frames=wav.frames();wav.finish()?;result?;
 if frames==0{return Err("No endpoint packets received; empty WAV retained".into());}
 Ok(Track{kind,path,offset,frames,rate:format.rate,discontinuities:gaps})
}
pub struct Worker{stop:Arc<AtomicBool>,join:Option<thread::JoinHandle<Result<Track,String>>>}
impl Worker{
 pub fn start(video:&Path,kind:Kind,start:Instant)->Result<Self,String>{Self::start_gain(video,kind,start,1.)}
 pub fn start_gain(video:&Path,kind:Kind,start:Instant,gain:f32)->Result<Self,String>{
  let stem=video.file_stem().and_then(|s|s.to_str()).ok_or("Recording filename invalid")?;
  let path=video.with_file_name(format!("{stem}_{}.wav",kind.suffix()));let stop=Arc::new(AtomicBool::new(true));let flag=stop.clone();
  let join=thread::Builder::new().name(format!("Horizon WASAPI {}",kind.suffix())).spawn(move||capture(path,kind,flag,start,gain)).map_err(|e|e.to_string())?;
  Ok(Self{stop,join:Some(join)})
 }
 pub fn cancel(&self){self.stop.store(false,Ordering::Release);}
 pub fn finish(mut self)->Result<Track,String>{self.cancel();self.join.take().ok_or("Audio worker missing")?.join().map_err(|_|"Audio worker panicked".to_string())?}
}
impl Drop for Worker{fn drop(&mut self){self.cancel();if let Some(t)=self.join.take(){let _=t.join();}}}
pub fn mux(video:&Path,tracks:&[Track],output:&Path)->Result<(),String>{
 if tracks.is_empty()||tracks.len()>2||output.exists()||video==output{return Err("Invalid audio mux request".into());}
 let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;let mut cmd=std::process::Command::new(exe);
 cmd.args(["-hide_banner","-loglevel","error","-n","-i"]).arg(video);
 for t in tracks{if !t.offset.is_finite()||!(0.0..=7200.).contains(&t.offset)||t.frames==0{return Err("Invalid audio track timing".into());}
  cmd.args(["-itsoffset",&format!("{:.4}",t.offset),"-i"]).arg(&t.path);}
 cmd.args(["-map","0:v:0"]);
 for(i,t)in tracks.iter().enumerate(){cmd.args(["-map",&format!("{}:a:0",i+1),&format!("-metadata:s:a:{i}"),&format!("title={}",t.kind.label())]);}
 cmd.args(["-c:v","copy","-c:a","aac","-b:a","160k"]).arg(output);
 let result=crate::media_tools::bounded(&mut cmd,Duration::from_secs(180)).and_then(|_|{let _=crate::media_tools::probe(output)?;let n=crate::media_tools::audio_track_count(output)?;if n!=tracks.len(){return Err("Mux audio-track validation failed".into());}Ok(())});
 if result.is_err(){let _=fs::remove_file(output);}result
}
pub fn proof()->Result<String,String>{
 let dir=std::env::temp_dir().join(format!("hb-audio-proof-{}",crate::media_tools::stamp()));fs::create_dir(&dir).map_err(|e|e.to_string())?;
 let result=(||{
  let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;let video=dir.join("video.mkv");let out=dir.join("mux.mkv");
  crate::media_tools::bounded(std::process::Command::new(&exe).args(["-hide_banner","-loglevel","error","-f","lavfi","-i","testsrc2=size=160x90:rate=30","-t","1","-c:v","libx264"]).arg(&video),Duration::from_secs(30))?;
  let mut tracks=Vec::new();for (i,kind) in [Kind::System,Kind::Microphone].into_iter().enumerate(){let p=dir.join(format!("track{i}.wav"));let mut w=Wav::new(&p,48000,1)?;let raw=(0..48000).flat_map(|n|(((n as f32*std::f32::consts::TAU*(440.+i as f32*220.)/48000.).sin()*10000.) as i16).to_le_bytes()).collect::<Vec<_>>();w.write(&raw)?;w.finish()?;drop(w);tracks.push(Track{kind,path:p,offset:0.,frames:48000,rate:48000,discontinuities:0});}
  mux(&video,&tracks,&out)?;crate::media_tools::bounded(std::process::Command::new(exe).args(["-v","error","-i"]).arg(&out).args(["-map","0:v:0","-map","0:a:0","-map","0:a:1","-f","null","NUL"]),Duration::from_secs(30))?;
  Ok("AUDIO_MUX=PASS SEPARATE_TRACKS=2 PCM16=true DECODE=true SYNTHETIC_ONLY=true NO_ENDPOINT_CAPTURE=true".into())
 })();let _=fs::remove_dir_all(dir);result
}
#[cfg(test)]mod tests{use super::*;#[test]fn header_and_conversion(){let h=header(48000,2,16);assert_eq!(&h[..4],b"RIFF");assert_eq!(&h[40..],&16u32.to_le_bytes());let f=Format{rate:48000,input_ch:2,out_ch:2,bits:32,float:true,align:8};let raw=[0.5f32.to_le_bytes(),(-0.5f32).to_le_bytes()].concat();let mut b=Vec::new();f.convert(&raw,1,&mut b).unwrap();assert_eq!(i16::from_le_bytes(b[..2].try_into().unwrap()),16384);assert_eq!(i16::from_le_bytes(b[2..].try_into().unwrap()),-16384);}}

pub fn monitor_spectrum(run:Arc<AtomicBool>,mut publish:impl FnMut(crate::audio_spectrum::Reading))->Result<(),String>{
 let kind=Kind::System;
 if !run.load(Ordering::Acquire){return Err("Audio start cancelled".into());}
 hr(unsafe{CoInitializeEx(NULL,0)},"COM initialization")?;let _com=Com;
 let cls=GUID::from_u128(0xbcde0395_e52f_467c_8e3d_c4579291692e);
 let en_iid=GUID::from_u128(0xa95664d2_9614_4f35_a746_de8db63617e6);
 let client_iid=GUID::from_u128(0x1cb9ad4c_dbfa_4c32_b178_c2f568a703b2);
 let reader_iid=GUID::from_u128(0xc8adbd64_e71e_48a0_a4de_185c395cd317);
 let mut p=NULL;let en=obj(unsafe{CoCreateInstance(&cls,NULL,23,&en_iid,&mut p)},p,"IMMDeviceEnumerator")?;
 let get:unsafe extern "system" fn(Ptr,i32,i32,*mut Ptr)->i32=unsafe{std::mem::transmute(en.entry(4))};
 let mut p=NULL;let device=obj(unsafe{get(en.0,if kind==Kind::System{0}else{1},0,&mut p)},p,"Default audio endpoint")?;
 let activate:unsafe extern "system" fn(Ptr,*const GUID,u32,Ptr,*mut Ptr)->i32=unsafe{std::mem::transmute(device.entry(3))};
 let mut p=NULL;let client=obj(unsafe{activate(device.0,&client_iid,23,NULL,&mut p)},p,"IAudioClient")?;
 let get_mix:unsafe extern "system" fn(Ptr,*mut *mut WAVEFORMATEX)->i32=unsafe{std::mem::transmute(client.entry(8))};
 let mut p=std::ptr::null_mut();hr(unsafe{get_mix(client.0,&mut p)},"GetMixFormat")?;let mix=Mix(p);let format=unsafe{Format::read(mix.0)}?;
 let flags=if kind==Kind::System{0x20000}else{0};
 let init:unsafe extern "system" fn(Ptr,i32,u32,i64,i64,*const WAVEFORMATEX,*const GUID)->i32=unsafe{std::mem::transmute(client.entry(3))};
 hr(unsafe{init(client.0,0,flags,1_000_000,0,mix.0,std::ptr::null())},"IAudioClient Initialize")?;
 let service:unsafe extern "system" fn(Ptr,*const GUID,*mut Ptr)->i32=unsafe{std::mem::transmute(client.entry(14))};
 let mut p=NULL;let reader=obj(unsafe{service(client.0,&reader_iid,&mut p)},p,"IAudioCaptureClient")?;
 let next:unsafe extern "system" fn(Ptr,*mut u32)->i32=unsafe{std::mem::transmute(reader.entry(5))};
 let buffer:unsafe extern "system" fn(Ptr,*mut *mut u8,*mut u32,*mut u32,*mut u64,*mut u64)->i32=unsafe{std::mem::transmute(reader.entry(3))};
 let release_buffer:unsafe extern "system" fn(Ptr,u32)->i32=unsafe{std::mem::transmute(reader.entry(4))};
 if !run.load(Ordering::Acquire){return Err("Audio start cancelled".into());}
 let start:unsafe extern "system" fn(Ptr)->i32=unsafe{std::mem::transmute(client.entry(10))};
 hr(unsafe{start(client.0)},"Audio Start")?;let _active=Active(&client);

 let mut analyzer=crate::audio_spectrum::Analyzer::default();let mut last=Instant::now();let mut last_packet=Instant::now();
 while run.load(Ordering::Acquire){
  for _ in 0..32{
   if !run.load(Ordering::Acquire){break;}
   let mut size=0;hr(unsafe{next(reader.0,&mut size)},"FFT GetNextPacketSize")?;if size==0{break;}
   let(mut data,mut frames,mut flags)=(std::ptr::null_mut(),0u32,0u32);
   hr(unsafe{buffer(reader.0,&mut data,&mut frames,&mut flags,std::ptr::null_mut(),std::ptr::null_mut())},"FFT GetBuffer")?;
   let result=(||->Result<(),String>{
    if frames>format.rate*2{return Err("Unexpected FFT packet length".into());}
    if flags&1!=0{analyzer.clear();}
    if flags&2!=0{analyzer.silence(frames as usize);}else if frames!=0{
     if data.is_null(){return Err("Null FFT packet".into());}
     let raw=unsafe{std::slice::from_raw_parts(data,frames as usize*format.align as usize)};
     let width=format.bits as usize/8;
     // First output channel only, no saved audio; avoid phase cancellation from summing stereo.
     for frame in raw.chunks_exact(format.align as usize){analyzer.push(format.sample(&frame[..width]) as f32/32768.);}
    }
    if frames>0{last_packet=Instant::now();}Ok(())
   })();
   let release=hr(unsafe{release_buffer(reader.0,frames)},"FFT ReleaseBuffer");result?;release?;
  }
  if last.elapsed()>=Duration::from_millis(100){
   if last_packet.elapsed()>Duration::from_millis(350){analyzer.clear();}
   if let Some(v)=analyzer.measure(format.rate,Instant::now()){publish(v);}
   last=Instant::now();
  }thread::sleep(Duration::from_millis(8));
 }Ok(())
}

pub fn apply_gain(buf:&mut [u8],gain:f32){let gain=if gain.is_finite(){gain.clamp(0.,2.)}else{1.};if gain==1.{return;}for b in buf.chunks_exact_mut(2){let x=i16::from_le_bytes([b[0],b[1]])as f32;let y=(x*gain).round().clamp(-32768.,32767.)as i16;b.copy_from_slice(&y.to_le_bytes());}}
#[cfg(test)]mod gain_v117{#[test]fn pcm_gain_and_mute_saturate(){let mut b=[20000i16.to_le_bytes(),(-20000i16).to_le_bytes()].concat();super::apply_gain(&mut b,2.);assert_eq!(i16::from_le_bytes(b[..2].try_into().unwrap()),32767);assert_eq!(i16::from_le_bytes(b[2..].try_into().unwrap()),-32768);super::apply_gain(&mut b,0.);assert_eq!(b,[0,0,0,0]);}}
