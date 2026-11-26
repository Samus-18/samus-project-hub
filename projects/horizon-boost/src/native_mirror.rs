//! Opt-in, original Direct3D11 LIVE GPU mirror of a selected game's HWND.
//! Captures via WGC, runs native HLSL on RX580, copies to OUR OWN independently
//! managed swapchain and displays in a normal separate Windows preview window.
//! It NEVER changes game resolution/window mode, never injects DLLs and does
//! NOT pretend to draw inside true exclusive fullscreen. It is NOT Magpie.
use std::{
    ffi::c_void,ptr,
    sync::{Arc,RwLock,Mutex,atomic::{AtomicBool,AtomicU32,AtomicU64,Ordering}},
    thread::{self,JoinHandle},
    time::{Duration,Instant},
};
use windows::{
    core::Interface,
    Win32::{
        Foundation::HWND,
        Graphics::{
            Dxgi::{
                CreateDXGIFactory1,IDXGIFactory2,IDXGIOutput,
                DXGI_PRESENT,DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT,
            },
            Direct3D11::{ID3D11Resource,ID3D11Texture2D},
        },
    },
};
type H=*mut c_void;
const WC:&str="HorizonBoostNativeMirrorV18";
/// Only changes the pointer drawn when hovered over the separate preview HWND.
/// No operating-system cursor scheme changes, no game cursor injection.
static OWN_CURSOR_PERCENT:AtomicU32=AtomicU32::new(100);
pub fn configure_preview_cursor_scale(percent:u8){
 OWN_CURSOR_PERCENT.store(if matches!(percent,100|150|200){u32::from(percent)}else{100},Ordering::Relaxed);
}
fn cursor_pixels(base:u32,percent:u32)->Option<i32>{
 if !matches!(percent,150|200)||!(8..=64).contains(&base){return None;}
 Some(((base*percent+50)/100).clamp(16,128) as i32)
}
fn selected_process_foreground(pid:u32,mirror:H)->bool{
    if pid==0{return false;}
    let hwnd=unsafe{GetForegroundWindow()};
    if hwnd.is_null()||hwnd==mirror{return false;}
    let mut owner=0;
    (unsafe{GetWindowThreadProcessId(hwnd,&mut owner)})!=0&&owner==pid
}
fn borderless_window_flags()->u32{
    // WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_TOPMOST.
    // No WS_EX_LAYERED: regular flip-sequential DXGI backbuffers require a
    // normal opaque HWND on some Windows builds.
    0x0800_0000 | 0x20 | 0x08
}
#[cfg(test)]mod v97_borderless_tests{
    #[test]fn transparent_nonactivating_popup_without_layered_flags(){
        let flags=super::borderless_window_flags();
        assert_eq!(flags&0x0800_0000,0x0800_0000);
        assert_eq!(flags&0x20,0x20);
        assert_eq!(flags&0x08,0x08);
        assert_eq!(flags&0x0008_0000,0);
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum MirrorMode{SeparateWindow,CompositedInPlace,BorderlessScreen,DesktopDuplication}

#[repr(C)]
struct WindowClass{
    style:u32,
    proc:Option<unsafe extern "system" fn(H,u32,usize,isize)->isize>,
    cls_extra:i32,wnd_extra:i32,instance:H,icon:H,cursor:H,brush:H,
    menu:*const u16,class:*const u16,
}
#[repr(C)]
struct Rect{left:i32,top:i32,right:i32,bottom:i32}
#[repr(C)]
struct Point{x:i32,y:i32}
#[repr(C)]
struct Message{hwnd:H,msg:u32,wparam:usize,lparam:isize,time:u32,pt:Point,private:u32}
#[link(name="user32")]
unsafe extern "system"{
    fn RegisterClassW(class:*const WindowClass)->u16;
    fn CreateWindowExW(ex:u32,class:*const u16,title:*const u16,style:u32,
        x:i32,y:i32,w:i32,h:i32,parent:H,menu:H,instance:H,param:H)->H;
    fn DefWindowProcW(hwnd:H,msg:u32,w:usize,l:isize)->isize;
    fn DestroyWindow(hwnd:H)->i32;
    fn IsWindow(hwnd:H)->i32;
    fn ShowWindow(hwnd:H,cmd:i32)->i32;
    fn EnumWindows(callback:unsafe extern "system" fn(H,isize)->i32,state:isize)->i32;
    fn IsWindowVisible(hwnd:H)->i32;
    fn GetWindowThreadProcessId(hwnd:H,pid:*mut u32)->u32;
    fn GetWindowRect(hwnd:H,out:*mut Rect)->i32;
    fn GetClientRect(hwnd:H,out:*mut Rect)->i32;
    fn GetForegroundWindow()->H;
    fn SetWindowDisplayAffinity(hwnd:H,affinity:u32)->i32;
    fn GetWindowDisplayAffinity(hwnd:H,affinity:*mut u32)->i32;
    fn WindowFromPoint(point:Point)->H;
    fn IsIconic(hwnd:H)->i32;
    fn SetWindowPos(hwnd:H,insert_after:H,x:i32,y:i32,cx:i32,cy:i32,flags:u32)->i32;
    fn SetLayeredWindowAttributes(hwnd:H,color:u32,alpha:u8,flags:u32)->i32;
    fn GetWindowLongW(hwnd:H,index:i32)->i32;
    fn GetWindowTextW(hwnd:H,title:*mut u16,capacity:i32)->i32;
    fn GetSystemMetrics(index:i32)->i32;
    fn LoadImageW(instance:H,name:*const u16,kind:u32,width:i32,height:i32,flags:u32)->H;
    fn SetCursor(cursor:H)->H;

    fn PeekMessageW(msg:*mut Message,hwnd:H,min:u32,max:u32,remove:u32)->i32;
    fn TranslateMessage(msg:*const Message)->i32;
    fn DispatchMessageW(msg:*const Message)->isize;
    fn GetDC(hwnd:H)->H;
    fn ReleaseDC(hwnd:H,dc:H)->i32;
    fn FillRect(dc:H,rect:*const Rect,brush:H)->i32;
}
#[link(name="gdi32")]
unsafe extern "system"{
    fn CreateSolidBrush(rgb:u32)->H;
    fn DeleteObject(obj:H)->i32;
}
#[link(name="kernel32")]
unsafe extern "system"{
    fn GetModuleHandleW(module:*const u16)->H;
}
unsafe extern "system" fn wnd_proc(hwnd:H,msg:u32,w:usize,l:isize)->isize{
    if msg==0x0010{unsafe{DestroyWindow(hwnd);}return 0;}
    // Only the shader-glass HWND is click-through. Other windows that use
    // this class (synthetic source or detached mirror) must retain normal
    // hit-testing. Previously the synthetic source was also transparent.
    let glass=unsafe{GetWindowLongW(hwnd,-20)} as u32 &0x20!=0;
    if !glass&&msg==0x0020 {
        // WM_SETCURSOR: optional genuinely sized OS arrow only over own
        // separate GPU preview. Never modify the original game window.
        let percent=OWN_CURSOR_PERCENT.load(Ordering::Relaxed);
        let system=unsafe{GetSystemMetrics(13)}; // SM_CXCURSOR
        if let Some(px)=cursor_pixels(system.max(0) as u32,percent){
            let mut title=[0u16;160];
            let n=unsafe{GetWindowTextW(hwnd,title.as_mut_ptr(),title.len() as i32)};
            if n>0&&String::from_utf16_lossy(&title[..n as usize]).contains("Native GPU Live Preview"){
                const IMAGE_CURSOR:u32=2;const LR_SHARED:u32=0x8000;
                let standard_arrow=32512usize as *const u16;
                let cursor=unsafe{LoadImageW(ptr::null_mut(),standard_arrow,IMAGE_CURSOR,px,px,LR_SHARED)};
                if !cursor.is_null(){unsafe{SetCursor(cursor);}return 1;}
            }
        }
    }
    if glass&&msg==0x0084{return -1;} // HTTRANSPARENT
    if glass&&msg==0x0021{return 3;} // MA_NOACTIVATE
    unsafe{DefWindowProcW(hwnd,msg,w,l)}
}
fn wide(text:&str)->Vec<u16>{text.encode_utf16().chain(Some(0)).collect()}
struct OwnWindow{hwnd:H}
impl OwnWindow{
    fn new(label:&str,x:i32,y:i32,w:i32,h:i32,small:bool)->Result<Self,String>{
        let module=unsafe{GetModuleHandleW(ptr::null())};
        let name=wide(WC);
        let class=WindowClass{
            style:0,proc:Some(wnd_proc),cls_extra:0,wnd_extra:0,instance:module,
            icon:ptr::null_mut(),cursor:ptr::null_mut(),brush:ptr::null_mut(),
            menu:ptr::null(),class:name.as_ptr(),
        };
        // A class can be reused for a synthetic local source and mirror
        // within one thread/process. Returning 0 here may mean "exists".
        let _=unsafe{RegisterClassW(&class)};
        let title=wide(label);
        const WS_EX_NOACTIVATE:u32=0x0800_0000;
        const WS_POPUP:u32=0x8000_0000;
        const WS_OVERLAPPEDWINDOW:u32=0x00cf_0000;
        let style=if small{WS_POPUP}else{WS_OVERLAPPEDWINDOW};
        let hwnd=unsafe{CreateWindowExW(WS_EX_NOACTIVATE,name.as_ptr(),
            title.as_ptr(),style,x,y,w,h,ptr::null_mut(),ptr::null_mut(),
            module,ptr::null_mut())};
        if hwnd.is_null(){return Err("Cannot create private native GPU mirror HWND".into());}
        Ok(Self{hwnd})
    }
    fn borderless(x:i32,y:i32,w:i32,h:i32)->Result<Self,String>{
        if w<160||h<90||w>8192||h>8192{return Err("Invalid borderless monitor dimensions".into());}
        let module=unsafe{GetModuleHandleW(ptr::null())};
        let name=wide(WC);
        let class=WindowClass{style:0,proc:Some(wnd_proc),cls_extra:0,wnd_extra:0,
            instance:module,icon:ptr::null_mut(),cursor:ptr::null_mut(),brush:ptr::null_mut(),
            menu:ptr::null(),class:name.as_ptr()};
        let _=unsafe{RegisterClassW(&class)};
        // Click-through topmost popup presents our own GPU output. Game HWND
        // styles and swapchain remain unchanged. Not exclusive-fullscreen proof.
        let flags=borderless_window_flags();
        let title=wide("HORIZON BOOST - Borderless GPU Output (Experimental)");
        let hwnd=unsafe{CreateWindowExW(flags,name.as_ptr(),title.as_ptr(),0x8000_0000,
            x,y,w,h,ptr::null_mut(),ptr::null_mut(),module,ptr::null_mut())};
        if hwnd.is_null(){return Err("Cannot create own borderless GPU output".into());}
        Ok(Self{hwnd})
    }
    fn show(&self){unsafe{ShowWindow(self.hwnd,8);}} // show without activation
    /// Same-screen *composited* shader view: overlay covers exactly the
    /// original HWND rectangle. Never changes source's style, position or
    /// swapchain. Does not pretend to bypass true exclusive FSE.
    fn composited(target:HWND)->Result<Self,String>{
        let mut rc=Rect{left:0,top:0,right:0,bottom:0};
        if unsafe{GetWindowRect(target.0,&mut rc)}==0{
            return Err("Selected window rectangle unavailable".into());
        }
        let w=rc.right-rc.left;
        let h=rc.bottom-rc.top;
        if w<160||h<90||w>8192||h>8192{
            return Err(format!("Invalid target rectangle {w}x{h}"));
        }
        let module=unsafe{GetModuleHandleW(ptr::null())};
        let name=wide(WC);
        let cls=WindowClass{
            style:0,proc:Some(wnd_proc),cls_extra:0,wnd_extra:0,instance:module,
            icon:ptr::null_mut(),cursor:ptr::null_mut(),brush:ptr::null_mut(),
            menu:ptr::null(),class:name.as_ptr(),
        };
        let _=unsafe{RegisterClassW(&cls)};
        let title=wide("HORIZON BOOST - Native Composited Shader Glass");
        // DO NOT apply windowed-to-fullscreen transformation to the target.
        // The window is exactly the target's current rectangle, not a new
        // fullscreen output. Z-order is managed by the compositor where able.
        const WS_EX_LAYERED:u32=0x0008_0000;
        const WS_EX_TRANSPARENT:u32=0x0000_0020;
        const WS_EX_TOPMOST:u32=0x0000_0008;
        const WS_EX_NOACTIVATE:u32=0x0800_0000;
        const WS_POPUP:u32=0x8000_0000;
        let ex=WS_EX_LAYERED|WS_EX_TRANSPARENT|WS_EX_TOPMOST|WS_EX_NOACTIVATE;
        let hwnd=unsafe{CreateWindowExW(ex,name.as_ptr(),title.as_ptr(),WS_POPUP,
            rc.left,rc.top,w,h,ptr::null_mut(),ptr::null_mut(),module,ptr::null_mut())};
        if hwnd.is_null(){return Err("Composited nonactivating glass HWND creation failed".into());}
        let output=Self{hwnd};
        if unsafe{SetLayeredWindowAttributes(hwnd,0,255,2)}==0{
            return Err("Windows did not allow per-window layered alpha".into());
        }
        Ok(output)
    }
}
fn source_rect(target:HWND)->Result<Rect,String>{
    let mut rc=Rect{left:0,top:0,right:0,bottom:0};
    if unsafe{GetWindowRect(target.0,&mut rc)}==0{
        return Err("Source window rectangle is no longer accessible".into());
    }
    if rc.right<=rc.left||rc.bottom<=rc.top{
        return Err("Source window rectangle collapsed".into());
    }
    Ok(rc)
}
/// Updates only our own overlay, never game geometry. Returns false when
/// game is not in foreground; hide to avoid obscuring Alt+Tab and menus.
#[derive(Default)]
struct GlassSyncState{
    rectangle:Option<[i32;4]>,
    visible:bool,
    last_z_update:Option<Instant>,
}
fn sync_glass(target:HWND,glass:&OwnWindow,proof:bool,ready:bool,
    state:&mut GlassSyncState)->Result<bool,String>{
    let target_rect=source_rect(target)?;
    let foreground=unsafe{GetForegroundWindow()};
    let mut source_pid=0u32;
    let mut active_pid=0u32;
    unsafe{
        GetWindowThreadProcessId(target.0,&mut source_pid);
        GetWindowThreadProcessId(foreground,&mut active_pid);
    }
    let showing=proof||(unsafe{IsIconic(target.0)}==0&&active_pid==source_pid);
    if !showing{
        if state.visible{unsafe{ShowWindow(glass.hwnd,0);}}
        state.visible=false;
        return Ok(false);
    }
    let bounds=[target_rect.left,target_rect.top,target_rect.right,target_rect.bottom];
    let needs_update=state.visible!=ready||state.rectangle!=Some(bounds)||
        state.last_z_update.is_none_or(|t|t.elapsed()>Duration::from_millis(1400));
    if needs_update {
        let w=target_rect.right-target_rect.left;
        let h=target_rect.bottom-target_rect.top;
        const SWP_NOACTIVATE:u32=0x10;
        const SWP_SHOWWINDOW:u32=0x40;
        if unsafe{SetWindowPos(glass.hwnd,(-1isize) as H,
            target_rect.left,target_rect.top,w,h,SWP_NOACTIVATE|if ready{SWP_SHOWWINDOW}else{0})}==0{
            return Err("Cannot align shader glass with original game rectangle".into());
        }
        state.visible=ready;
        state.rectangle=Some(bounds);
        state.last_z_update=Some(Instant::now());
    }
    Ok(true)
}


/// Aspect-preserving window upscaling on the GPU. Avoid a CPU texture upload.
pub(crate) fn fit_game_to_monitor(src:(u32,u32),monitor:(u32,u32),cap:f32)->(u32,u32){
 let (sw,sh)=src;let (mw,mh)=monitor;
 if sw==0||sh==0||mw==0||mh==0{return src;}
 let ratio=(mw as f32/sw as f32).min(mh as f32/sh as f32).min(cap.clamp(1.,2.));
 let w=(sw as f32*ratio).round().max(1.) as u32;
 let h=(sh as f32*ratio).round().max(1.) as u32;
 (w.min(mw),h.min(mh))
}
pub(crate) fn center_game_output(monitor:(i32,i32,u32,u32),output:(u32,u32))->(i32,i32,i32,i32){
 let (x,y,mw,mh)=monitor;let(w,h)=output;
 (x+((mw.saturating_sub(w))/2) as i32,y+((mh.saturating_sub(h))/2) as i32,w as i32,h as i32)
}
#[cfg(test)]mod magpie_fit_tests{
 use super::*;
 #[test]fn fullscreen_16_9_and_secondary_monitor_coords(){
  assert_eq!(fit_game_to_monitor((1280,720),(1920,1080),2.),(1920,1080));
  assert_eq!(center_game_output((-1920,0,1920,1080),(1920,1080)),(-1920,0,1920,1080));
 }
 #[test]fn four_by_three_is_centered_not_stretched(){
  assert_eq!(fit_game_to_monitor((800,600),(1920,1080),2.),(1440,1080));
  assert_eq!(center_game_output((1920,0,1920,1080),(1440,1080)),(2160,0,1440,1080));
 }
 #[test]fn bound_gpu_scale_and_allow_downscale(){
  assert_eq!(fit_game_to_monitor((1280,720),(1920,1080),1.25),(1600,900));
  assert_eq!(fit_game_to_monitor((3840,2160),(1920,1080),2.),(1920,1080));
  assert_eq!(fit_game_to_monitor((0,720),(1920,1080),2.),(0,720));
 }
}
#[derive(Default)]
struct StableMonitorWatch {
    candidate: Option<Option<(i32,i32,u32,u32)>>,
    consecutive: u8,
}
impl StableMonitorWatch {
    /// Debounce screen moves and transient failures; a different monitor
    /// needs new swapchain dimensions and GPU resampling resources.
    fn changed(&mut self, initial: Option<(i32,i32,u32,u32)>,
               observed: Option<(i32,i32,u32,u32)>) -> bool {
        if initial == observed {
            self.candidate=None;
            self.consecutive=0;
            return false;
        }
        if self.candidate == Some(observed) {
            self.consecutive=self.consecutive.saturating_add(1);
        } else {
            self.candidate=Some(observed);
            self.consecutive=1;
        }
        self.consecutive>=2
    }
}
#[cfg(test)]
mod v134_monitor_watch_tests {
    use super::*;
    #[test]
    fn default_display_does_not_fabricate_a_topology_change() {
        let mut w=StableMonitorWatch::default();
        assert!(!w.changed(None,None));
        assert!(!w.changed(Some((0,0,1920,1080)),Some((0,0,1920,1080))));
        assert!(!w.changed(Some((-1920,0,1920,1080)),Some((-1920,0,1920,1080))));
    }
    #[test]
    fn moving_to_negative_x_monitor_rebuilds_only_after_stable_observations() {
        let mut w=StableMonitorWatch::default();
        let primary=Some((0,0,1920,1080));
        let secondary=Some((-1920,0,1920,1080));
        assert!(!w.changed(primary,secondary));
        assert!(!w.changed(primary,primary));
        assert!(!w.changed(primary,secondary));
        assert!(w.changed(primary,secondary));
    }
    #[test]
    fn lost_monitor_and_resolution_change_are_stable_events() {
        let mut w=StableMonitorWatch::default();
        let original=Some((0,0,2560,1440));
        assert!(!w.changed(original,None));
        assert!(w.changed(original,None));
        let mut second=StableMonitorWatch::default();
        assert!(!second.changed(original,Some((0,0,1920,1080))));
        assert!(second.changed(original,Some((0,0,1920,1080))));
    }
}

#[derive(Default)]
struct GameWindowAlignment{
    bounds:Option<[i32;4]>,last_sync:Option<Instant>
}
/// Move only Horizon's own non-layered DXGI output over the selected window.
/// Game geometry, style, input handling and swapchain are never changed.
fn align_gpu_output(target:HWND,output:&OwnWindow,extent:(u32,u32),
    state:&mut GameWindowAlignment)->Result<(),String>{
    let _=source_rect(target)?;
    let bounds=crate::win::window_monitor_bounds(target.0)
        .ok_or("Selected game monitor is unavailable")?;
    let (x,y,w,h)=center_game_output(bounds,extent);
    let b=[x,y,x+w,y+h];
    if state.bounds==Some(b)&&state.last_sync.is_some_and(|t|t.elapsed()<Duration::from_millis(950)){
        return Ok(());
    }
    const SWP_NOACTIVATE:u32=0x0010;
    const HWND_TOPMOST:isize=-1;
    if unsafe{SetWindowPos(output.hwnd,HWND_TOPMOST as H,x,y,w,h,SWP_NOACTIVATE)}==0{
        return Err("Could not center GPU output on selected game's monitor".into());
    }
    state.bounds=Some(b);
    state.last_sync=Some(Instant::now());
    Ok(())
}

impl Drop for OwnWindow{
    fn drop(&mut self){if !self.hwnd.is_null()&&unsafe{IsWindow(self.hwnd)}!=0{
        unsafe{DestroyWindow(self.hwnd);}
    }}
}
unsafe extern "system" fn match_window(hwnd:H,ptr:isize)->i32{
    let state=unsafe{&mut*(ptr as *mut (u32,H,i64))};
    if unsafe{IsWindowVisible(hwnd)}==0||unsafe{IsIconic(hwnd)}!=0||crate::win::is_dwm_cloaked(hwnd){return 1;}
    let mut owner=0u32;
    unsafe{GetWindowThreadProcessId(hwnd,&mut owner);}
    if owner!=state.0{return 1;}
    let mut rect=Rect{left:0,top:0,right:0,bottom:0};
    if unsafe{GetWindowRect(hwnd,&mut rect)}==0{return 1;}
    let area=(rect.right-rect.left).max(0) as i64*(rect.bottom-rect.top).max(0) as i64;
    if area>state.2&&area>=160*90 {
        state.1=hwnd;state.2=area;
    }
    1
}
fn hwnd_for_pid(pid:u32)->Result<HWND,String>{
    if pid==0||pid==std::process::id(){
        return Err("Native mirror must target a distinct selected process".into());
    }
    let mut state=(pid,ptr::null_mut::<c_void>(),0i64);
    unsafe{EnumWindows(match_window,&mut state as *mut _ as isize);}
    if state.1.is_null(){Err(format!("Selected game PID {pid} has no suitable visible HWND"))}
    else{Ok(HWND(state.1))}
}
fn pump_messages(){
    let mut m=Message{
        hwnd:ptr::null_mut(),msg:0,wparam:0,lparam:0,time:0,
        pt:Point{x:0,y:0},private:0,
    };
    while unsafe{PeekMessageW(&mut m,ptr::null_mut(),0,0,1)}!=0{
        unsafe{TranslateMessage(&m);DispatchMessageW(&m);}
    }
}
fn paint_test_source(hwnd:H,iteration:u32){
    let dc=unsafe{GetDC(hwnd)};
    if dc.is_null(){return;}
    let red=((iteration*13)%200+35) as u8;
    let green=((iteration*29)%190+40) as u8;
    let blue=200u8;
    let rgb=red as u32|((green as u32)<<8)|((blue as u32)<<16);
    let brush=unsafe{CreateSolidBrush(rgb)};
    let rect=Rect{left:0,top:0,right:240,bottom:130};
    if !brush.is_null(){
        unsafe{FillRect(dc,&rect,brush);DeleteObject(brush);}
    }
    unsafe{ReleaseDC(hwnd,dc);}
}

/// Source frame belongs to target WGC HWND, not the screen or our output.
/// A separate *windowed* visible preview is always used; user game untouched.
/// Capture-side adaptive filter pass budget; never claims to speed up a game.
/// Hysteresis prevents the GPU work from switching rapidly near thresholds.
/// 60Hz captures may graduate from 1 -> 2 passes; 75Hz+ are needed for 3.
/// Slower sources stay at one pass and retain the original game frame rate.
fn adaptive_pass_limit(current:usize,throughput:f32,available:usize)->usize{
    if available==0{return 0;}
    if !throughput.is_finite()||throughput<0.0{return 1;}
    let current=current.clamp(1,available.min(3));
    let next=match current{
        1 if throughput>=56.0=>2,
        2 if throughput>=75.0=>3,
        3 if throughput<52.0=>2,
        2 if throughput<39.0=>1,
        _=>current,
    };
    next.min(available)
}


/// A Present API success HRESULT may still mean occluded, not submitted.
/// Never count an occluded/busy output frame as delivered DXGI output FPS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DupPresentAction { Count, Retry, Recover }
fn duplication_present_action(hr:i32)->DupPresentAction {
    use crate::present_status::Outcome;
    match crate::present_status::classify(hr) {
        Outcome::Submitted => DupPresentAction::Count,
        Outcome::Occluded | Outcome::OtherStatus | Outcome::Busy => DupPresentAction::Retry,
        Outcome::Failure => DupPresentAction::Recover,
    }
}
#[cfg(test)]
mod v135_duplication_present_tests {
    use super::*;
    #[test] fn only_s_ok_counts_as_real_output() {
        assert_eq!(duplication_present_action(0),DupPresentAction::Count);
        assert_ne!(duplication_present_action(0x087A0001),DupPresentAction::Count);
        assert_ne!(duplication_present_action(1),DupPresentAction::Count);
    }
    #[test] fn positive_occlusion_and_busy_reset_pacing_without_fake_fps() {
        assert_eq!(duplication_present_action(0x087A0001),DupPresentAction::Retry);
        assert_eq!(duplication_present_action(1),DupPresentAction::Retry);
        assert_eq!(duplication_present_action(0x887A000Au32 as i32),DupPresentAction::Retry);
    }
    #[test] fn device_removed_or_other_failure_rebuilds_capture() {
        assert_eq!(duplication_present_action(0x887A0005u32 as i32),DupPresentAction::Recover);
        assert_eq!(duplication_present_action(0x887A0006u32 as i32),DupPresentAction::Recover);
    }
}

fn run_desktop_duplication(target:HWND,stages:&[usize],running:&AtomicBool,
    frames:&AtomicU64,lighting:&RwLock<crate::lighting::LightTuning>,
    health:&crate::capture_health::Health,status:&impl Fn(String),
    cap:Option<&AtomicU32>)->Result<u64,String>{
    // Desktop duplication sees the entire monitor. Never start for a window
    // smaller than that monitor, and pause whenever selected PID loses focus.
    let mut rect=Rect{left:0,top:0,right:0,bottom:0};
    if unsafe{GetWindowRect(target.0,&mut rect)}==0{return Err("DXGI fallback game rectangle unavailable".into());}
    let owner=unsafe{let mut id=0;GetWindowThreadProcessId(target.0,&mut id);id};
    if owner==0{return Err("DXGI fallback game window closed".into());}
    let source=crate::native_dup_output::DupSource::new([rect.left,rect.top,rect.right,rect.bottom])?;
    status(format!("DXGI Desktop Duplication fallback: monitor-filling game {}x{}, pure GPU pipeline, own output, no game injection",source.width,source.height));
    let bounds=source.monitor;
    let win=OwnWindow::borderless(bounds[0],bounds[1],source.width as i32,source.height as i32)?;
    const EXCLUDE_FROM_CAPTURE:u32=0x11; // WDA_EXCLUDEFROMCAPTURE, Windows 10 2004+
    let mut protected=0u32;
    if unsafe{SetWindowDisplayAffinity(win.hwnd,EXCLUDE_FROM_CAPTURE)}==0 ||
       unsafe{GetWindowDisplayAffinity(win.hwnd,&mut protected)}==0 ||
       protected!=EXCLUDE_FROM_CAPTURE{
        return Err("DXGI fallback refused: own output cannot be excluded from desktop capture; avoids GPU feedback loop".into());
    }
    status("DXGI fallback protected own output with WDA_EXCLUDEFROMCAPTURE (driver behavior still requires live verification)".into());
    let factory:IDXGIFactory2=unsafe{CreateDXGIFactory1()}.map_err(|e|format!("DXGI fallback factory: {e}"))?;
    let desc=crate::native_present::swapchain_descriptor(source.width,source.height);
    let chain=unsafe{factory.CreateSwapChainForHwnd(&source.device,HWND(win.hwnd),&desc,None,None::<&IDXGIOutput>)}
        .map_err(|e|format!("DXGI fallback flip output: {e}"))?;
    let back:ID3D11Texture2D=unsafe{chain.GetBuffer(0)}
        .map_err(|e|format!("DXGI fallback backbuffer: {e}"))?;
    let destination:ID3D11Resource=back.cast().map_err(|e|e.to_string())?;
    health.scale_input.store((u64::from(source.width)<<32)|u64::from(source.height),Ordering::Relaxed);
    health.scale_output.store((u64::from(source.width)<<32)|u64::from(source.height),Ordering::Relaxed);
    let mut shown=false;
    let mut submitted=0u64;
    let mut cached_tuning=lighting.read().map(|g|g.clone()).unwrap_or_default();
    let mut deadline=crate::preview_pacer::PresentDeadline::default();
    while running.load(Ordering::Acquire)&&unsafe{IsWindow(win.hwnd)}!=0{
        health.beat();pump_messages();
        if unsafe{IsWindow(target.0)}==0{return Err("DXGI fallback game window closed".into());}
        let mut current=0;
        if unsafe{GetWindowThreadProcessId(target.0,&mut current)}==0||current!=owner{
            return Err("DXGI fallback game PID/window identity changed".into());
        }
        let mut game=Rect{left:0,top:0,right:0,bottom:0};
        if unsafe{GetWindowRect(target.0,&mut game)}==0||
            !crate::native_dup_output::covers([game.left,game.top,game.right,game.bottom],bounds){
            return Err("DXGI fallback game stopped filling its capture monitor; output safely closed".into());
        }
        let active=unsafe{IsIconic(target.0)}==0&&unsafe{IsWindowVisible(target.0)}!=0
            &&selected_process_foreground(owner,win.hwnd);
        if !active{
            if shown{unsafe{ShowWindow(win.hwnd,0);}}
            shown=false;deadline.reset();health.paused.store(true,Ordering::Relaxed);health.clear_timings();
            std::thread::sleep(Duration::from_millis(85));continue;
        }
        health.paused.store(false,Ordering::Relaxed);
        let limit=cap.map_or(0,|c|c.load(Ordering::Relaxed));
        let remain=deadline.remaining(Instant::now(),limit);
        if crate::preview_pacer::wait_step(remain){continue;}
        if let Ok(guard)=lighting.read(){
            if *guard!=cached_tuning{cached_tuning.clone_from(&guard);}
        }
        let live=health.live_stages.read().ok();
        let active_stages=live.as_ref().and_then(|g|g.as_ref().cloned());
        let stages=active_stages.as_deref().unwrap_or(stages);
        let start=Instant::now();
        let processed=match source.next(stages,&cached_tuning){
            Ok(Some(texture))=>texture,
            Ok(None)=>{health.no_frame_polls.fetch_add(1,Ordering::Relaxed);std::thread::sleep(Duration::from_millis(4));continue;},
            Err(e)=>return Err(format!("DXGI_DESKTOP_DUPLICATION_LOST: {e}")),
        };
        if !selected_process_foreground(owner,win.hwnd){
            if shown{unsafe{ShowWindow(win.hwnd,0);}}
            shown=false;deadline.reset();health.paused.store(true,Ordering::Relaxed);continue;
        }
        let src:ID3D11Resource=processed.cast().map_err(|e|e.to_string())?;
        unsafe{source.context.CopyResource(&destination,&src);}
        let present_start=Instant::now();
        let vsync=if health.vsync.load(Ordering::Relaxed){1}else{0};
        let result=unsafe{chain.Present(vsync,windows::Win32::Graphics::Dxgi::DXGI_PRESENT(0))};
        let action=duplication_present_action(result.0);
        if action==DupPresentAction::Recover {
            return Err(format!("DXGI_DESKTOP_DUPLICATION_LOST: own flip Present HRESULT 0x{:08X}",result.0 as u32));
        }
        if !shown{win.show();shown=true;}
        if action==DupPresentAction::Retry {
            // DXGI_STATUS_OCCLUDED and other non-submitted statuses are not
            // successfully presented frames, even when HRESULT.ok() succeeds.
            health.non_submitted.fetch_add(1,Ordering::Relaxed);
            health.clear_timings();
            deadline.reset();
            std::thread::sleep(Duration::from_millis(40));
            continue;
        }
        deadline.submitted(start,Instant::now(),limit);
        health.effective_passes.store(source.effective_stages() as u64,Ordering::Relaxed);
        health.frame(start.elapsed(),present_start.elapsed());
        frames.fetch_add(1,Ordering::Relaxed);submitted+=1;
    }
    Ok(submitted)
}

fn run_capture(target:HWND,stages:&[usize],mode:MirrorMode,stop:&AtomicBool,
    frames_seen:&AtomicU64,generated:&AtomicU64,factor:u8,
    lighting:&Arc<RwLock<crate::lighting::LightTuning>>,
    status:impl Fn(String),max_frames:Option<u64>)->Result<u64,String>{
    run_capture_scaled(target,stages,mode,stop,frames_seen,generated,factor,lighting,
        0,1.0,0,status,max_frames)
}
fn run_capture_scaled(target:HWND,stages:&[usize],mode:MirrorMode,stop:&AtomicBool,
    frames_seen:&AtomicU64,generated:&AtomicU64,factor:u8,
    lighting:&Arc<RwLock<crate::lighting::LightTuning>>,
    upscale_mode:u8,upscale_ratio:f32,quality:u8,
    status:impl Fn(String),max_frames:Option<u64>)->Result<u64,String>{
    run_capture_scaled_paced(target,stages,mode,stop,frames_seen,generated,factor,
        lighting,upscale_mode,upscale_ratio,quality,status,max_frames,None,100,None)
}
fn run_capture_scaled_paced(target:HWND,stages:&[usize],mode:MirrorMode,
    stop:&AtomicBool,frames_seen:&AtomicU64,generated:&AtomicU64,factor:u8,
    lighting:&Arc<RwLock<crate::lighting::LightTuning>>,
    upscale_mode:u8,upscale_ratio:f32,quality:u8,
    status:impl Fn(String),max_frames:Option<u64>,
    live_cap:Option<&AtomicU32>,processing_percent:u8,health:Option<&crate::capture_health::Health>)->Result<u64,String>{
    crate::framegen::validate_factor(factor)?;
    // Vendor mode 8 (FSR1), 9 (NIS) and high-quality mode 10 (Lanczos3)
    // must be accepted by LIVE WGC, not only the standalone GPU shader proof.
    let max_mode=(crate::spatial::MODES.len()-1) as u8;
    if upscale_mode>max_mode{
        return Err(format!("Unsupported independent GPU scaling mode {upscale_mode} (max {max_mode})"));
    }

    if mode==MirrorMode::DesktopDuplication{return Err("Use explicit foreground DXGI duplication worker, not WGC".into());}
    let mut wgc=crate::native_wgc::WgcCapture::for_window_scaled_on(target,processing_percent,health.and_then(|h|h.renderer_adapter.as_deref()))?;
    let (width,height)=wgc.dimensions();
    let (pw,ph)=wgc.processed_size();
    let is_glass=mode==MirrorMode::CompositedInPlace;
    // Only a separate own preview may target another display. Glass stays on the source HWND.
    let requested=health.and_then(|h|h.preview_display.as_deref());
    let target_display=if mode==MirrorMode::SeparateWindow&&health.is_some()&&requested.is_some(){
        crate::display_probe::select(&crate::display_probe::read(),requested)
    }else{None};
    let source_monitor=crate::win::window_monitor_bounds(target.0);
    // The default game overlay must be placed on the SAME physical monitor as
    // the selected game, including negative X/Y in multi-monitor Windows.
    let monitor_size=target_display.as_ref().map(|d|(d.width,d.height))
        .or_else(||source_monitor.map(|(_,_,w,h)|(w,h)))
        .unwrap_or_else(crate::win::screen_size);
    if !is_glass&&requested.is_some()&&target_display.as_ref().map(|d|d.name.as_str())!=requested{
        status("Selected display unavailable; falling back to current primary for own preview".into());
    }
    let (outw,outh)=if mode==MirrorMode::BorderlessScreen{
        // Avoid 4x GPU pixel overhead on RX 580 when Eco/Balance is chosen.
        let cap=match quality{0=>2.0,1|3=>1.50,_=>1.25};
        fit_game_to_monitor((width,height),monitor_size,cap)
    }else{
        crate::spatial::target_size((width,height),monitor_size,upscale_ratio,is_glass)
    };
    status(format!("Native GPU WGC {width}x{height} -> effects {pw}x{ph} -> own output {outw}x{outh}, {} HLSL passes; spatial mode {upscale_mode}; game never resized",
        stages.len()));
    let mirror=match mode{
        MirrorMode::SeparateWindow=>{
            let view_scale=(1200.0/outw as f32).min(790.0/outh as f32).min(1.0);
            let (x,y,w,h)=target_display.as_ref().map(|d|d.window_rect(outw,outh)).unwrap_or(
                (180,110,(outw as f32*view_scale).round() as i32,(outh as f32*view_scale).round() as i32));
            OwnWindow::new("HORIZON BOOST - Native GPU Live Preview (separate window)",x,y,w,h,false)?
        },
        MirrorMode::CompositedInPlace=>OwnWindow::composited(target)?,
        MirrorMode::DesktopDuplication=>return Err("DXGI fallback must not run through WGC path".into()),
        MirrorMode::BorderlessScreen=>{
            let bounds=source_monitor.ok_or("Cannot locate selected game's monitor")?;
            let (x,y,w,h)=center_game_output(bounds,(outw,outh));
            status(format!("Window upscaler: {width}x{height} -> {outw}x{outh} on game monitor. Aspect preserved; GPU quality {quality}. Game resolution unchanged."));
            OwnWindow::borderless(x,y,w,h)?
        },
    };
    let glass=mode==MirrorMode::CompositedInPlace;
    if glass{
        let style=unsafe{GetWindowLongW(mirror.hwnd,-20)} as u32;
        const EX_REQUIRED:u32=0x80000|0x20|0x8|0x08000000;
        if style&EX_REQUIRED!=EX_REQUIRED{
            return Err("Composited output missing nonactivating click-through styles".into());
        }
        status("Composited WGC glass initialized. Source game HWND untouched; FSE visibility unverified.".into());
    }
    let factory:IDXGIFactory2=unsafe{CreateDXGIFactory1()}
        .map_err(|e|format!("Native mirror DXGI factory: {e}"))?;
    let mut hdr_processor=None;
    let hdr_requested=factor==1&&!glass&&max_frames.is_none()&&health.is_some_and(|h|h.hdr_requested);
    if hdr_requested{
        let name=target_display.as_ref().map(|d|d.name.as_str());
        match crate::hdr_output::available_monitor(name){
            Ok(true)=>match crate::hdr_output::Processor::new(&wgc.device,outw,outh){
                Ok(pass)=>{status("Opt-in scRGB FP16 GPU surface ready; checking DXGI color-space support".into());hdr_processor=Some(pass);},
                Err(e)=>status(format!("HDR output GPU resources rejected; using SDR fallback: {e}"))
            },
            Ok(false)=>status("HDR output disabled: selected monitor is not actively in supported HDR10 mode; using SDR fallback".into()),
            Err(e)=>status(format!("HDR monitor probe unavailable; using SDR fallback: {e}"))
        }
    }
    let mut desc=crate::native_present::swapchain_descriptor(outw,outh);
    if hdr_processor.is_some(){desc.Format=windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R16G16B16A16_FLOAT;}
    let want_composition=mode==MirrorMode::SeparateWindow&&factor==1&&hdr_processor.is_none()&&health.is_some_and(|h|h.composition_requested);
    let mut composition=if want_composition{
        match crate::composition_output::Output::new(&wgc.device,&factory,HWND(mirror.hwnd),outw,outh){
            Ok(v)=>{status("DirectComposition visual output active (separate window)".into());Some(v)},
            Err(e)=>{status(format!("DirectComposition unavailable; standard flip output fallback: {e}"));None}
        }
    }else{None};
    if let Some(h)=health{h.composition_active.store(composition.is_some(),Ordering::Release);h.nominal_refresh.store(target_display.as_ref().and_then(|d|d.refresh).unwrap_or(0) as u64,Ordering::Relaxed);}
    let request_wait=composition.is_none()&&!glass&&factor==1&&health.is_some_and(|h|h.waitable_requested.load(Ordering::Relaxed));
    if request_wait{desc.Flags=DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32;}
    let tearing_cap=crate::adapter_select::tearing(&factory);
    let request_tearing=composition.is_none()&&!glass&&factor==1&&max_frames.is_none()&&health.is_some_and(|h|h.tearing_requested)&&tearing_cap==Some(true);
    let tearing_bit=windows::Win32::Graphics::Dxgi::DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING.0 as u32;
    if request_tearing{desc.Flags|=tearing_bit;}
    if let Some(h)=health{h.tearing_supported.store(match tearing_cap{Some(true)=>2,Some(false)=>1,None=>0},Ordering::Relaxed);}
    if !glass{let _=unsafe{factory.MakeWindowAssociation(HWND(mirror.hwnd),windows::Win32::Graphics::Dxgi::DXGI_MWA_NO_ALT_ENTER)};}
    let mut initial=if let Some(c)=composition.as_ref(){Ok(c.chain.clone())}else{unsafe{factory.CreateSwapChainForHwnd(&wgc.device,HWND(mirror.hwnd),&desc,None,None::<&IDXGIOutput>)}};
    if initial.is_err()&&hdr_processor.is_some(){
        status("FP16 own swapchain creation rejected; reverting to normal SDR format".into());
        hdr_processor=None;desc.Format=windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R8G8B8A8_UNORM;
        initial=unsafe{factory.CreateSwapChainForHwnd(&wgc.device,HWND(mirror.hwnd),&desc,None,None::<&IDXGIOutput>)};
    }
    let mut chain=match initial{
        Ok(v)=>v,
        Err(e) if request_wait||request_tearing=>{
            status(format!("Optional waitable/tearing swapchain flags unavailable; using ordinary own output: {e}"));
            desc.Flags=0;
            unsafe{factory.CreateSwapChainForHwnd(&wgc.device,HWND(mirror.hwnd),&desc,None,None::<&IDXGIOutput>)}
                .map_err(|e|format!("Native mirror independent flip swapchain: {e}"))?
        },
        Err(e)=>return Err(format!("Native mirror independent flip swapchain: {e}")),
    };
    if hdr_processor.is_some(){
        if let Err(e)=crate::hdr_output::set_color_space(&chain){
            status(format!("Actual scRGB DXGI color space failed; clean SDR fallback: {e}"));
            drop(chain);
            hdr_processor=None;
            desc.Format=windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R8G8B8A8_UNORM;
            chain=unsafe{factory.CreateSwapChainForHwnd(&wgc.device,HWND(mirror.hwnd),&desc,None,None::<&IDXGIOutput>)}
                .map_err(|e|format!("Cannot restore own SDR swapchain: {e}"))?;
        }else{status("Real DXGI FP16 scRGB own-window output active (SDR source converted linearly, no original HDR recovered)".into());}
    }
    if let Some(h)=health{h.hdr_active.store(hdr_processor.is_some(),Ordering::Release);}
    let tearing_created=desc.Flags&tearing_bit!=0;
    let mut waitable=if desc.Flags&(DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32)!=0{
        match crate::dxgi_wait::FramePermit::new(&chain){
            Ok(p)=>Some(p),Err(e)=>{status(format!("DXGI waitable object unavailable: {e}"));None}
        }
    }else{None};
    if let Some(h)=health{h.waitable_active.store(waitable.is_some(),Ordering::Relaxed);}
    let mut permit=false;
    let mut wait_failures=0u32;
    let back:ID3D11Texture2D=unsafe{chain.GetBuffer(0)}
        .map_err(|e|format!("Mirror swapchain backbuffer: {e}"))?;
    let back_resource:ID3D11Resource=back.cast()
        .map_err(|e|format!("Mirror resource: {e}"))?;
    let mut glass_state=GlassSyncState::default();
    let mut alignment=GameWindowAlignment::default();
    let mut output_shown=false;
    if max_frames.is_some(){
        if mode==MirrorMode::BorderlessScreen{
            align_gpu_output(target,&mirror,(outw,outh),&mut alignment)?;
        }
        if !glass{mirror.show();}else{sync_glass(target,&mirror,true,true,&mut glass_state)?;}
        output_shown=true;
    }
    // WinRT free-threaded frame pool may have no available frame
    // immediately after StartCapture; allow the first DWM tick.
    std::thread::sleep(Duration::from_millis(240));
    let mut submitted=0u64;
    let mut source_ready=false;
    // Same-resolution filtering cannot invent game detail; applying another
    // resampling stage in a fullscreen-sized WGC mirror is wasted GPU work
    // and can make the source less sharp. Skip it even if user saved a mode.
    let mut spatial=if (outw,outh)!=(pw,ph){
        Some(crate::spatial::NativeSpatialScaler::new(&wgc.device,outw,outh)?)
    }else{
        if upscale_mode>0{
            status("Native scaler skipped: source and output pixel sizes match. Kept image pixel-exact to avoid extra blur/GPU overhead.".into());
        }
        None
    };
    if matches!(upscale_mode,8|9)&&(outw>pw.saturating_mul(2)||outh>ph.saturating_mul(2)){
        status("Vendor scaling ratio exceeds 2x after reduced processing; using bilinear reconstruction.".into());
    }
    let fg=if factor>1{
        status(format!("Experimental capture-only optical-flow interpolation x{factor}, {}px automatic field tiles; game engine FPS unchanged",crate::framegen::auto_tile(outw,outh)));
        let pyramid=lighting.read().map(|v|v.motion.pyramid_flow).unwrap_or(false);
        Some(crate::framegen::FlowInterpolator::new_with_pyramid(&wgc.device,outw,outh,crate::framegen::auto_tile(outw,outh),pyramid)?)
    }else{None};
    let mut previous_report=Instant::now();
    let mut previous_frames=0u64;
    let mut last_frame=Instant::now();
    let mut pacing_guard=crate::framegen::CapturePacingGuard::default();
    let mut deferred_synthetic=0u64;let mut fg_failed=false;
    let mut adaptive_passes=stages.len().min(8);
    let mut previous_look=stages.to_vec();
    let mut previous_scale=upscale_mode;
    let mut scaler_failure:Option<u8>=None;
    if let Some(h)=health{h.scale_input.store((u64::from(pw)<<32)|u64::from(ph),Ordering::Relaxed);h.scale_output.store((u64::from(outw)<<32)|u64::from(outh),Ordering::Relaxed);}
    let mut previous_quality=quality;
    let mut previous_render_tuning:Option<crate::lighting::LightTuning>=None;
    // Keep a prepared tuning snapshot across frames. Prior versions cloned
    // several profile maps every poll, even when source input had not changed.
    let mut raw_tuning=lighting.read().map(|g|g.clone()).unwrap_or_default();
    let mut light_state=raw_tuning.clone();
    if let Some(h)=health{
        crate::display_color::apply(&mut light_state,&h.display_grades,target_display.as_ref().map(|d|d.name.as_str()));
    }
    if factor>1{light_state.layers.split=None;}
    if factor>1||light_state.motion.temporal_mode!=0{light_state.reconstruction.animated_grain=false;}
    let mut capture_config=crate::capture_cadence::Control::default();
    let mut previous_stamp=None;
    let mut adaptive_budget=crate::preview_pacer::GpuBudget::default();
    let mut demo_tick=0u32;
    let mut source_paused=false;
    let mut deadline=crate::preview_pacer::PresentDeadline::default();
    let mut observed_drops=0u64;let mut observed_duplicates=0u64;let mut observed_stale=0u64;
    let mut fg_pressure=crate::render_pressure::FgGate::default();
    let mut reconstruction_sharpener:Option<crate::reconstruction::Sharpen>=None;
    let mut temporal=crate::temporal_runtime::Runtime::default();
    let mut scale_timer=crate::live_gpu_timing::Sampler::default();
    let mut flow_timer=crate::live_gpu_timing::Sampler::default();
    let mut warp_timer=crate::live_gpu_timing::Sampler::default();
    let mut topology_check=Instant::now();
    let mut topology_watch=crate::display_probe::Watch::default();
    let mut source_monitor_watch=StableMonitorWatch::default();
    let mut expected_owner=0;unsafe{GetWindowThreadProcessId(target.0,&mut expected_owner);}
    if expected_owner==0{return Err("Selected capture window closed".into());}
    while stop.load(Ordering::Acquire)&&unsafe{IsWindow(mirror.hwnd)}!=0 {
        if let Some(h)=health{h.beat();}
        let mut owner=0;unsafe{GetWindowThreadProcessId(target.0,&mut owner);}
        if owner!=expected_owner{return Err("Selected capture window identity changed".into());}
        pump_messages();
        if let Some(c)=composition.as_mut(){
            let mut rect=Rect{left:0,top:0,right:0,bottom:0};
            if unsafe{GetClientRect(mirror.hwnd,&mut rect)}!=0{
                c.fit(((rect.right-rect.left).max(0) as u32,(rect.bottom-rect.top).max(0) as u32))?;
            }
        }
        if !glass&&health.is_some()&&topology_check.elapsed()>=Duration::from_secs(2){
            topology_check=Instant::now();
            // MonitorForWindow follows the selected game. The default output
            // has NO explicit display choice: comparing None against the
            // primary monitor used to force a rebuild every four seconds.
            let observed_source=crate::win::window_monitor_bounds(target.0);
            if source_monitor_watch.changed(source_monitor,observed_source){
                return Err("WGC_REBUILD_REQUIRED: selected game monitor moved, resized or disconnected".into());
            }
            // Watch a named *separate* preview only when one was requested.
            if mode==MirrorMode::SeparateWindow&&requested.is_some(){
                let observed=crate::display_probe::select(&crate::display_probe::read(),requested);
                if topology_watch.changed(target_display.as_ref(),observed){
                    return Err("WGC_REBUILD_REQUIRED: selected preview display mode or topology changed".into());
                }
            }
        }
        if unsafe{IsWindow(target.0)}==0{return Err("Target game window closed".into());}
        let own_minimized=!glass&&output_shown&&unsafe{IsIconic(mirror.hwnd)}!=0;
        if let Some(h)=health{h.own_minimized.store(own_minimized,Ordering::Relaxed);}
        if own_minimized{
            source_ready=false;previous_stamp=None;temporal.pause();pacing_guard=Default::default();fg_pressure.reset();deadline.reset();
            if let Some(h)=health{h.paused.store(true,Ordering::Relaxed);h.clear_timings();}
            last_frame=Instant::now();std::thread::sleep(Duration::from_millis(100));continue;
        }
        if unsafe{IsIconic(target.0)}!=0||unsafe{IsWindowVisible(target.0)}==0{
            if !source_paused{
                source_paused=true;source_ready=false;output_shown=false;previous_stamp=None;temporal.pause();pacing_guard=Default::default();fg_pressure.reset();glass_state.visible=false;deadline.reset();adaptive_budget.reset();
                unsafe{ShowWindow(mirror.hwnd,0);}
                if let Some(h)=health{h.paused.store(true,Ordering::Relaxed);h.clear_timings();}
                status("Source minimized or hidden; own GPU dispatch paused".into());
            }
            last_frame=Instant::now();
            std::thread::sleep(Duration::from_millis(100));
            continue;
        }else if source_paused{
            source_paused=false;
            if let Some(h)=health{h.paused.store(false,Ordering::Relaxed);}
            status("Source visible again; own GPU output resumed".into());
        }
        if mode==MirrorMode::BorderlessScreen&&!selected_process_foreground(expected_owner,mirror.hwnd){
            if output_shown{unsafe{ShowWindow(mirror.hwnd,0);}}
            output_shown=false;source_ready=false;previous_stamp=None;temporal.pause();
            pacing_guard=Default::default();fg_pressure.reset();deadline.reset();adaptive_budget.reset();
            if let Some(h)=health{h.paused.store(true,Ordering::Relaxed);h.clear_timings();}
            last_frame=Instant::now();
            std::thread::sleep(Duration::from_millis(80));
            continue;
        }
        if mode==MirrorMode::BorderlessScreen{
            align_gpu_output(target,&mirror,(outw,outh),&mut alignment)?;
        }
        if glass {
            let visible=sync_glass(target,&mirror,max_frames.is_some(),output_shown,&mut glass_state)?;
            if !visible{
                // Capture remains active while user Alt+Tabs; rendering pauses.
                source_ready=false;output_shown=false;previous_stamp=None;temporal.pause();pacing_guard=Default::default();fg_pressure.reset();deadline.reset();adaptive_budget.reset();
                if let Some(h)=health{h.paused.store(true,Ordering::Relaxed);h.clear_timings();}
                last_frame=Instant::now();
                std::thread::sleep(Duration::from_millis(90));
                continue;
            }
        }
        if let Some(h)=health{h.paused.store(false,Ordering::Relaxed);}
        if max_frames.is_some(){
            // Only used on our synthetic own HWND in explicit self-test.
            demo_tick=demo_tick.wrapping_add(1);
            paint_test_source(target.0,demo_tick);
        }
        if unsafe{IsWindow(target.0)}==0 {return Err("Target game window closed".into());}
        // Snapshot stages and tuning consistently; release locks before GPU work.
        let look_guard=health.and_then(|h|h.live_stages.read().ok());
        let selected=look_guard.as_ref().and_then(|g|g.as_ref().cloned());
        // No allocation on unchanged lighting settings; clone_from may reuse
        // existing vector/map allocations only when user changes a profile.
        let tuning_changed=if let Ok(guard)=lighting.read(){
            if *guard!=raw_tuning{raw_tuning.clone_from(&guard);true}else{false}
        }else{false};
        if tuning_changed{
            light_state.clone_from(&raw_tuning);
            if let Some(h)=health{
                crate::display_color::apply(&mut light_state,&h.display_grades,target_display.as_ref().map(|d|d.name.as_str()));
            }
            if factor>1{light_state.layers.split=None;}
            if factor>1||light_state.motion.temporal_mode!=0{light_state.reconstruction.animated_grain=false;}
        }
        let quality=health.and_then(|h|h.live_quality.load(Ordering::Acquire).checked_sub(1)).map_or(quality,|q|q as u8);
        drop(look_guard);
        let stages=selected.as_deref().unwrap_or(stages);
        let upscale_mode=health.and_then(|h|h.live_scale_mode.load(Ordering::Acquire).checked_sub(1)).map_or(upscale_mode,|mode|mode as u8);
        if previous_look.as_slice()!=stages||previous_scale!=upscale_mode||previous_quality!=quality{
            previous_quality=quality;
            if previous_scale!=upscale_mode{scaler_failure=None;}
            previous_scale=upscale_mode;
            previous_look.clear();
            previous_look.extend_from_slice(stages);
            adaptive_passes=stages.len().min(8);adaptive_budget.reset();
            // Never interpolate between different color transforms.
            source_ready=false;previous_stamp=None;temporal.pause();pacing_guard=Default::default();fg_pressure.reset();
            scale_timer.clear();flow_timer.clear();warp_timer.clear();
        }
        if (factor>1||light_state.motion.temporal_mode!=0)&&previous_render_tuning.as_ref()!=Some(&light_state){
            previous_render_tuning=Some(light_state.clone());source_ready=false;previous_stamp=None;temporal.pause();pacing_guard=Default::default();fg_pressure.reset();flow_timer.clear();warp_timer.clear();
        }
        if factor==1&&max_frames.is_none(){
            let requested=live_cap.map_or(0,|c|c.load(Ordering::Relaxed));
            wgc.set_pressure_cap(Some(requested));
            if let Some(ticks)=capture_config.change(requested){
                let supported=wgc.set_min_interval(ticks);
                if let Some(h)=health{h.capture_interval_active.store(supported,Ordering::Relaxed);}
                if !supported{capture_config.disable();}
            }
        }
        let user_timing=raw_tuning.reconstruction.live_gpu_timing;
        light_state.reconstruction.live_gpu_timing=user_timing||quality==3;
        let active_stages=if quality==3{
            &stages[..stages.len().min(adaptive_passes)]
        }else{stages};
        // Publish the effective count only after dispatch, including a last-good fallback.
        if factor==1&&max_frames.is_none(){
            if let Some(cap)=live_cap{
                let remaining=deadline.remaining(Instant::now(),cap.load(Ordering::Relaxed));
                if crate::preview_pacer::wait_step(remaining){continue;}
            }
        }
        if !permit{
            if let Some(w)=waitable.as_ref(){
                match w.wait(){
                    Ok(true)=>{permit=true;wait_failures=0;},
                    Ok(false)=>{
                        wait_failures+=1;
                        if let Some(h)=health{h.wait_timeouts.fetch_add(1,Ordering::Relaxed);}
                        if wait_failures<40{continue;}
                        status("DXGI frame wait timed out repeatedly; ordinary pacing restored".into());
                        waitable=None;
                        if let Some(h)=health{h.waitable_active.store(false,Ordering::Relaxed);}
                    },
                    Err(e)=>{
                        status(e);waitable=None;
                        if let Some(h)=health{h.waitable_active.store(false,Ordering::Relaxed);}
                    }
                }
            }
        }
        if factor>1&&max_frames.is_none(){wgc.set_pressure_cap(Some(60));}
        let capture_start=Instant::now();
        let next=match wgc.next_filtered_frame(active_stages,&light_state){
            Ok(v)=>v,
            Err(error)=>{
                if unsafe{wgc.device.GetDeviceRemovedReason()}.is_err(){
                    return Err(format!("WGC_DEVICE_LOST: {error}"));
                }
                return Err(error);
            }
        };
        let capture_elapsed=capture_start.elapsed();
        if let Some(h)=health{
            let total=wgc.discarded_frames();
            h.dequeued.fetch_add(total.saturating_sub(observed_drops),Ordering::Relaxed);
            observed_drops=total;
            let duplicates=wgc.duplicate_frames();h.duplicate_frames.fetch_add(duplicates.saturating_sub(observed_duplicates),Ordering::Relaxed);observed_duplicates=duplicates;
            let discarded=wgc.stale_pre_filter();h.stale_pre_filter.fetch_add(discarded.saturating_sub(observed_stale),Ordering::Relaxed);observed_stale=discarded;
            let (sample,failed)=wgc.sample_status();
            h.sample_code.store(if !light_state.reconstruction.capture_diagnostic{0}else if failed{3}else{sample.map_or(0,|v|if v.near_black{2}else{1})},Ordering::Relaxed);
            h.sample_repeats.store(sample.map_or(0,|v|v.repeats as u64),Ordering::Relaxed);h.optional_warning.store(wgc.optional_warning() as u64,Ordering::Relaxed);
        }
        match next{
            Some(processed)=>{
                if let Some(h)=health{h.fused_saved.store(wgc.fused_saved() as u64,Ordering::Relaxed);h.fence_active.store(wgc.used_copy_fence(),Ordering::Relaxed);h.live_look_rejected.store(wgc.live_look_rejected(),Ordering::Relaxed);h.effective_passes.store(wgc.live_active_count() as u64,Ordering::Relaxed);}
                let timing=light_state.reconstruction.live_gpu_timing||factor>1;
                let n=Instant::now();
                if timing{scale_timer.poll(wgc.context(),n);flow_timer.poll(wgc.context(),n);warp_timer.poll(wgc.context(),n);}
                else{scale_timer.clear();flow_timer.clear();warp_timer.clear();}
                let scale_needed=crate::spatial::needs_scaling((pw,ph),(outw,outh),upscale_mode);
                if timing&&(scale_needed||((pw,ph)!=(width,height)&&light_state.reconstruction.sharpness>0.)){scale_timer.begin(&wgc.device,wgc.context(),n);}
                if spatial.is_none()&&crate::spatial::needs_scaling((pw,ph),(outw,outh),upscale_mode){spatial=Some(crate::spatial::NativeSpatialScaler::new(&wgc.device,outw,outh)?);}
                let mut actual_mode=crate::spatial::effective_mode((pw,ph),(outw,outh),upscale_mode);
                let processed=if let Some(scaler)=spatial.as_mut().filter(|_|scale_needed){
                    if scaler_failure==Some(upscale_mode){actual_mode=1;}
                    match scaler.render(&wgc.device,wgc.context(),&processed,actual_mode){
                        Ok(v)=>v,
                        Err(e)=>{
                            if actual_mode==1||max_frames.is_some()||unsafe{wgc.device.GetDeviceRemovedReason()}.is_err(){return Err(e);}
                            scaler_failure=Some(upscale_mode);actual_mode=1;
                            if let Some(h)=health{h.scale_fallbacks.fetch_add(1,Ordering::Relaxed);}
                            source_ready=false;previous_stamp=None;temporal.pause();pacing_guard=Default::default();fg_pressure.reset();
                            status(format!("Scaler failed; using bilinear until algorithm changes: {e}"));
                            scaler.render(&wgc.device,wgc.context(),&processed,1)?
                        }
                    }
                }else{processed};
                if let Some(h)=health{h.effective_scale_mode.store(u64::from(actual_mode)+1,Ordering::Relaxed);h.scale_runtime_fallback.store(scaler_failure==Some(upscale_mode),Ordering::Relaxed);}
                let processed=if (pw,ph)!=(width,height)&&light_state.reconstruction.sharpness>0.{
                    if reconstruction_sharpener.is_none(){reconstruction_sharpener=Some(crate::reconstruction::Sharpen::new(&wgc.device,outw,outh)?);}
                    reconstruction_sharpener.as_mut().ok_or("Reconstruction sharpener missing")?.render(&wgc.device,wgc.context(),&processed,light_state.reconstruction.sharpness)?
                }else{reconstruction_sharpener=None;processed};
                scale_timer.end(wgc.context(),Instant::now());
                let temporal_active=!glass&&factor==1&&light_state.motion.temporal_mode!=0;
                if timing&&temporal_active{flow_timer.begin(&wgc.device,wgc.context(),Instant::now());}
                let processed=if temporal_active{
                    match temporal.render(&wgc.device,wgc.context(),&processed,outw,outh,&light_state,active_stages.len(),Instant::now()){
                        Ok(v)=>v,Err(e)=>{status(format!("Temporal effect disabled after error; current-frame fallback: {e}"));processed}
                    }
                }else{temporal.clear();processed};
                if temporal_active{flow_timer.end(wgc.context(),Instant::now());}
                if glass&&!sync_glass(target,&mirror,max_frames.is_some(),output_shown,&mut glass_state)?{source_ready=false;previous_stamp=None;output_shown=false;continue;}
                if mode==MirrorMode::BorderlessScreen&&!selected_process_foreground(expected_owner,mirror.hwnd){
                    unsafe{ShowWindow(mirror.hwnd,0);}source_ready=false;previous_stamp=None;output_shown=false;temporal.pause();continue;
                }
                let submit_start=Instant::now();
                if let Some(ref generator)=fg{
                    // Conservative by default: if the live game capture is
                    // already slower than a frame budget, do not sacrifice
                    // real frames just to increase the *counter* of synthetic.
                    let interval=last_frame.elapsed().as_secs_f32()*1000.0;
                    let source_interval=crate::capture_clock::interval_ms(previous_stamp,wgc.frame_stamp());
                    let nominal=target_display.as_ref().and_then(|d|d.refresh);
                    let age=wgc.current_capture_age();
                    let stale=age.is_none()||crate::framegen::stale_for_fg(age,light_state.motion.max_fg_age_ms);
                    let queued=wgc.backlog()>=2;
                    let mut effective=if max_frames.is_some(){factor}else{pacing_guard.observe_for_display(factor,source_interval,interval,nominal,light_state.motion.adaptive_fg)};
                    if !source_ready||fg_failed||(max_frames.is_none()&&(stale||queued)){effective=1;}
                    if max_frames.is_none(){
                        let reason=if fg_failed{6}else if !source_ready{1}else if stale{2}else if queued{3}else if effective==1{4}else{0};
                        let budget=source_interval.map(|ms|ms as f64);
                        let now=Instant::now();
                        let allowed=fg_pressure.allow_samples(now,budget,flow_timer.reading(now),warp_timer.reading(now),effective);
                        let reason=if reason==0&&!allowed{5}else{reason};
                        if reason!=0{effective=1;}
                        if let Some(h)=health{h.fg_reason.store(reason,Ordering::Relaxed);if reason==5{h.fg_pressure_skips.fetch_add(1,Ordering::Relaxed);}}
                    }
                    if queued&&source_ready{if let Some(h)=health{h.fg_backlog.fetch_add(1,Ordering::Relaxed);}}
                    if let Some(h)=health{
                        h.fg_effective.store(effective as u64,Ordering::Relaxed);
                        if source_ready&&stale{h.fg_stale.fetch_add(1,Ordering::Relaxed);}
                        if effective>1&&effective<factor{h.fg_reduced.fetch_add(1,Ordering::Relaxed);}
                    }
                    if source_ready&&effective==1{deferred_synthetic+=1;if let Some(h)=health{h.deferred_fg.fetch_add(1,Ordering::Relaxed);}}
                    if source_ready&&effective>1 {
                        let attempt=(||->Result<(),String>{
                            if timing{flow_timer.begin(&wgc.device,wgc.context(),Instant::now());}
                            let estimate=generator.estimate(&wgc.device,wgc.context(),&processed);
                            flow_timer.end(wgc.context(),Instant::now());
                            let pair=estimate?;
                            for phase in 1..effective{
                                if !stop.load(Ordering::Acquire){break;}
                                if max_frames.is_none()&&(wgc.backlog()>=2||wgc.current_capture_age().is_none()||crate::framegen::stale_for_fg(wgc.current_capture_age(),light_state.motion.max_fg_age_ms)){
                                    if let Some(h)=health{h.fg_stale.fetch_add(1,Ordering::Relaxed);}break;
                                }
                                if timing{warp_timer.begin(&wgc.device,wgc.context(),Instant::now());}
                                let synthesis=generator.synth_tuned(wgc.context(),&pair,effective,phase,&light_state.motion,0);
                                warp_timer.end(wgc.context(),Instant::now());
                                let synthesized=synthesis?;
                                let virtual_image:ID3D11Resource=synthesized.cast().map_err(|e|format!("Synthesized GPU resource: {e}"))?;
                                unsafe{wgc.context().CopyResource(&back_resource,&virtual_image);}
                                let present=if max_frames.is_some(){unsafe{chain.Present(1,DXGI_PRESENT(0))}}else{unsafe{chain.Present(1,DXGI_PRESENT(8))}};
                                if unsafe{wgc.device.GetDeviceRemovedReason()}.is_err(){return Err("WGC_DEVICE_LOST: FG Present".into());}
                                if crate::present_status::classify(present.0)==crate::present_status::Outcome::Submitted{generated.fetch_add(1,Ordering::Release);}
                                else if max_frames.is_some(){return Err(format!("Deterministic GPU synthesis Present failed: {:?}",present));}
                                else{fg_pressure.penalize(Instant::now());if let Some(h)=health{h.failed_fg_present.fetch_add(1,Ordering::Relaxed);}break;}
                            }
                            Ok(())
                        })();
                        if let Err(e)=attempt{
                            if max_frames.is_some()||unsafe{wgc.device.GetDeviceRemovedReason()}.is_err(){return Err(e);}
                            fg_failed=true;
                            if let Some(h)=health{h.fg_gpu_errors.fetch_add(1,Ordering::Relaxed);h.fg_effective.store(1,Ordering::Relaxed);}
                            status(format!("FG compute failed; retaining real frames until preview restart: {e}"));
                        }
                    }
                }
                if !stop.load(Ordering::Acquire){break;}
                let image:ID3D11Resource=if let Some(hdr)=hdr_processor.as_mut(){
                    hdr.render_configured(&wgc.device,wgc.context(),&processed,health.map_or(100.,|h|h.hdr_white_nits))?;
                    hdr.output.cast().map_err(|e|format!("Own scRGB GPU resource: {e}"))?
                }else{processed.cast().map_err(|e|format!("Mirror filtered image: {e}"))?};
                unsafe{
                    // Present submits the pending source copy. Avoid an
                    // extra driver flush every real frame on RX 580.
                    wgc.context().CopyResource(&back_resource,&image);
                }
                // VSync off is allowed only in our separate real-frame
                // preview, not in glass/FG or deterministic QA. No tearing flag.
                let interval=if !glass&&factor==1&&max_frames.is_none()&&health.is_some_and(|h|!h.vsync.load(Ordering::Relaxed)){0}else{1};
                let flags=crate::adapter_select::tearing_flag(interval,tearing_created);
                let result=unsafe{chain.Present(interval,flags)};
                let outcome=crate::present_status::classify(result.0);
                if let Some(h)=health{h.tearing_active.store(outcome==crate::present_status::Outcome::Submitted&&flags.0!=0,Ordering::Relaxed);}
                if let Err(e)=result.ok(){
                    if unsafe{wgc.device.GetDeviceRemovedReason()}.is_err(){
                        return Err(format!("WGC_DEVICE_LOST: {e}"));
                    }
                    return Err(format!("Mirror visible native Present: {e}"));
                }
                permit=false;
                // Never reveal an uninitialized output. A hidden window can
                // return OCCLUDED; showing after its first attempted present
                // is safe because the backbuffer already contains a real frame.
                if !output_shown{
                    if glass{if !sync_glass(target,&mirror,max_frames.is_some(),true,&mut glass_state)?{continue;}}
                    else{
                        if mode==MirrorMode::BorderlessScreen{
                            align_gpu_output(target,&mirror,(outw,outh),&mut alignment)?;
                        }
                        mirror.show();
                    }
                    output_shown=true;
                }
                if outcome!=crate::present_status::Outcome::Submitted{
                    if max_frames.is_some(){return Err(format!("Own test Present did not submit: {outcome:?}"));}
                    if let Some(h)=health{h.non_submitted.fetch_add(1,Ordering::Relaxed);h.clear_timings();}
                    source_ready=false;previous_stamp=None;temporal.pause();pacing_guard=Default::default();fg_pressure.penalize(Instant::now());deadline.reset();adaptive_budget.reset();
                    last_frame=Instant::now();std::thread::sleep(Duration::from_millis(40));continue;
                }
                deadline.submitted(capture_start,Instant::now(),live_cap.map_or(0,|c|c.load(Ordering::Relaxed)));
                if let Some(h)=health{
                    h.frame(capture_elapsed,submit_start.elapsed());
                    let p=wgc.phase_times();let n=Instant::now();
                    h.present_age(wgc.current_capture_age());
                    h.phases(wgc.capture_age(),if user_timing{[p[0],p[1],scale_timer.value(n),flow_timer.value(n),warp_timer.value(n)]}else{[None;5]});
                }
                previous_stamp=wgc.frame_stamp();
                if let Some(ref generator)=fg{
                    generator.save_real_frame(wgc.context(),&processed);
                    source_ready=true;
                }
                submitted+=1;
                if quality==3&&!wgc.live_look_rejected(){
                    let cap=live_cap.map_or(60,|c|c.load(Ordering::Relaxed));
                    let now=Instant::now();
                    let next=adaptive_budget.observe_reading(now,adaptive_passes,stages.len(),cap,wgc.filter_reading(now));
                    if next!=adaptive_passes{adaptive_passes=next;source_ready=false;previous_stamp=None;temporal.pause();pacing_guard=Default::default();status(format!("Adaptive GPU: {next} native passes based on sampled GPU filter cost, not game FPS"));}
                }
                if glass&&max_frames.is_some()&&submitted==5{
                    let rect=source_rect(target)?;
                    let cursor_point=Point{
                        x:rect.left+(rect.right-rect.left)/2,
                        y:rect.top+(rect.bottom-rect.top)/2,
                    };
                    let hit=unsafe{WindowFromPoint(cursor_point)};
                    if hit==mirror.hwnd{
                        return Err("Glass overlay intercepted test hit-testing; click-through NOT verified".into());
                    }
                    if hit!=target.0{
                        return Err("Windows hit-test did not resolve to synthetic source; click-through inconclusive".into());
                    }
                }
                // Keep a cumulative count even when capture resources are
                // rebuilt after a game resolution/window-size change.
                frames_seen.fetch_add(1,Ordering::Release);
                last_frame=Instant::now();
                if previous_report.elapsed()>Duration::from_secs(4){
                    let fps=(submitted-previous_frames) as f32/
                        previous_report.elapsed().as_secs_f32().max(0.001);
                    status(format!("Own output: {submitted} captured real frames · {} interpolated frames · {:.1} processed source FPS; game FPS measured separately",
                        generated.load(Ordering::Relaxed),fps));
                    if factor>1{
                        status(format!("FG safety: {} captured frames skipped interpolation due to insufficient sustained GPU/capture cadence. Real frames retained; no fake game FPS.",deferred_synthetic));
                    }
                    previous_report=Instant::now();
                    previous_frames=submitted;
                }
                if max_frames.is_some_and(|max|submitted>=max){break;}
            },
            None=>{if let Some(h)=health{h.no_frame_polls.fetch_add(1,Ordering::Relaxed);if last_frame.elapsed()>Duration::from_secs(4){h.clear_timings();}}wgc.wait_for_frame(crate::render_pressure::idle_wait(last_frame.elapsed()));},
        }
        // Never run an unbounded stalled loop if the capture was denied.
        if last_frame.elapsed()>Duration::from_secs(10){
            return Err("WGC supplied no new frames for 10s; target may be protected/minimized".into());
        }
        // Both real-frame live routes now use the same pre-capture deadline.
        // Do not stack a fixed sleep after a VSync-blocking Present.
        // Synthetic test routes retain their previous conservative pacing.
        if live_cap.is_none()&&factor==1{
            std::thread::sleep(Duration::from_millis(23));
        }
    }
    status(format!("Native GPU mirror stopped: {submitted} real + {} interpolated frames on own output",
        generated.load(Ordering::Relaxed)));
    Ok(submitted)
}
fn look_stages(look:usize,quality:u8)->Result<Vec<usize>,String>{
    let preset=crate::native_presets::PRESETS.get(look).ok_or("Invalid native GPU mirror preset")?;
    quality_stages(preset.stages,quality)
}
fn quality_stages(stages:&[usize],quality:u8)->Result<Vec<usize>,String>{
    if quality>3{return Err("Invalid GPU quality".into());}
    let limit=match quality{1=>2,2=>1,_=>8};
    Ok(stages[..stages.len().min(limit)].to_vec())
}
#[cfg(test)]
#[test]
fn live_look_keeps_order_and_quality_cap(){
    let stages=crate::native_presets::PRESETS[5].stages;
    assert_eq!(look_stages(5,0).unwrap(),stages);
    assert_eq!(look_stages(5,2).unwrap(),&stages[..1]);
    assert!(look_stages(usize::MAX,0).is_err());
    assert!(look_stages(5,4).is_err());
}
pub struct MirrorWorker{
    running:Arc<AtomicBool>,
    frames:Arc<AtomicU64>,
    generated:Arc<AtomicU64>,
    started_at:Instant,
    multiplier:u8,
    output_mode:MirrorMode,
    lighting:Arc<RwLock<crate::lighting::LightTuning>>,
    rebuilds:Arc<AtomicU64>,
    preview_limit:Arc<AtomicU32>,
    health:Arc<crate::capture_health::Health>,
    thread:Option<JoinHandle<()>>,
    latest_status:Arc<Mutex<Option<String>>>,
}
impl MirrorWorker{
    pub fn start(pid:u32,look:usize,mode:MirrorMode)->Result<Self,String>{
        Self::start_with_factor(pid,look,mode,1)
    }
    pub fn start_with_factor(pid:u32,look:usize,mode:MirrorMode,factor:u8)->Result<Self,String>{
        Self::start_with_lighting(pid,look,mode,factor,crate::lighting::LightTuning::default())
    }
    pub fn start_with_lighting(pid:u32,look:usize,mode:MirrorMode,factor:u8,
        initial:crate::lighting::LightTuning)->Result<Self,String>{
        Self::start_with_quality(pid,look,mode,factor,initial,0)
    }
    pub fn start_with_quality(pid:u32,look:usize,mode:MirrorMode,factor:u8,
        initial:crate::lighting::LightTuning,quality:u8)->Result<Self,String>{
        Self::start_with_render(pid,look,mode,factor,initial,quality,0,1.0)
    }
    pub fn start_with_render(pid:u32,look:usize,mode:MirrorMode,factor:u8,
        initial:crate::lighting::LightTuning,quality:u8,scale_mode:u8,
        scale_ratio:f32)->Result<Self,String>{
        Self::start_with_custom_render(pid,look,mode,factor,initial,quality,scale_mode,scale_ratio,None)
    }
    pub fn start_with_custom_render(pid:u32,look:usize,mode:MirrorMode,factor:u8,
        initial:crate::lighting::LightTuning,quality:u8,scale_mode:u8,
        scale_ratio:f32,custom:Option<&[usize]>)->Result<Self,String>{
        Self::start_with_processing(pid,look,mode,factor,initial,quality,scale_mode,scale_ratio,custom,100,false)
    }
    pub fn start_with_processing(pid:u32,look:usize,mode:MirrorMode,factor:u8,
        initial:crate::lighting::LightTuning,quality:u8,scale_mode:u8,
        scale_ratio:f32,custom:Option<&[usize]>,processing_percent:u8,low_latency:bool)->Result<Self,String>{
        Self::start_on_display(pid,look,mode,factor,initial,quality,scale_mode,scale_ratio,custom,processing_percent,low_latency,None,Default::default(),None,false,false,false,100.)
    }
    pub fn start_on_display(pid:u32,look:usize,mode:MirrorMode,factor:u8,
        initial:crate::lighting::LightTuning,quality:u8,scale_mode:u8,scale_ratio:f32,
        custom:Option<&[usize]>,processing_percent:u8,low_latency:bool,display:Option<String>,display_grades:crate::display_color::Profiles,renderer_adapter:Option<String>,tearing:bool,composition_requested:bool,hdr_requested:bool,hdr_white_nits:f32)->Result<Self,String>{
        crate::framegen::validate_factor(factor)?;
        if mode==MirrorMode::DesktopDuplication&&factor!=1{return Err("DXGI desktop duplication fallback is 1x only; independent WGC offers x2/x3".into());}
        // Interpolation in composited overlay runs *before* the latest real
        // present; on Polaris this can delay the game image and feel worse.
        // Until low-latency scene-aware sequencing is proven, FG is preview-
        // ONLY, and game-surface glass is always native-real-frame x1.
        if factor>1&&mode==MirrorMode::CompositedInPlace{
            return Err("For responsiveness, experimental 2D FrameGen only runs in a separate preview window. Glass over game stays 1x real frames.".into());
        }
        let preset=crate::native_presets::PRESETS.get(look)
            .ok_or("Invalid native GPU mirror preset")?;
        if quality>3{return Err("GPU preset quality must be Adaptive, Full, Balanced or Eco".into());}
        if pid==0||pid==std::process::id(){return Err("Select another running game".into());}
        let pass_limit=match quality{1=>2,2=>1,_=>8}; // Adaptive keeps source stages, slices per frame.
        let chosen=custom.unwrap_or(preset.stages);
        if custom.is_some(){crate::native_stack::validate(chosen)?;}
        let stages=chosen[..chosen.len().min(pass_limit)].to_vec();
        let latest_status=Arc::new(Mutex::new(None));let status_slot=latest_status.clone();
        let lighting=Arc::new(RwLock::new(initial));
        let tuning=Arc::clone(&lighting);
        let running=Arc::new(AtomicBool::new(true));
        let alive=Arc::clone(&running);
        let frames=Arc::new(AtomicU64::new(0));
        let count=Arc::clone(&frames);
        let generated=Arc::new(AtomicU64::new(0));
        let interpolated=Arc::clone(&generated);
        let rebuilds=Arc::new(AtomicU64::new(0));
        let restarts=Arc::clone(&rebuilds);
        let preview_limit=Arc::new(AtomicU32::new(45));
        let current_preview_limit=Arc::clone(&preview_limit);
        let mut h=crate::capture_health::Health::new();h.preview_display=display;h.display_grades=display_grades;h.renderer_adapter=renderer_adapter;h.tearing_requested=tearing;h.composition_requested=composition_requested;h.hdr_requested=hdr_requested;h.hdr_white_nits=hdr_white_nits.clamp(80.,400.);
        let health=Arc::new(h);
        health.waitable_requested.store(low_latency,Ordering::Relaxed);
        health.live_scale_mode.store(scale_mode as u64+1,Ordering::Release);
        health.live_quality.store(quality as u64+1,Ordering::Release);
        let live_health=health.clone();
        let worker=thread::Builder::new().name("horizon-native-wgc-mirror".into())
            .spawn(move||{
                let status=|s:String|{if let Ok(mut slot)=status_slot.lock(){*slot=Some(s);}};
                let _mmcss=if low_latency{match crate::mmcss_runtime::install(){Ok(g)=>{status("Own capture worker MMCSS Games scheduling active (reverted on worker exit)".into());Some(g)},Err(e)=>{status(format!("Own capture MMCSS unavailable, continuing normal priority: {e}"));None}}}else{None};
                let result=hwnd_for_pid(pid).and_then(|hwnd|{
                    let mut rebuilds=0usize;
                    loop {
                        if !alive.load(Ordering::Acquire){break Ok(0);}
                        live_health.beat();
                        let mut owner=0;unsafe{GetWindowThreadProcessId(hwnd.0,&mut owner);}
                        if owner!=pid{break Err("Selected game window identity changed".into());}
                        let result=if mode==MirrorMode::DesktopDuplication{
                            run_desktop_duplication(hwnd,&stages,&alive,&count,&tuning,&live_health,&status,Some(&current_preview_limit))
                        }else{run_capture_scaled_paced(hwnd,&stages,mode,&alive,&count,&interpolated,
                            factor,&tuning,scale_mode,scale_ratio,quality,&status,None,
                            Some(&current_preview_limit),processing_percent,Some(&live_health))};
                        match result {
                            Err(e) if crate::capture_health::recoverable(&e)
                                && rebuilds<4&&alive.load(Ordering::Acquire)=>{
                                rebuilds+=1;
                                restarts.fetch_add(1,Ordering::Relaxed);
                                status(format!("Recoverable native capture error; re-creating own surfaces ({rebuilds}/4): {e}"));
                                for _ in 0..(rebuilds*5){
                                    if !alive.load(Ordering::Acquire){break;}
                                    std::thread::sleep(Duration::from_millis(100));
                                }
                                if !alive.load(Ordering::Acquire){break Ok(0);}

                            },
                            other=>break other
                        }
                    }
                });
                if let Err(e)=result {status(format!("Native GPU mirror stopped: {e}"));}
                live_health.clear_timings();live_health.waitable_active.store(false,Ordering::Relaxed);live_health.composition_active.store(false,Ordering::Release);live_health.hdr_active.store(false,Ordering::Release);live_health.capture_interval_active.store(false,Ordering::Relaxed);live_health.own_minimized.store(false,Ordering::Relaxed);
                alive.store(false,Ordering::Release);
            }).map_err(|e|format!("Mirror background worker could not start: {e}"))?;
        Ok(Self{running,frames,generated,started_at:Instant::now(),
            multiplier:factor,output_mode:mode,lighting,rebuilds,preview_limit,health,
            thread:Some(worker),latest_status})
    }
    pub fn set_look(&self,look:usize,quality:u8,updated:&crate::lighting::LightTuning)->Result<(),String>{
        let stages=look_stages(look,quality)?;
        self.update_stages(stages,quality,updated)
    }
    pub fn set_stack(&self,ids:&[usize],quality:u8,updated:&crate::lighting::LightTuning)->Result<(),String>{
        crate::native_stack::validate(ids)?;
        self.update_stages(quality_stages(ids,quality)?,quality,updated)
    }
    fn update_stages(&self,stages:Vec<usize>,quality:u8,updated:&crate::lighting::LightTuning)->Result<(),String>{
        let mut selected=self.health.live_stages.write().map_err(|_|"Live look state unavailable")?;
        let mut tuning=self.lighting.write().map_err(|_|"Live tuning unavailable")?;
        *tuning=updated.clone();
        *selected=Some(stages.into());
        self.health.live_quality.store(quality as u64+1,Ordering::Release);
        Ok(())
    }
    pub fn set_lighting(&self,updated:&crate::lighting::LightTuning){
        if let Ok(mut guard)=self.lighting.write(){*guard=updated.clone();}
    }
    pub fn set_scale_mode(&self,mode:u8)->bool{
        if mode as usize>=crate::spatial::MODES.len(){return false;}
        self.health.live_scale_mode.store(mode as u64+1,Ordering::Release);true
    }
    pub fn set_preview_limit(&self,fps:u32){
        self.preview_limit.store(crate::preview_pacer::valid_limit(fps),
            Ordering::Release);
    }
    pub fn preview_limit(&self)->u32{self.preview_limit.load(Ordering::Acquire)}
    pub fn set_vsync(&self,on:bool){self.health.vsync.store(on,Ordering::Release);}
    pub fn diagnostics(&self)->crate::capture_health::Snapshot{self.health.snapshot()}
    pub fn composition_active(&self)->bool{self.health.composition_active.load(Ordering::Acquire)}
    pub fn hdr_active(&self)->bool{self.health.hdr_active.load(Ordering::Acquire)}
    pub fn fused_dispatches_saved(&self)->u64{self.health.fused_saved.load(Ordering::Relaxed)}
    pub fn fence_active(&self)->bool{self.health.fence_active.load(Ordering::Relaxed)}
    pub fn fg_backlog_deferrals(&self)->u64{self.health.fg_backlog.load(Ordering::Relaxed)}
    pub fn nominal_refresh(&self)->Option<u64>{let n=self.health.nominal_refresh.load(Ordering::Relaxed);(n>0).then_some(n)}
    pub fn generated_frames(&self)->u64{self.generated.load(Ordering::Acquire)}
    pub fn multiplier(&self)->u8{self.multiplier}
    pub fn mode(&self)->MirrorMode{self.output_mode}
    /// Average real processed/captured WGC frames per wall-clock second.
    /// This is not source game engine FPS; includes initialization and pauses.
    pub fn capture_average_fps(&self)->Option<f32>{
        let elapsed=self.started_at.elapsed().as_secs_f32();
        let frames=self.frames.load(Ordering::Acquire);
        if elapsed<0.7||frames==0{return None;}
        Some(frames as f32/elapsed)
    }
    /// Average own swapchain presentation rate including generated frames;
    /// NEVER label this as game FPS or input-to-display performance.
    pub fn output_average_fps(&self)->Option<f32>{
        let cap=self.capture_average_fps()?;
        let real=self.frames.load(Ordering::Acquire).max(1);
        let generated=self.generated.load(Ordering::Acquire);
        Some(cap*(real+generated) as f32/real as f32)
    }
    pub fn running(&self)->bool{self.running.load(Ordering::Acquire)}
    pub fn submitted_frames(&self)->u64{self.frames.load(Ordering::Acquire)}
    pub fn rebuild_count(&self)->u64{self.rebuilds.load(Ordering::Relaxed)}
    pub fn poll_status(&self)->Option<String>{
        self.latest_status.lock().ok().and_then(|mut s|s.take())
    }
    pub fn finished(&self)->bool{self.thread.as_ref().is_none_or(|t|t.is_finished())}
    pub fn stalled(&self)->bool{self.running()&&self.started_at.elapsed()>Duration::from_secs(20)&&self.health.heartbeat_age_ms().is_none_or(|ms|ms>5000.)}
    pub fn request_stop(&self){self.running.store(false,Ordering::Release);self.health.clear_timings();}
    pub fn stop(&mut self){
        self.request_stop();
        if let Some(t)=self.thread.take(){let _=t.join();}
        self.health.waitable_active.store(false,Ordering::Relaxed);
        self.health.paused.store(false,Ordering::Relaxed);self.health.clear_timings();
    }
}
impl Drop for MirrorWorker{fn drop(&mut self){self.stop();}}


/// Read-only active-game proof on a specific opt-in HWND/PID.
/// It presents selected GPU filters only in Horizon's independent window,
/// never accesses game memory and never changes target window mode.
/// Unlike synthetic proof, does NOT paint or modify the game's HWND.
pub fn live_game_probe(pid:u32,look:usize,factor:u8)->Result<String,String>{
    crate::framegen::validate_factor(factor)?;
    let hwnd=hwnd_for_pid(pid)?;
    let mut title=[0u16;256];
    let title_len=unsafe{GetWindowTextW(hwnd.0,title.as_mut_ptr(),title.len() as i32)};
    let title=String::from_utf16_lossy(&title[..(title_len.max(0) as usize)]);
    if !title.to_ascii_lowercase().contains("roblox"){
        return Err(format!("Read-only Roblox proof rejected non-Roblox HWND: {title}"));
    }
    let mut worker=MirrorWorker::start_with_factor(pid,look,MirrorMode::SeparateWindow,factor)?;
    let begin=Instant::now();
    let mut last="No frame received".to_string();
    while worker.running()&&worker.submitted_frames()<20&&begin.elapsed()<Duration::from_secs(14){
        if let Some(note)=worker.poll_status(){last=note;}
        std::thread::sleep(Duration::from_millis(60));
    }
    let real=worker.submitted_frames();
    let generated=worker.generated_frames();
    if let Some(note)=worker.poll_status(){last=note;}
    worker.stop();
    if real<8{
        return Err(format!("Real Roblox WGC frames below proof minimum: {real}; generated={generated}; status={last}"));
    }
    if factor>1&&generated<(real.saturating_sub(1))*(factor-1) as u64{
        return Err(format!("Real Roblox FG presents incomplete: {real} real, {generated} generated"));
    }
    Ok(format!("REAL_ROBLOX_WGC=PASS PID={pid} TITLE={title:?} REAL={real} SYNTHETIC={generated} FACTOR={factor} LOOK={look} OWN_GPU_OUTPUT=true GAME_UNMODIFIED=true"))
}
/// Temporary LIVE glass diagnostic; no game memory/process modification.
/// Keeps the compositor active for screenshot comparison, then restores desktop.
pub fn live_roblox_glass_probe(pid:u32,look:usize,factor:u8)->Result<String,String>{
    let hwnd=hwnd_for_pid(pid)?;
    let mut title=[0u16;256];
    let len=unsafe{GetWindowTextW(hwnd.0,title.as_mut_ptr(),title.len() as i32)};
    let title=String::from_utf16_lossy(&title[..len.max(0) as usize]);
    if !title.to_ascii_lowercase().contains("roblox"){
        return Err(format!("Glass proof refused non-Roblox HWND: {title}"));
    }
    let lighting=crate::lighting::LightTuning{
        enabled:true,shadow:0.76,glow:0.16,bounce:0.32,detail:0.33,
        film:0.21,warmth:0.08,mix:0.87,grade:crate::color_grade::Grade::default(),layers:Default::default(),motion:Default::default(),lut:Default::default(),exposure:Default::default(),reconstruction:Default::default(),precision:Default::default(),recipe:Default::default(),cinema:Default::default(),bypass_look:false,
    };
    let mut mirror=MirrorWorker::start_with_lighting(pid,look,
        MirrorMode::CompositedInPlace,factor,lighting)?;
    let start=Instant::now();
    let mut last="WGC starting".to_string();
    let mut once=false;
    while mirror.running()&&start.elapsed()<Duration::from_secs(18){
        if let Some(s)=mirror.poll_status(){last=s;}
        if !once&&mirror.submitted_frames()>=8{
            println!("GLASS_ACTIVE_FOR_DESKTOP_SCREENSHOT PID={pid} REAL={} MODE=GLASS LIGHTING=ON",mirror.submitted_frames());
            once=true;
        }
        std::thread::sleep(Duration::from_millis(90));
    }
    let real=mirror.submitted_frames();
    let synth=mirror.generated_frames();
    if let Some(s)=mirror.poll_status(){last=s;}
    mirror.stop();
    if real<8{
        return Err(format!("Glass refused/paused: {real} frames, {synth} synth, status={last}"));
    }
    Ok(format!("ROBLOX_LIVE_GLASS_GPU=PASS PID={pid} REAL={real} GENERATED={synth} OWN_CLICKTHROUGH_WINDOW=true GAME_WINDOW_UNMODIFIED=true SCREEN_VISIBILITY_NEEDS_VISUAL_PROOF=true"))
}
/// Self-contained, explicitly invoked verification of actual WinRT GPU
/// window capture + HLSL + our visible swapchain with two disposable windows.
/// No user game screenshots or image copies to CPU.
pub fn self_proof()->Result<String,String>{self_proof_with_frames(4)}
pub fn self_proof_with_frames(frame_target:u64)->Result<String,String>{
    if frame_target==0||frame_target>120{return Err("Native WGC test frames must be 1..120".into());}
    let demo=OwnWindow::new("Horizon Boost Local WGC Test - Synthetic Source",24,48,240,130,true)?;
    demo.show();
    paint_test_source(demo.hwnd,0);
    let stop=AtomicBool::new(true);
    let counter=AtomicU64::new(0);
    let frames=run_capture(HWND(demo.hwnd), &[0,2,4], MirrorMode::SeparateWindow,
        &stop,&counter,&AtomicU64::new(0),1,
        &Arc::new(RwLock::new(crate::lighting::LightTuning::default())),
        |_|{},Some(frame_target))?;
    if frames==0{return Err("No WGC test frames captured".into());}
    Ok(format!("NATIVE_WGC_GPU_MIRROR=PASS FRAMES={frames} HLSL_PASSES=3 OWN_VISIBLE_SWAPCHAIN=true GAME_OUTPUT_MODIFIED=false"))
}
/// End-to-end disposable window -> Windows Graphics Capture -> vendor
/// D3D11 EASU+RCAS/NIS/Lanczos3 -> OWN DXGI Present. No game process involved.
/// This catches the old live-WGC guard that refused modes >=8 even though the
/// isolated spatial GPU screenshot proof passed.
pub fn native_vendor_wgc_proof()->Result<String,String>{
    let test=OwnWindow::new("HB Vendor Scaling Synthetic WGC Source",
        123,137,260,150,true)?;
    test.show();
    paint_test_source(test.hwnd,0);
    let run=AtomicBool::new(true);
    let real=AtomicU64::new(0);
    let generated=AtomicU64::new(0);
    let light=Arc::new(RwLock::new(crate::lighting::LightTuning::default()));
    let mut reports=Vec::new();
    for (name,mode) in [("FSR1",8u8),("NIS",9u8),("Lanczos3",10u8)]{
        let count=run_capture_scaled(HWND(test.hwnd),&[2],MirrorMode::SeparateWindow,
            &run,&real,&generated,1,&light,mode,1.5,2,
            |_|{},Some(8))?;
        if count!=8{
            return Err(format!("{name} real WGC capture/present only {count}/8 frames"));
        }
        reports.push(format!("{name}={count}"));
    }
    Ok(format!("NATIVE_WGC_VENDOR_SCALE=PASS {} WGC_CAPTURE=true OWN_DXGI_PRESENT=true USER_GAME_UNMODIFIED=true",
        reports.join(" ")))
}

pub fn waitable_wgc_proof()->Result<String,String>{
    let test=OwnWindow::new("HB waitable queue QA",100,100,320,180,true)?;
    test.show();paint_test_source(test.hwnd,0);
    let run=AtomicBool::new(true);let real=AtomicU64::new(0);let generated=AtomicU64::new(0);
    let light=Arc::new(RwLock::new(crate::lighting::LightTuning::default()));
    let health=crate::capture_health::Health::new();
    health.waitable_requested.store(true,Ordering::Relaxed);
    let count=run_capture_scaled_paced(HWND(test.hwnd),&[39],MirrorMode::SeparateWindow,
        &run,&real,&generated,1,&light,1,1.0,0,|_|{},Some(8),None,100,Some(&health))?;
    let h=health.snapshot();
    if count!=8||!h.waitable_active{return Err("Waitable DXGI proof fell back or missed frames".into());}
    Ok(format!("DXGI_WAITABLE=PASS PRESENTED={count} MAX_QUEUE=1 TIMEOUTS={} OWN_ONLY=true",h.wait_timeouts))
}
pub fn reduced_wgc_proof()->Result<String,String>{
    let test=OwnWindow::new("HB reduced effects QA",100,100,320,180,true)?;
    test.show();paint_test_source(test.hwnd,0);
    let run=AtomicBool::new(true);let real=AtomicU64::new(0);let generated=AtomicU64::new(0);
    let light=Arc::new(RwLock::new(crate::lighting::LightTuning::default()));
    for percent in [50u8,75,100]{
        let count=run_capture_scaled_paced(HWND(test.hwnd),&[50,39,38],MirrorMode::SeparateWindow,
            &run,&real,&generated,1,&light,1,1.0,0,|_|{},Some(8),None,percent,None)?;
        if count!=8{return Err(format!("Reduced WGC {percent}% only {count} frames"));}
    }
    Ok("REDUCED_WGC=PASS LEVELS=50,75,100 PRESENTED=24 HISTOGRAM=true SOURCE_UNCHANGED=true".into())
}

/// Real synthetic-window WGC proof through explicitly ordered custom stacks.
pub fn custom_stack_wgc_proof()->Result<String,String>{
    let test=OwnWindow::new("HB Custom Stack QA Source",100,100,260,150,true)?;
    test.show();paint_test_source(test.hwnd,0);
    let run=AtomicBool::new(true);
    let counter=AtomicU64::new(0);
    let generated=AtomicU64::new(0);
    let light=Arc::new(RwLock::new(crate::lighting::LightTuning::default()));
    for stages in [&[37usize,31,38][..],&[38usize,31,37][..],
        &[37usize,39,40,41,31,42,38,32][..]]{
        crate::native_stack::validate(stages)?;
        let count=run_capture_scaled(HWND(test.hwnd),stages,MirrorMode::SeparateWindow,
            &run,&counter,&generated,1,&light,0,1.0,0,|_|{},Some(8))?;
        if count!=8{return Err("Custom stack WGC did not present 8 frames".into());}
    }
    Ok("NATIVE_CUSTOM_STACK_WGC=PASS STACKS=3 PRESENTED=24 REAL_HLSL=true MAX_STAGES=8".into())
}

/// Actual independent WGC + hardware optical-flow + synthesized-frame
/// presentation proof. Source is an owned colored synthetic window only.
pub fn framegen_synthetic_proof(factor:u8)->Result<String,String>{
    if factor!=2&&factor!=3{return Err("Synthetic FG proof requires 2x/3x".into());}
    let test=OwnWindow::new("HB FrameGen Synthetic Source",126,146,260,156,true)?;
    test.show();
    paint_test_source(test.hwnd,0);
    let run=AtomicBool::new(true);
    let real=AtomicU64::new(0);
    let generated=AtomicU64::new(0);
    let count=14u64;
    let actual=run_capture(HWND(test.hwnd),&[2],MirrorMode::SeparateWindow,
        &run,&real,&generated,factor,
        &Arc::new(RwLock::new(crate::lighting::LightTuning::default())),
        |_|{},Some(count))?;
    let synth=generated.load(Ordering::Acquire);
    let expected=(actual.saturating_sub(1))*(factor-1) as u64;
    if actual!=count||synth!=expected{
        return Err(format!("FG output counters mismatch: real={actual}, generated={synth}, expected={expected}"));
    }
    Ok(format!("EXTERNAL_GPU_FLOW_FRAMEGEN=PASS FACTOR={factor} REAL_FRAMES={actual} SYNTHETIC_PRESENTED={synth} OWN_SWAPCHAIN_ONLY=true GAME_FRAME_RATE_UNCHANGED=true"))
}
/// Disposable separate-process GUI that creates a synthetic composited window.
/// Invoked ONLY by the self-contained validation command; not a user game.
pub fn test_source_child()->Result<(),String>{
    let demo=OwnWindow::new("Horizon Native External WGC Test Source",55,90,240,130,true)?;
    demo.show();
    let start=Instant::now();
    let mut i=0u32;
    while start.elapsed()<Duration::from_secs(8){
        pump_messages();
        paint_test_source(demo.hwnd,i);
        i=i.wrapping_add(1);
        std::thread::sleep(Duration::from_millis(32));
    }
    Ok(())
}
/// Changes only the disposable synthetic *test* window dimensions twice.
/// A real game's monitor resolution, HWND and fullscreen mode are untouched.
pub fn test_source_resize_child()->Result<(),String>{
    let demo=OwnWindow::new("HB WGC Synthetic Resizing Source",74,108,260,150,true)?;
    demo.show();
    let start=Instant::now();
    let mut phase=0u32;
    let mut i=0u32;
    while start.elapsed()<Duration::from_secs(8){
        pump_messages();
        let e=start.elapsed();
        if phase==0&&e>=Duration::from_millis(1200){
            if unsafe{SetWindowPos(demo.hwnd,ptr::null_mut(),74,108,340,200,0x14)}==0{
                return Err("Synthetic resize to 340x200 failed".into());
            }
            phase=1;
        }else if phase==1&&e>=Duration::from_millis(3200){
            if unsafe{SetWindowPos(demo.hwnd,ptr::null_mut(),74,108,280,170,0x14)}==0{
                return Err("Synthetic resize to 280x170 failed".into());
            }
            phase=2;
        }
        paint_test_source(demo.hwnd,i);
        i=i.wrapping_add(1);
        std::thread::sleep(Duration::from_millis(33));
    }
    if phase!=2{return Err("Synthetic source did not exercise two resizes".into());}
    Ok(())
}
pub fn resize_recovery_proof()->Result<String,String>{
    use std::os::windows::process::CommandExt;
    let exe=std::env::current_exe().map_err(|e|e.to_string())?;
    let mut child=std::process::Command::new(exe)
        .arg("--native-test-source-resizing")
        .creation_flags(0x0800_0000)
        .spawn().map_err(|e|format!("Synthetic resize child launch: {e}"))?;
    let pid=child.id();
    let result=(||->Result<String,String>{
        let ready=Instant::now();
        while hwnd_for_pid(pid).is_err()&&ready.elapsed()<Duration::from_secs(3){
            std::thread::sleep(Duration::from_millis(40));
        }
        if hwnd_for_pid(pid).is_err(){return Err("Synthetic resizing source HWND missing".into());}
        let mut mirror=MirrorWorker::start(pid,2,MirrorMode::SeparateWindow)?;
        let start=Instant::now();
        while mirror.running()&&start.elapsed()<Duration::from_millis(6200){
            std::thread::sleep(Duration::from_millis(50));
        }
        let restarts=mirror.rebuild_count();
        let frames=mirror.submitted_frames();
        let last=mirror.poll_status();
        mirror.stop();
        if restarts<2{return Err(format!(
            "Expected 2 WGC rebuilds, observed {restarts}; frames={frames}; last={last:?}"));}
        if frames<28{return Err(format!("Resizing WGC only presented {frames} real frames"));}
        Ok(format!("WGC_LIVE_RESIZE_RECOVERY=PASS REBUILDS={restarts} FRAMES={frames} SOURCE_PID={pid} GAME_SETTINGS_UNTOUCHED=true"))
    })();
    let _=child.kill();
    let _=child.wait();
    result
}
/// Explicit opt-in verification of real cross-process WGC capture on Polaris.
/// Never captures games/desktop; owns and closes both disposable test processes.
pub fn external_process_proof()->Result<String,String>{
    use std::os::windows::process::CommandExt;
    let exe=std::env::current_exe().map_err(|e|format!("Test executable: {e}"))?;
    let mut child=std::process::Command::new(exe)
        .arg("--native-test-source-window")
        .creation_flags(0x0800_0000)
        .spawn().map_err(|e|format!("Spawn synthetic WGC source: {e}"))?;
    let pid=child.id();
    let result=(||->Result<String,String>{
        let start=Instant::now();
        let mut ready=false;
        while start.elapsed()<Duration::from_secs(3){
            if hwnd_for_pid(pid).is_ok(){ready=true;break;}
            std::thread::sleep(Duration::from_millis(45));
        }
        if !ready{return Err("Synthetic external window never became visible".into());}
        let mut worker=MirrorWorker::start(pid,5,MirrorMode::SeparateWindow)?; // Amber GPU scene on real synthetic source
        let begin=Instant::now();
        while worker.running()&&worker.submitted_frames()<30&&begin.elapsed()<Duration::from_secs(6){
            std::thread::sleep(Duration::from_millis(50));
        }
        let frames=worker.submitted_frames();
        let note=worker.poll_status();
        worker.stop();
        if frames<15{return Err(format!("Only {frames} external-process GPU frames; {note:?}"));}
        Ok(format!("WGC_EXTERNAL_PID_GPU_MIRROR=PASS SOURCE_PID={pid} FRAMES={frames} LOOK=Amber NO_GAME_MODIFICATIONS=true"))
    })();
    let _=child.kill();
    let _=child.wait();
    result
}
/// Experimental same-rectangle glass test against our own synthetic window.
/// Does not use Magpie or alter the synthetic source HWND. This checks native
/// frame presentation and Win32 click-through flags; *not exclusive game FSE*.
pub fn glass_self_proof()->Result<String,String>{glass_self_proof_frames(30)}
pub fn glass_self_proof_frames(target_frames:u64)->Result<String,String>{
    if !(1..=150).contains(&target_frames){return Err("Glass proof frames out of bounds".into());}
    let demo=OwnWindow::new("HB Glass Safety Synthetic Source",88,104,240,130,true)?;
    demo.show();
    // Keep the disposable source above unrelated desktop windows, without
    // activating it. The subsequently created glass must remain click-through.
    if unsafe{SetWindowPos(demo.hwnd,(-1isize) as H,0,0,0,0,0x0013)}==0{
        return Err("Could not position isolated glass QA source".into());
    }
    let original=source_rect(HWND(demo.hwnd))?;
    let foreground_before=unsafe{GetForegroundWindow()};
    let stop=AtomicBool::new(true);
    let frames=AtomicU64::new(0);
    // Monochrome effect makes proof screenshots visually unambiguous. Input
    // synthetic source is multicolored, output should become grayscale.
    let n=run_capture(HWND(demo.hwnd),&[7],
        MirrorMode::CompositedInPlace,&stop,&frames,&AtomicU64::new(0),1,
        &Arc::new(RwLock::new(crate::lighting::LightTuning::default())),
        |_|{},Some(target_frames))?;
    let after=source_rect(HWND(demo.hwnd))?;
    if unsafe{GetForegroundWindow()}!=foreground_before{
        return Err("Glass test changed the Windows foreground focus".into());
    }
    let same=(original.left,original.top,original.right,original.bottom)==
        (after.left,after.top,after.right,after.bottom);
    if !same{return Err("Glass proof unexpectedly changed source HWND geometry".into());}
    if n<target_frames{return Err(format!("Glass proof only presented {n}/{target_frames} frames"));}
    Ok(format!("NATIVE_GPU_COMPOSITED_GLASS=PASS FRAMES={n} HIT_TEST_CLICKS_TO_SOURCE=true SOURCE_HWND_UNMODIFIED=true FOCUS_UNCHANGED=true EXCLUSIVE_GAME_OUTPUT_UNVERIFIED=true"))
}
#[cfg(test)]
mod tests{
    #[test]fn adaptive_pass_budget_conserves_gpu_at_low_capture_rate(){
        assert_eq!(super::adaptive_pass_limit(1,25.0,5),1);
        assert_eq!(super::adaptive_pass_limit(1,60.0,5),2);
        assert_eq!(super::adaptive_pass_limit(2,47.0,5),2);
        assert_eq!(super::adaptive_pass_limit(2,30.0,5),1);
        assert_eq!(super::adaptive_pass_limit(2,90.0,5),3);
        assert_eq!(super::adaptive_pass_limit(3,45.0,5),2);
        assert_eq!(super::adaptive_pass_limit(2,90.0,1),1);
        assert_eq!(super::adaptive_pass_limit(2,90.0,0),0);
        assert_eq!(super::adaptive_pass_limit(1,f32::NAN,3),1);
    }

    use super::*;
    #[test]fn frame_generation_never_starts_on_game_glass(){
        // This must reject BEFORE HWND selection/worker startup; no game
        // process or visible window is required to verify the guard.
        for factor in [2,3]{
            let result=MirrorWorker::start_with_render(1234,0,
                MirrorMode::CompositedInPlace,factor,
                crate::lighting::LightTuning::default(),3,0,1.0);
            assert!(matches!(result,Err(ref reason) if reason.contains("separate preview")));
        }
    }
    #[test]fn mirror_refuses_null_and_self_process(){
        assert!(hwnd_for_pid(0).is_err());
        assert!(hwnd_for_pid(std::process::id()).is_err());
        assert!(MirrorWorker::start(0,0,MirrorMode::SeparateWindow).is_err());
        assert!(MirrorWorker::start(1234,usize::MAX,MirrorMode::CompositedInPlace).is_err());
    }
}

#[cfg(test)]mod live_quality_tests{
 use super::*;
 #[test]fn custom_stack_quality_preserves_order_and_repetitions(){let ids=[5,1,5,2,9];assert_eq!(quality_stages(&ids,0).unwrap(),ids);assert_eq!(quality_stages(&ids,1).unwrap(),[5,1]);assert_eq!(quality_stages(&ids,2).unwrap(),[5]);assert_eq!(quality_stages(&ids,3).unwrap(),ids);assert!(quality_stages(&ids,4).is_err());}
}

#[cfg(test)]mod v101_cursor_size_tests{
 use super::*;
 #[test]fn scaled_arrow_is_opt_in_and_bounded(){
  assert_eq!(cursor_pixels(32,100),None);
  assert_eq!(cursor_pixels(32,150),Some(48));
  assert_eq!(cursor_pixels(32,200),Some(64));
  assert_eq!(cursor_pixels(64,200),Some(128));
  assert_eq!(cursor_pixels(5,200),None);
  assert_eq!(cursor_pixels(32,255),None);
 }
 #[test]fn default_never_touches_pointer(){
  configure_preview_cursor_scale(100);
  assert_eq!(OWN_CURSOR_PERCENT.load(Ordering::Relaxed),100);
  configure_preview_cursor_scale(255);
  assert_eq!(OWN_CURSOR_PERCENT.load(Ordering::Relaxed),100);
 }
 #[test]fn system_arrow_resource_is_available_without_visible_window(){
  let pixel=cursor_pixels(32,150).unwrap();
  let arrow=unsafe{LoadImageW(ptr::null_mut(),32512usize as *const u16,2,pixel,pixel,0x8000)};
  assert!(!arrow.is_null(),"Win32 system cursor size resource not available");
 }
}

/// Internal integration fixture: WGC -> native GPU look -> own DXGI swapchain Present.
#[cfg(test)]
pub(crate) fn test_output_fixture_present(hwnd:HWND)->Result<(u64,u64),String>{
 // Internal run_capture takes a *running* boolean, despite its stop parameter name.
 let stop=AtomicBool::new(true);
 let frames=AtomicU64::new(0);
 let generated=AtomicU64::new(0);
 let lighting=Arc::new(RwLock::new(crate::lighting::LightTuning::default()));
 let shown=run_capture(hwnd,&[0],MirrorMode::SeparateWindow,&stop,&frames,
  &generated,1,&lighting,|message|{eprintln!("WGC_PRESENT_FIXTURE: {message}");},Some(1))?;
 Ok((shown,frames.load(Ordering::Acquire)))
}
