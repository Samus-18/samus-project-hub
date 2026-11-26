//! Manual opt-in, no-focus integration proof of actual Windows.Graphics.Capture.
//! Not part of the default test suite. No game window, injections, hooks or OS configuration.
use windows::Win32::Foundation::HWND;
use std::{ffi::c_void,time::{Duration,Instant},thread};
type H=*mut c_void;
#[repr(C)]
struct Msg{hwnd:H,message:u32,wparam:usize,lparam:isize,time:u32,pt:[i32;2],private:u32}
#[link(name="user32")]
unsafe extern "system"{
 fn CreateWindowExW(ex:u32,class:*const u16,text:*const u16,style:u32,
                    x:i32,y:i32,w:i32,h:i32,parent:H,menu:H,instance:H,param:H)->H;
 fn DestroyWindow(hwnd:H)->i32;
 fn ShowWindow(hwnd:H,cmd:i32)->i32;
 fn SetWindowPos(hwnd:H,after:H,x:i32,y:i32,w:i32,h:i32,flags:u32)->i32;
 fn InvalidateRect(hwnd:H,rect:*const c_void,erase:i32)->i32;
 fn UpdateWindow(hwnd:H)->i32;
 fn PeekMessageW(msg:*mut Msg,hwnd:H,min:u32,max:u32,action:u32)->i32;
 fn TranslateMessage(msg:*const Msg)->i32;
 fn DispatchMessageW(msg:*const Msg)->isize;
}
struct Fixture(H);
impl Fixture{
 fn new()->Result<Self,String>{
  let c="STATIC".encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
  let caption="Horizon safe WGC test fixture".encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
  // WS_EX_NOACTIVATE|TOOLWINDOW; WS_POPUP; never a foreground game/window.
  let hwnd=unsafe{CreateWindowExW(0x0800_0080,c.as_ptr(),caption.as_ptr(),0x8000_0000,30,100,352,208,
   std::ptr::null_mut(),std::ptr::null_mut(),std::ptr::null_mut(),std::ptr::null_mut())};
  if hwnd.is_null(){return Err("Cannot create WGC test fixture HWND".into());}
  unsafe{
   ShowWindow(hwnd,4); // SW_SHOWNOACTIVATE
   // HWND_BOTTOM + SWP_NOMOVE|NOSIZE|NOACTIVATE => do not cover the user's work.
   SetWindowPos(hwnd,1usize as H,0,0,0,0,0x0013);
   InvalidateRect(hwnd,std::ptr::null(),1);UpdateWindow(hwnd);
  }
  Ok(Self(hwnd))
 }
 fn tick(&self){
  unsafe{
   InvalidateRect(self.0,std::ptr::null(),1);
   UpdateWindow(self.0);
   let mut m=std::mem::MaybeUninit::<Msg>::uninit();
   for _ in 0..16{
    if PeekMessageW(m.as_mut_ptr(),std::ptr::null_mut(),0,0,1)==0{break;}
    TranslateMessage(m.as_ptr());DispatchMessageW(m.as_ptr());
   }
  }
 }
}
impl Drop for Fixture{fn drop(&mut self){unsafe{DestroyWindow(self.0);}}}
#[test]
#[ignore="Run manually to verify real WGC target capture, not during gameplay"]
fn actual_win32_window_capture_and_d3d11_filter(){
 let fixture=Fixture::new().expect("create hidden-from-focus sample");
 let mut capture=crate::native_wgc::WgcCapture::for_window_scaled_on(HWND(fixture.0),100,None)
  .expect("create WGC capture for independent window");
 let (w,h)=capture.dimensions();
 assert!(w>=300&&h>=180,"Unexpected WGC target size: {w}x{h}");
 let light=crate::lighting::LightTuning::default();
 let mut success=0u32;
 let started=Instant::now();
 while started.elapsed()<Duration::from_secs(5) {
  fixture.tick();
  match capture.next_filtered_frame(&[0],&light){
   Ok(Some(texture))=>{
    let mut desc=windows::Win32::Graphics::Direct3D11::D3D11_TEXTURE2D_DESC::default();
    unsafe{texture.GetDesc(&mut desc);}
    assert_eq!((desc.Width,desc.Height),capture.processed_size());
    success+=1;if success>=2{break;}
   },
   Ok(None)=>{},
   Err(e)=>panic!("WGC fixture real shader frame failure: {e}"),
  }
  thread::sleep(Duration::from_millis(40));
 }
 assert!(success>=1,"No WinRT WGC frames from independently created and visible non-activating fixture (5s)");
 println!("ACTUAL_WGC=PASS; WINDOW=352x208; FILTERED_FRAMES={success}; NO_GAME_INJECTION=true");
}

#[test]
#[ignore="Opt-in real WGC->D3D11 shader->DXGI Present against independent non-activating HWND"]
fn actual_wgc_filter_to_own_dxgi_swapchain(){
 let fixture=Fixture::new().expect("non-activating local test window");
 fixture.tick();
 let (presented,submitted)=crate::native_mirror::test_output_fixture_present(HWND(fixture.0))
  .expect("WGC capture, native HLSL effects and own DXGI Present");
 assert!(presented>=1&&submitted>=1,"Native own GPU output never presented a frame");
 println!("WGC_FULL_OUTPUT=PASS; submitted={submitted}; presented={presented}; no_game_change=true");
}
