//! AMD ADL2 read-only temperature, from installed System32 driver.
use std::{ffi::c_void,sync::mpsc::{self,Receiver},time::{Instant,Duration}};
type Ptr=*mut c_void;
#[link(name="kernel32")]unsafe extern "system"{fn LoadLibraryExW(n:*const u16,h:Ptr,f:u32)->Ptr;fn GetProcAddress(h:Ptr,n:*const u8)->*const c_void;fn FreeLibrary(h:Ptr)->i32;}
#[link(name="msvcrt")]unsafe extern "C"{fn malloc(size:usize)->Ptr;}
unsafe extern "system" fn allocate(n:i32)->Ptr{if n<=0{std::ptr::null_mut()}else{unsafe{malloc(n as usize)}}}
#[repr(C)]#[derive(Clone,Copy)]struct AdapterInfo{size:i32,index:i32,udid:[u8;256],bus:i32,device:i32,function:i32,vendor:i32,name:[u8;256],display:[u8;256],present:i32,exist:i32,driver:[u8;256],driver_ext:[u8;256],pnp:[u8;256],os_index:i32}
#[repr(C)]struct Temperature{size:i32,value:i32}
type Destroy=unsafe extern "C" fn(Ptr)->i32;
struct Session{dll:Ptr,ctx:Ptr,destroy:Destroy}
impl Drop for Session{fn drop(&mut self){unsafe{if !self.ctx.is_null(){(self.destroy)(self.ctx);}FreeLibrary(self.dll);}}}
#[derive(Clone,Debug)]pub struct Sensor{pub name:String,pub edge:Option<f32>,pub hotspot:Option<f32>,pub memory:Option<f32>}
fn valid(raw:i32)->Option<f32>{(1000..=150000).contains(&raw).then_some(raw as f32/1000.)}
fn symbol(dll:Ptr,name:&[u8])->Result<*const c_void,String>{let p=unsafe{GetProcAddress(dll,name.as_ptr())};if p.is_null(){Err("ADL export missing".into())}else{Ok(p)}}
pub fn read()->Result<Vec<Sensor>,String>{
 let dll=unsafe{LoadLibraryExW("atiadlxx.dll".encode_utf16().chain(Some(0)).collect::<Vec<_>>().as_ptr(),std::ptr::null_mut(),0x800)};
 if dll.is_null(){return Err("AMD System32 ADL driver library unavailable".into());}
 let setup=(||->Result<Session,String>{
  let create:unsafe extern "C" fn(unsafe extern "system" fn(i32)->Ptr,i32,*mut Ptr)->i32=unsafe{std::mem::transmute(symbol(dll,b"ADL2_Main_Control_Create\0")?)};
  let destroy:Destroy=unsafe{std::mem::transmute(symbol(dll,b"ADL2_Main_Control_Destroy\0")?)};
  let mut ctx=std::ptr::null_mut();if unsafe{create(allocate,1,&mut ctx)}!=0||ctx.is_null(){if !ctx.is_null(){unsafe{destroy(ctx);}}return Err("ADL context unavailable".into());}
  Ok(Session{dll,ctx,destroy})
 })();let s=match setup{Ok(s)=>s,Err(e)=>{unsafe{FreeLibrary(dll);}return Err(e);}};
 let count:unsafe extern "C" fn(Ptr,*mut i32)->i32=unsafe{std::mem::transmute(symbol(dll,b"ADL2_Adapter_NumberOfAdapters_Get\0")?)};
 let info:unsafe extern "C" fn(Ptr,*mut AdapterInfo,i32)->i32=unsafe{std::mem::transmute(symbol(dll,b"ADL2_Adapter_AdapterInfo_Get\0")?)};
 let caps:unsafe extern "C" fn(Ptr,i32,*mut i32,*mut i32,*mut i32)->i32=unsafe{std::mem::transmute(symbol(dll,b"ADL2_Overdrive_Caps\0")?)};
 let od5= symbol(dll,b"ADL2_Overdrive5_Temperature_Get\0").ok().map(|p|unsafe{std::mem::transmute::<_,unsafe extern "C" fn(Ptr,i32,i32,*mut Temperature)->i32>(p)});
 let od6= symbol(dll,b"ADL2_Overdrive6_Temperature_Get\0").ok().map(|p|unsafe{std::mem::transmute::<_,unsafe extern "C" fn(Ptr,i32,*mut i32)->i32>(p)});
 let odn= symbol(dll,b"ADL2_OverdriveN_Temperature_Get\0").ok().map(|p|unsafe{std::mem::transmute::<_,unsafe extern "C" fn(Ptr,i32,i32,*mut i32)->i32>(p)});
 let mut n=0;if unsafe{count(s.ctx,&mut n)}!=0||!(1..=64).contains(&n){return Err("No bounded ADL adapter list".into());}
 let zero:AdapterInfo=unsafe{std::mem::zeroed()};let mut rows=vec![zero;n as usize];for row in &mut rows{row.size=std::mem::size_of::<AdapterInfo>() as i32;}
 if unsafe{info(s.ctx,rows.as_mut_ptr(),(rows.len()*std::mem::size_of::<AdapterInfo>()) as i32)}!=0{return Err("ADL adapter information unavailable".into());}
 let mut seen=std::collections::BTreeSet::new();let mut out=Vec::new();
 for row in rows{if row.bus<0||row.index<0||!seen.insert((row.bus,row.device,row.function)){continue;}
  let(mut supported,mut enabled,mut version)=(0,0,0);if unsafe{caps(s.ctx,row.index,&mut supported,&mut enabled,&mut version)}!=0||supported==0{continue;}
  let ntemp=|kind|{let mut raw=0;let f=odn?;if unsafe{f(s.ctx,row.index,kind,&mut raw)}==0{valid(raw)}else{None}};
  let edge=match version{5=>od5.and_then(|f|{let mut t=Temperature{size:8,value:0};if unsafe{f(s.ctx,row.index,0,&mut t)}==0{valid(t.value)}else{None}}),6=>od6.and_then(|f|{let mut t=0;if unsafe{f(s.ctx,row.index,&mut t)}==0{valid(t)}else{None}}),v if v>=7=>ntemp(1),_=>None};
  let end=row.name.iter().position(|&v|v==0).unwrap_or(row.name.len());
  out.push(Sensor{name:String::from_utf8_lossy(&row.name[..end]).into_owned(),edge,hotspot:if version>=7{ntemp(7)}else{None},memory:if version>=7{ntemp(2)}else{None}});
 }
 Ok(out)
}
#[derive(Default)]pub struct Meter{pending:Option<Receiver<(Instant,Result<Vec<Sensor>,String>)>>,last:Option<Instant>,value:Option<(Instant,Result<Vec<Sensor>,String>)>}
impl Meter{
 pub fn tick(&mut self,enabled:bool,now:Instant){
  if let Some(rx)=&self.pending{match rx.try_recv(){Ok((at,v))=>{self.pending=None;if enabled&&self.last.is_some(){self.value=Some((at,v));}},Err(mpsc::TryRecvError::Disconnected)=>{self.pending=None;},Err(mpsc::TryRecvError::Empty)=>{}}}
  if !enabled{self.value=None;self.last=None;return;}
  if self.pending.is_none()&&self.last.is_none_or(|t|now.saturating_duration_since(t)>=Duration::from_secs(5)){let(tx,rx)=mpsc::channel();self.pending=Some(rx);self.last=Some(now);std::thread::spawn(move||{let value=read();let _=tx.send((Instant::now(),value));});}
 }
 pub fn value(&self,now:Instant)->Option<&Result<Vec<Sensor>,String>>{self.value.as_ref().filter(|(t,_)|now.saturating_duration_since(*t)<=Duration::from_secs(12)).map(|(_,v)|v)}
 pub fn hottest(&self,now:Instant)->Option<f32>{self.value(now)?.as_ref().ok()?.iter().filter_map(|s|s.edge).max_by(f32::total_cmp)}
}
pub fn proof()->Result<String,String>{let sensors=read()?;let valid=sensors.iter().filter(|s|s.edge.is_some()).count();if valid==0{return Err("AMD ADL returned no valid edge temperature".into());}Ok(format!("AMD_SENSORS=PASS GPU_COUNT={} EDGE_AVAILABLE={} READ_ONLY=true CPU_TEMP_UNKNOWN=true",sensors.len(),valid))}
#[cfg(test)]mod tests{use super::*;#[test]fn adl_abi_and_units(){assert_eq!(std::mem::size_of::<AdapterInfo>(),1572);assert_eq!(std::mem::size_of::<Temperature>(),8);assert_eq!(valid(42500),Some(42.5));assert_eq!(valid(42),None);}}

#[cfg(test)]mod v88_actual_sensors{#[test]fn real_read_only_amd_sensor(){println!("{}",super::proof().expect("read-only AMD sensor"));}}