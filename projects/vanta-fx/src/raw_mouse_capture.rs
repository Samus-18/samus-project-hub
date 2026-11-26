//! Vanta FX V138: user-opted raw relative mouse capture for permitted local games.
//! No hooks, DLL injection, clipboard access or global keyboard recording.
//! Windows input-sink receives relative deltas while the selected target is focused.
use std::{ffi::c_void,ptr};
type Hwnd=*mut c_void;
#[repr(C)]#[derive(Default,Clone,Copy)]struct Pt{x:i32,y:i32}
#[repr(C)]#[derive(Default,Clone,Copy)]struct Msg{
 hwnd:Hwnd,message:u32,wparam:usize,lparam:isize,time:u32,pt:Pt,lprivate:u32
}
#[repr(C)]#[derive(Clone,Copy)]struct RawDevice{
 usage_page:u16,usage:u16,flags:u32,target:Hwnd
}
#[repr(C)]#[derive(Clone,Copy,Default)]struct RawHeader{
 kind:u32,size:u32,device:usize,wparam:usize
}
#[repr(C)]#[derive(Clone,Copy,Default)]struct RawMouse{
 flags:u16,padding:u16,buttons:u32,raw_buttons:u32,x:i32,y:i32,extra:u32
}
#[repr(C)]#[derive(Clone,Copy,Default)]struct RawPacket{
 header:RawHeader,mouse:RawMouse
}
#[link(name="user32")]unsafe extern "system"{
 fn CreateWindowExW(ex:u32,class:*const u16,title:*const u16,
   style:u32,x:i32,y:i32,w:i32,h:i32,parent:Hwnd,menu:Hwnd,instance:Hwnd,param:*mut c_void)->Hwnd;
 fn DestroyWindow(hwnd:Hwnd)->i32;
 fn RegisterRawInputDevices(devices:*const RawDevice,count:u32,size:u32)->i32;
 fn GetRegisteredRawInputDevices(devices:*mut RawDevice,count:*mut u32,size:u32)->u32;
 fn PeekMessageW(msg:*mut Msg,hwnd:Hwnd,min:u32,max:u32,remove:u32)->i32;
 fn GetRawInputData(handle:Hwnd,command:u32,data:*mut c_void,size:*mut u32,header_size:u32)->u32;
 fn DefWindowProcW(hwnd:Hwnd,msg:u32,wparam:usize,lparam:isize)->isize;
}
fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(Some(0)).collect()}
fn parse(packet:&RawPacket)->Option<(i32,i32)>{
 // RIM_TYPEMOUSE and MOUSE_MOVE_RELATIVE. Ignore absolute-position tablets.
 if packet.header.kind!=0||packet.mouse.flags&1!=0{return None;}
 (packet.mouse.x!=0||packet.mouse.y!=0).then_some((packet.mouse.x,packet.mouse.y))
}

fn previous_mouse_registration()->Result<Option<RawDevice>,String>{
 let size=std::mem::size_of::<RawDevice>()as u32;
 let mut count=0u32;
 unsafe{GetRegisteredRawInputDevices(ptr::null_mut(),&mut count,size);}
 if count==0{return Ok(None);}
 if count>64{return Err("Unexpected raw input registration count".into());}
 let mut devices=vec![RawDevice{usage_page:0,usage:0,flags:0,target:ptr::null_mut()};count as usize];
 let got=unsafe{GetRegisteredRawInputDevices(devices.as_mut_ptr(),&mut count,size)};
 if got==u32::MAX{return Err("Cannot inspect existing raw input registration".into());}
 Ok(devices.into_iter().take(got as usize)
  .find(|d|d.usage_page==1&&d.usage==2))
}
pub(crate) struct Listener{hwnd:Hwnd,previous:Option<RawDevice>}
impl Listener{
 pub fn start()->Result<Self,String>{
  if std::mem::size_of::<RawHeader>()!=24||std::mem::size_of::<RawMouse>()!=24||
    std::mem::size_of::<RawPacket>()!=48{
    return Err("Unsupported RAWINPUT ABI".into());
  }
  // Win32 allows one registration per mouse TLC per process. Save and
  // restore Winit's existing registration instead of breaking main-window input.
  let previous=previous_mouse_registration()?;
  // Built-in STATIC window class, message-only hidden HWND. No desktop overlay.
  let cls=wide("STATIC");let name=wide("Vanta FX local raw mouse capture");
  let hwnd=unsafe{CreateWindowExW(0,cls.as_ptr(),name.as_ptr(),0,0,0,0,0,
    (-3isize) as Hwnd,ptr::null_mut(),ptr::null_mut(),ptr::null_mut())};
  if hwnd.is_null(){return Err("Windows refused hidden raw input listener".into());}
  let device=RawDevice{usage_page:1,usage:2,flags:0x00000100,target:hwnd};
  if unsafe{RegisterRawInputDevices(&device,1,std::mem::size_of::<RawDevice>()as u32)}==0{
   unsafe{DestroyWindow(hwnd);}
   return Err("Windows raw mouse registration failed".into());
  }
  Ok(Self{hwnd,previous})
 }
 pub fn poll(&mut self)->Result<Option<(i32,i32)>,String>{
  const WM_INPUT:u32=0x00ff;
  let mut x=0i32;let mut y=0i32;
  let mut count=0usize;
  loop{
   let mut msg=Msg::default();
   if unsafe{PeekMessageW(&mut msg,self.hwnd,WM_INPUT,WM_INPUT,1)}==0{break;}
   let mut aligned=[0u64;16];
   let mut length=(aligned.len()*8)as u32;
   let read=unsafe{GetRawInputData(msg.lparam as Hwnd,0x10000003,
    aligned.as_mut_ptr() as *mut c_void,&mut length,
    std::mem::size_of::<RawHeader>()as u32)};
   if read!=u32::MAX&&read>=std::mem::size_of::<RawPacket>()as u32{
    let packet=unsafe{&*(aligned.as_ptr() as *const RawPacket)};
    if let Some((dx,dy))=parse(packet){
     x=x.saturating_add(dx);y=y.saturating_add(dy);
    }
   }else{
    unsafe{DefWindowProcW(msg.hwnd,msg.message,msg.wparam,msg.lparam);}
    return Err("Windows raw input read failed".into());
   }
   // Required cleanup for foreground RIM_INPUT messages.
   unsafe{DefWindowProcW(msg.hwnd,msg.message,msg.wparam,msg.lparam);}
   count+=1;
   if count>=2048{break;} // Don't stall the recorder on input floods.
  }
  Ok((x!=0||y!=0).then_some((x,y)))
 }
}
impl Drop for Listener{
 fn drop(&mut self){
  if let Some(old)=self.previous{
   // This restoration is mandatory for the parent application's mouse stream.
   let _=unsafe{RegisterRawInputDevices(&old,1,std::mem::size_of::<RawDevice>() as u32)};
  }else{
   let remove=RawDevice{usage_page:1,usage:2,flags:0x00000001,target:ptr::null_mut()};
   unsafe{RegisterRawInputDevices(&remove,1,std::mem::size_of::<RawDevice>() as u32);}
  }
  unsafe{DestroyWindow(self.hwnd);}
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn windows_raw_input_abi(){assert_eq!(std::mem::size_of::<RawDevice>(),16);assert_eq!(std::mem::size_of::<RawHeader>(),24);
  assert_eq!(std::mem::size_of::<RawMouse>(),24);assert_eq!(std::mem::size_of::<RawPacket>(),48);assert_eq!(std::mem::size_of::<Msg>(),48);}
 #[test]fn only_relative_mouse_deltas_are_collected(){
  let mut p=RawPacket::default();p.mouse.x=-14;p.mouse.y=39;
  assert_eq!(parse(&p),Some((-14,39)));
  p.mouse.flags=1;assert_eq!(parse(&p),None);
  p.mouse.flags=0;p.header.kind=1;assert_eq!(parse(&p),None);
 }
}
