//! Optional Win32 ClipCursor bounded to explicitly selected foreground game.
 //! Does NOT inject, hide, move or scale the cursor. Armed only by user input.
use std::ffi::c_void;
type Window=*mut c_void;
#[repr(C)]#[derive(Clone,Copy,Debug,Default,Eq,PartialEq)]
pub struct Rect{pub left:i32,pub top:i32,pub right:i32,pub bottom:i32}
#[repr(C)]#[derive(Clone,Copy,Debug,Default)]struct Point{x:i32,y:i32}
#[link(name="user32")]unsafe extern "system"{
 fn GetForegroundWindow()->Window;
 fn GetWindowThreadProcessId(w:Window,pid:*mut u32)->u32;
 fn GetClientRect(w:Window,rect:*mut Rect)->i32;
 fn IsIconic(w:Window)->i32;
 fn ClientToScreen(w:Window,pos:*mut Point)->i32;
 fn GetClipCursor(rect:*mut Rect)->i32;
 fn ClipCursor(rect:*const Rect)->i32;
}
fn expected_rect(client:Rect,tl:Point,br:Point)->Option<Rect>{
 if client.left>=client.right||client.top>=client.bottom{return None;}
 let (l,t,r,b)=(tl.x,tl.y,br.x,br.y);
 if l>=r||t>=b||r.saturating_sub(l)>16384||b.saturating_sub(t)>16384{return None;}
 Some(Rect{left:l,top:t,right:r,bottom:b})
}
pub struct Lock{saved:Rect,applied:Rect,active:bool,pid:Option<u32>}
impl Default for Lock{fn default()->Self{Self{saved:Rect::default(),applied:Rect::default(),active:false,pid:None}}}
impl Lock{
 pub fn active(&self)->bool{self.active}
 pub fn pid(&self)->Option<u32>{self.pid}
 pub fn enable(&mut self,selected_pid:u32)->Result<(),String>{
  if selected_pid==0||selected_pid==std::process::id(){return Err("Select a real game window before locking cursor".into());}
  self.release();
  let hwnd=unsafe{GetForegroundWindow()};
  let mut owner=0;
  if hwnd.is_null()||unsafe{IsIconic(hwnd)}!=0||unsafe{GetWindowThreadProcessId(hwnd,&mut owner)}==0||owner!=selected_pid{
   return Err("Cursor lock requires the selected game to be in foreground".into());
  }
  let mut client=Rect::default();if unsafe{GetClientRect(hwnd,&mut client)}==0{return Err("Game client rectangle unavailable".into());}
  let mut tl=Point{x:client.left,y:client.top};let mut br=Point{x:client.right,y:client.bottom};
  if unsafe{ClientToScreen(hwnd,&mut tl)}==0||unsafe{ClientToScreen(hwnd,&mut br)}==0{return Err("Cannot map game client area to screen".into());}
  let requested=expected_rect(client,tl,br).ok_or("Game client area is invalid or oversized")?;
  let mut original=Rect::default();
  if unsafe{GetClipCursor(&mut original)}==0{return Err("Windows cursor bounds unavailable".into());}
  if unsafe{ClipCursor(&requested)}==0{return Err("Windows declined cursor clipping".into());}
  self.saved=original;self.applied=requested;self.pid=Some(selected_pid);self.active=true;Ok(())
 }
 pub fn tick(&mut self){
  if !self.active{return;}
  let mut owner=0;let hwnd=unsafe{GetForegroundWindow()};
  if hwnd.is_null()||unsafe{IsIconic(hwnd)}!=0||unsafe{GetWindowThreadProcessId(hwnd,&mut owner)}==0||Some(owner)!=self.pid{
   self.release();
   return;
  }
  let mut client=Rect::default();
  if unsafe{GetClientRect(hwnd,&mut client)}==0{self.release();return;}
  let mut tl=Point{x:client.left,y:client.top};
  let mut br=Point{x:client.right,y:client.bottom};
  if unsafe{ClientToScreen(hwnd,&mut tl)}==0||unsafe{ClientToScreen(hwnd,&mut br)}==0{self.release();return;}
  let Some(rect)=expected_rect(client,tl,br)else{self.release();return;};
  let mut current=Rect::default();
  if unsafe{GetClipCursor(&mut current)}==0{self.release();return;}
  if current!=self.applied{self.active=false;self.pid=None;return;}
  if rect!=self.applied{
   if unsafe{ClipCursor(&rect)}==0{self.release();return;}
   self.applied=rect;
  }
 }
 pub fn release(&mut self){
  if !self.active{return;}
  self.active=false;self.pid=None;
  let mut current=Rect::default();
  // If the game changed the clipping bounds, leave its new bounds untouched.
  if unsafe{GetClipCursor(&mut current)}!=0&&current==self.applied{unsafe{ClipCursor(&self.saved);}}
 }
}
impl Drop for Lock{fn drop(&mut self){self.release();}}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn client_clip_bounds_are_explicit_and_not_screen_relative(){
  let r=Rect{left:0,top:0,right:640,bottom:480};
  let out=expected_rect(r,Point{x:-1200,y:50},Point{x:-560,y:530}).unwrap();
  assert_eq!(out.left,-1200);assert_eq!(out.bottom,530);
  assert!(expected_rect(r,Point{x:0,y:0},Point{x:0,y:500}).is_none());
  assert!(expected_rect(r,Point{x:0,y:0},Point{x:50000,y:500}).is_none());
 }
 #[test]fn default_is_no_global_pointer_interference(){
  let mut lock=Lock::default();assert!(!lock.active());lock.tick();lock.release();assert_eq!(lock.pid(),None);
 }
}

#[cfg(test)]mod v101_resize_tests{
 use super::*;
 #[test]fn negative_monitor_and_resize_keep_virtual_screen_coordinates(){
  let before=expected_rect(Rect{left:0,top:0,right:1280,bottom:720},
   Point{x:-1920,y:0},Point{x:-640,y:720}).unwrap();
  let after=expected_rect(Rect{left:0,top:0,right:960,bottom:540},
   Point{x:-1600,y:120},Point{x:-640,y:660}).unwrap();
  assert_eq!(before.right,-640);assert_ne!(before,after);
  assert_eq!(after.left,-1600);assert_eq!(after.bottom,660);
 }
 #[test]fn zero_sized_and_absurd_client_are_rejected(){
  for bad in [Rect{left:0,top:0,right:0,bottom:600},Rect{left:0,top:0,right:40000,bottom:600}]{
   assert!(expected_rect(bad,Point{x:0,y:0},Point{x:bad.right,y:bad.bottom}).is_none());
  }
 }
}
