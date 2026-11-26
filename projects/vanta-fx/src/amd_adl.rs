//! Optional read-only AMD Display Library (ADL2) query.
//! AMD Polaris fan, temperature and overdrive capability diagnostics.
//! NEVER calls FanSpeed_Set, voltage Set, overclocking or driver writes.
use std::{ffi::{c_void,CStr},sync::{Arc,Mutex,OnceLock},thread};
type Ptr=*mut c_void;
#[link(name="kernel32")]
unsafe extern "system"{
 fn LoadLibraryExW(path:*const u16,file:Ptr,flags:u32)->Ptr;
 fn GetProcAddress(module:Ptr,name:*const u8)->*const c_void;
 fn FreeLibrary(module:Ptr)->i32;
}
#[link(name="msvcrt")]
unsafe extern "C"{fn malloc(bytes:usize)->Ptr;}
unsafe extern "system" fn alloc(bytes:i32)->Ptr{
 if bytes<=0||bytes>4*1024*1024{return std::ptr::null_mut();}
 unsafe{malloc(bytes as usize)}
}
struct Driver{dll:Ptr,context:Ptr,destroy:Option<unsafe extern "system" fn(Ptr)->i32>}
impl Driver{
 fn symbol(&self,name:&CStr)->Result<*const c_void,String>{
  let ptr=unsafe{GetProcAddress(self.dll,name.as_ptr().cast())};
  (!ptr.is_null()).then_some(ptr).ok_or_else(||format!("AMD ADL export {} unavailable",name.to_string_lossy()))
 }
 fn resolve<F:Copy>(&self,bytes:&'static [u8])->Result<F,String>{
  let name=CStr::from_bytes_with_nul(bytes).map_err(|e|e.to_string())?;
  let ptr=self.symbol(name)?;
  Ok(unsafe{std::mem::transmute_copy(&ptr)})
 }
 fn open()->Result<Self,String>{
  // LOAD_LIBRARY_SEARCH_SYSTEM32: only signed-installed AMD Windows driver,
  // never relative DLL/path from a selected game or current directory.
  let name="atiadlxx.dll".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
  let dll=unsafe{LoadLibraryExW(name.as_ptr(),std::ptr::null_mut(),0x800)};
  if dll.is_null(){return Err("AMD ADL2 driver DLL is not installed".into());}
  let mut driver=Self{dll,context:std::ptr::null_mut(),destroy:None};
  type Create=unsafe extern "system" fn(unsafe extern "system" fn(i32)->Ptr,i32,*mut Ptr)->i32;
  type Destroy=unsafe extern "system" fn(Ptr)->i32;
  let create:Create=driver.resolve(b"ADL2_Main_Control_Create\0")?;
  let destroy:Destroy=driver.resolve(b"ADL2_Main_Control_Destroy\0")?;
  driver.destroy=Some(destroy);
  let status=unsafe{create(alloc,1,&mut driver.context)};
  if status!=0||driver.context.is_null(){return Err(format!("AMD ADL2 initialization refused: {status}"));}
  Ok(driver)
 }
}
impl Drop for Driver{
 fn drop(&mut self){
  if !self.context.is_null(){
   if let Some(drop)=self.destroy{unsafe{drop(self.context);}}
  }
  if !self.dll.is_null(){unsafe{FreeLibrary(self.dll);}}
 }
}
#[repr(C)]
#[derive(Default)]
struct FanSpeed{size:i32,speed_type:i32,speed:i32,flags:i32}
#[repr(C)]
#[derive(Default)]
struct Temperature{size:i32,temperature_milli_c:i32}
#[repr(C)]
#[derive(Default)]
struct FanInfo{size:i32,flags:i32,min_percent:i32,max_percent:i32,min_rpm:i32,max_rpm:i32}
pub fn read()->Result<String,String>{
 // The user explicitly invokes this, not on game launch or interval.
 let driver=Driver::open()?;
 type Count=unsafe extern "system" fn(Ptr,*mut i32)->i32;
 let count_fn:Count=driver.resolve(b"ADL2_Adapter_NumberOfAdapters_Get\0")?;
 let mut count=0;
 let status=unsafe{count_fn(driver.context,&mut count)};
 if status!=0||!(1..=32).contains(&count){return Err(format!("AMD ADL2 adapter enumeration failed ({status}; count {count})"));}
 type Active=unsafe extern "system" fn(Ptr,i32,*mut i32)->i32;
 let active_fn:Active=driver.resolve(b"ADL2_Adapter_Active_Get\0")?;
 let active=(0..count).filter(|&i|{
  let mut value=0;
  (unsafe{active_fn(driver.context,i,&mut value)})==0&&value!=0
 }).collect::<Vec<_>>();
 if active.len()!=1{
  return Err(format!("AMD ADL2 detects {} active logical adapters; refusing ambiguous fan/temperature mapping",active.len()));
 }
 let index=active[0];
 type GetFan=unsafe extern "system" fn(Ptr,i32,i32,*mut FanSpeed)->i32;
 type GetFanInfo=unsafe extern "system" fn(Ptr,i32,i32,*mut FanInfo)->i32;
 type GetTemperature=unsafe extern "system" fn(Ptr,i32,i32,*mut Temperature)->i32;
 let get_fan:GetFan=driver.resolve(b"ADL2_Overdrive5_FanSpeed_Get\0")?;
 let get_fan_info:GetFanInfo=driver.resolve(b"ADL2_Overdrive5_FanSpeedInfo_Get\0")?;
 let get_temp:GetTemperature=driver.resolve(b"ADL2_Overdrive5_Temperature_Get\0")?;
 let mut fan=FanSpeed{size:std::mem::size_of::<FanSpeed>() as i32,speed_type:1,..Default::default()};
 let mut fan_info=FanInfo{size:std::mem::size_of::<FanInfo>() as i32,..Default::default()};
 let mut temp=Temperature{size:std::mem::size_of::<Temperature>() as i32,..Default::default()};
 let status_fan=unsafe{get_fan(driver.context,index,0,&mut fan)};
 let status_info=unsafe{get_fan_info(driver.context,index,0,&mut fan_info)};
 let status_temp=unsafe{get_temp(driver.context,index,0,&mut temp)};
 let temp=if status_temp==0&&(0..=115_000).contains(&temp.temperature_milli_c){
  format!("{:.1} C",temp.temperature_milli_c as f32/1000.)
 }else{format!("unsupported/read refused ({status_temp})")};
 let fan=if status_fan==0&&fan.speed_type==1&&(0..=100).contains(&fan.speed){
  format!("{} %",fan.speed)
 }else if status_fan==0&&fan.speed_type==2&&(0..=15_000).contains(&fan.speed){
  format!("{} RPM",fan.speed)
 }else{format!("unsupported/read refused ({status_fan})")};
 let ranges=if status_info==0&&fan_info.min_percent>=0&&fan_info.max_percent<=100&&fan_info.min_percent<=fan_info.max_percent{
  format!("{}..{} %",fan_info.min_percent,fan_info.max_percent)
 }else{format!("unavailable ({status_info})")};
 Ok(format!("AMD driver ADL2 adapter {index} (only active logical adapter), GPU temp {temp}, fan {fan}, reported fan support {ranges}. READ ONLY; no fan/voltage/power setting changed. Manual fan curves and undervolting are NOT implemented."))
}
#[derive(Default)]struct State{busy:bool,status:Option<String>}
static STATUS:OnceLock<Arc<Mutex<State>>>=OnceLock::new();
fn state()->&'static Arc<Mutex<State>>{STATUS.get_or_init(||Arc::new(Mutex::new(State::default())))}
pub fn start()->Result<(),String>{
 let shared=state().clone();
 {let mut s=shared.lock().map_err(|_|"AMD driver diagnostic state unavailable")?;
  if s.busy{return Err("AMD ADL query already running".into());}s.busy=true;s.status=None;}
 let child=shared.clone();
 if let Err(e)=thread::Builder::new().name("Horizon AMD read-only driver query".into()).spawn(move||{
  let outcome=read().unwrap_or_else(|e|format!("AMD ADL read-only query unavailable: {e}"));
  if let Ok(mut s)=child.lock(){s.busy=false;s.status=Some(outcome);}
 }){
  if let Ok(mut s)=shared.lock(){s.busy=false;}
  return Err(format!("AMD driver query cannot start: {e}"));
 }
 Ok(())
}
pub fn snapshot()->(bool,Option<String>){
 state().lock().map(|s|(s.busy,s.status.clone())).unwrap_or((false,None))
}
#[cfg(test)]mod tests{
 #[test]fn amd_adl2_driver_exports_available_without_any_driver_write(){
  let driver=super::Driver::open().unwrap();
  for name in [
   b"ADL2_Adapter_NumberOfAdapters_Get\0".as_slice(),
   b"ADL2_Overdrive5_Temperature_Get\0".as_slice(),
   b"ADL2_Overdrive5_FanSpeed_Get\0".as_slice(),
  ]{
   let name=std::ffi::CStr::from_bytes_with_nul(name).unwrap();
   assert!(driver.symbol(name).is_ok(),"Missing ADL2 getter {name:?}");
  }
 }
}
