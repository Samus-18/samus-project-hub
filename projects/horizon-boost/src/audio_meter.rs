// Read-only Windows Core Audio endpoint peak. Documented COM ABI.
use std::ffi::c_void;
use windows::core::GUID;
type Ptr=*mut c_void;
#[link(name="kernel32")]
unsafe extern "system"{
 fn LoadLibraryExW(name:*const u16,file:Ptr,flags:u32)->Ptr;
 fn GetProcAddress(module:Ptr,name:*const u8)->*const c_void;
 fn FreeLibrary(module:Ptr)->i32;
}
struct Library(Ptr);
impl Drop for Library{fn drop(&mut self){unsafe{FreeLibrary(self.0);}}}
struct Object(Ptr);
impl Object{
 fn entry(&self,index:usize)->*const c_void{
  // COM objects expose an immutable pointer to the interface vtable.
  unsafe{*(*(self.0 as *const *const *const c_void)).add(index)}
 }
}
impl Drop for Object{
 fn drop(&mut self){
  let release:unsafe extern "system" fn(Ptr)->u32=unsafe{std::mem::transmute(self.entry(2))};
  unsafe{release(self.0);}
 }
}
fn check(hr:i32,p:Ptr,operation:&str)->Result<Object,String>{
 if hr<0||p.is_null(){Err(format!("{operation} failed (0x{:08x})",hr as u32))}else{Ok(Object(p))}
}
pub fn peak()->Result<f32,String>{
 let wide="ole32.dll".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
 let lib=Library(unsafe{LoadLibraryExW(wide.as_ptr(),std::ptr::null_mut(),0x800)});
 if lib.0.is_null(){return Err("System COM library unavailable".into());}
 let symbol=|n:&[u8]|->Result<*const c_void,String>{
  let p=unsafe{GetProcAddress(lib.0,n.as_ptr())};if p.is_null(){Err("COM export unavailable".into())}else{Ok(p)}
 };
 let init:unsafe extern "system" fn(Ptr,u32)->i32=unsafe{std::mem::transmute(symbol(b"CoInitializeEx\0")?)};
 let end:unsafe extern "system" fn()=unsafe{std::mem::transmute(symbol(b"CoUninitialize\0")?)};
 let create:unsafe extern "system" fn(*const GUID,Ptr,u32,*const GUID,*mut Ptr)->i32=unsafe{std::mem::transmute(symbol(b"CoCreateInstance\0")?)};
 let hr=unsafe{init(std::ptr::null_mut(),0)};if hr<0{return Err(format!("COM initialization failed 0x{:08x}",hr as u32));}
 struct Com(unsafe extern "system" fn());impl Drop for Com{fn drop(&mut self){unsafe{(self.0)();}}}
 let _com=Com(end);
 let cls=GUID::from_u128(0xbcde0395_e52f_467c_8e3d_c4579291692e);
 let iid=GUID::from_u128(0xa95664d2_9614_4f35_a746_de8db63617e6);
 let meter_iid=GUID::from_u128(0xc02216f6_8c67_4b5b_9d00_d008e73e0064);
 let mut p=std::ptr::null_mut();
 let hr=unsafe{create(&cls,std::ptr::null_mut(),23,&iid,&mut p)};
 let en=check(hr,p,"IMMDeviceEnumerator")?;
 // IUnknown(0..2), EnumAudioEndpoints(3), GetDefaultAudioEndpoint(4).
 let get:unsafe extern "system" fn(Ptr,i32,i32,*mut Ptr)->i32=unsafe{std::mem::transmute(en.entry(4))};
 let mut p=std::ptr::null_mut();let hr=unsafe{get(en.0,0,0,&mut p)}; // eRender/eConsole
 let device=check(hr,p,"Default playback endpoint")?;
 // IMMDevice::Activate is slot 3.
 let activate:unsafe extern "system" fn(Ptr,*const GUID,u32,Ptr,*mut Ptr)->i32=unsafe{std::mem::transmute(device.entry(3))};
 let mut p=std::ptr::null_mut();let hr=unsafe{activate(device.0,&meter_iid,23,std::ptr::null_mut(),&mut p)};
 let meter=check(hr,p,"IAudioMeterInformation")?;
 let get_peak:unsafe extern "system" fn(Ptr,*mut f32)->i32=unsafe{std::mem::transmute(meter.entry(3))};
 let mut value=0f32;let hr=unsafe{get_peak(meter.0,&mut value)};
 if hr<0||!value.is_finite()||!(0.0..=1.0).contains(&value){return Err("Endpoint peak unavailable".into());}
 Ok(value)
}
#[derive(Default)]pub struct Meter{
 rx:Option<std::sync::mpsc::Receiver<Result<f32,String>>>,
 pub value:Option<Result<f32,String>>,last:Option<std::time::Instant>
}
impl Meter{
 pub fn reset(&mut self){self.value=None;self.rx=None;self.last=None;}
 pub fn tick(&mut self){
  if let Some(v)=self.rx.as_ref().and_then(|r|match r.try_recv(){Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Audio meter stopped".into())),_=>None}){
   self.rx=None;self.value=Some(v);
  }
  if self.rx.is_none()&&self.last.is_none_or(|t|t.elapsed()>=std::time::Duration::from_millis(250)){
   self.last=Some(std::time::Instant::now());let(tx,rx)=std::sync::mpsc::channel();self.rx=Some(rx);
   std::thread::spawn(move||{let _=tx.send(peak());});
  }
 }
}
