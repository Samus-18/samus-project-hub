pub(crate) const HUD_TITLE:&str="Vanta FX - HUD";
pub(crate) const MACRO_HUD_TITLE:&str="Vanta FX - Macro";
pub(crate) const MAIN_TITLE:&str="Vanta FX - Native GPU Tools";
// Windows top-level game detection. Does not inject, hook or alter any game process.
use std::{ffi::c_void,path::{Path,PathBuf},process::{Command,Stdio}};
use std::os::windows::process::CommandExt;
use sysinfo::{Pid,System,ProcessesToUpdate};
type Handle=*mut c_void;
const PROCESS_QUERY_LIMITED_INFORMATION:u32=0x1000;
const CREATE_NO_WINDOW:u32=0x0800_0000;
#[link(name="user32")]
unsafe extern "system" {
    fn GetForegroundWindow()->Handle;
    fn GetWindowThreadProcessId(hwnd:Handle,pid:*mut u32)->u32;
    fn GetWindowTextW(hwnd:Handle,buffer:*mut u16,max:i32)->i32;
    fn GetClassNameW(hwnd:Handle,buffer:*mut u16,max:i32)->i32;
    fn GetAsyncKeyState(vkey:i32)->i16;
    fn GetSystemMetrics(index:i32)->i32;
    fn IsZoomed(hwnd:Handle)->i32;
    fn GetWindowRect(hwnd:Handle,rect:*mut WinRect)->i32;
    fn GetDpiForWindow(hwnd:Handle)->u32;
    fn EnumWindows(callback:unsafe extern "system" fn(Handle,isize)->i32,data:isize)->i32;
    fn IsWindowVisible(hwnd:Handle)->i32;
    fn IsIconic(hwnd:Handle)->i32;
    fn MonitorFromWindow(hwnd:Handle,flags:u32)->Handle;
    fn GetMonitorInfoW(monitor:Handle,info:*mut WinMonitorInfo)->i32;
    fn GetWindow(hwnd:Handle,cmd:u32)->Handle;
    fn FindWindowW(class:*const u16,title:*const u16)->Handle;
    fn GetPropW(hwnd:Handle,name:*const u16)->Handle;
    fn SetWindowPos(hwnd:Handle,after:Handle,x:i32,y:i32,cx:i32,cy:i32,flags:u32)->i32;
    fn GetWindowLongW(hwnd:Handle,index:i32)->i32;
    fn SetWindowLongW(hwnd:Handle,index:i32,value:i32)->i32;
    fn SetLayeredWindowAttributes(hwnd:Handle,key:u32,alpha:u8,flags:u32)->i32;
    fn RegisterWindowMessageW(name:*const u16)->u32;
    fn PostMessageW(hwnd:Handle,message:u32,wparam:usize,lparam:isize)->i32;
}
// Optional DWM status without linking a vendor import lib (mingw/LLD may lack -ldwmapi).
// Keep module loaded once for the lifetime of the cached function pointer.
type DwmGetWindowAttributeFn=unsafe extern "system" fn(Handle,u32,*mut c_void,u32)->i32;
pub fn is_dwm_cloaked(hwnd:Handle)->bool{
 static DWM:std::sync::OnceLock<Option<DwmGetWindowAttributeFn>>=std::sync::OnceLock::new();
 let f=DWM.get_or_init(||unsafe{
  let module=LoadLibraryA(b"dwmapi.dll\0".as_ptr());
  if module.is_null(){return None;}
  let proc=GetProcAddress(module,b"DwmGetWindowAttribute\0".as_ptr());
  if proc.is_null(){return None;}
  Some(std::mem::transmute::<*const c_void,DwmGetWindowAttributeFn>(proc))
 });
 let Some(get)=f else{return false;};
 let mut cloak=0u32;
 unsafe{get(hwnd,14,&mut cloak as *mut u32 as *mut c_void,4)==0&&cloak!=0}
}
#[link(name="kernel32")]
unsafe extern "system" {
    fn LoadLibraryA(name:*const u8)->Handle;
    fn OpenProcess(access:u32,inherit:i32,pid:u32)->Handle;
    fn QueryFullProcessImageNameW(process:Handle,flags:u32,buffer:*mut u16,size:*mut u32)->i32;
    fn CloseHandle(handle:Handle)->i32;
}
#[repr(C)]
#[derive(Default)]
struct WinRect{left:i32,top:i32,right:i32,bottom:i32}
#[repr(C)]
#[derive(Default)]
struct WinMonitorInfo{size:u32,monitor:WinRect,work:WinRect,flags:u32}
pub fn window_monitor_bounds(hwnd:Handle)->Option<(i32,i32,u32,u32)>{
 let mon=unsafe{MonitorFromWindow(hwnd,2)};
 if mon.is_null(){return None;}
 let mut info=WinMonitorInfo{size:std::mem::size_of::<WinMonitorInfo>()as u32,..Default::default()};
 if unsafe{GetMonitorInfoW(mon,&mut info)}==0{return None;}
 let w=(info.monitor.right-info.monitor.left).max(0)as u32;
 let h=(info.monitor.bottom-info.monitor.top).max(0)as u32;
 (w>=640&&h>=480&&w<=16384&&h<=16384).then_some((info.monitor.left,info.monitor.top,w,h))
}
fn monitor_dimensions(hwnd:Handle)->Option<(u32,u32)>{
 window_monitor_bounds(hwnd).map(|(_,_,w,h)|(w,h))
}
#[derive(Clone,Debug)]
pub struct GameInfo{
    pub pid:u32,pub title:String,pub path:PathBuf,
    pub class_name:String,
    /// Covers monitor or maximized. This DOES NOT prove the use of exclusive fullscreen.
    pub fullscreen_like:bool
}
pub struct Telemetry {
    system:System,
    pub active_game:Option<GameInfo>,
    pub visible_games:Vec<GameInfo>,
    pub selected_pid:Option<u32>,
    pub show_capture_windows:bool,
    pub cpu_percent:f32,
    pub game_ram_mb:u64,
    pub used_ram_gb:f64,
    pub total_ram_gb:f64,
    pub cpu_history:Vec<f32>,
}
impl Telemetry {
    pub fn new()->Self{
        Self{
            system:System::new(),active_game:None,visible_games:Vec::new(),selected_pid:None,show_capture_windows:false,
            cpu_percent:0.0,game_ram_mb:0,used_ram_gb:0.0,total_ram_gb:0.0,cpu_history:Vec::new()
        }
    }
    pub fn select_game(&mut self,pid:u32)->bool{
        if let Some(g)=self.visible_games.iter().find(|g|g.pid==pid).cloned(){
            self.selected_pid=Some(pid);
            self.active_game=Some(g);
            return true;
        }
        false
    }
    pub fn unlock_game(&mut self){self.selected_pid=None;}
    pub fn refresh(&mut self,detect:bool){
        let previous=self.active_game.as_ref().map(|g|g.pid);
        // Refresh only the active game PID, not every process in Windows each second.
        self.system.refresh_memory();
        self.used_ram_gb=self.system.used_memory() as f64/1_073_741_824.0;
        self.total_ram_gb=self.system.total_memory() as f64/1_073_741_824.0;
        if detect {
            self.visible_games=scan_visible_games();
            if self.show_capture_windows{
                for g in scan_captureable_windows(){if !self.visible_games.iter().any(|x|x.pid==g.pid){self.visible_games.push(g);}}
            }
            let prior_pid=self.active_game.as_ref().map(|g|g.pid);
            let current_is_running=prior_pid.is_some_and(|pid|{
                let game_pid=Pid::from_u32(pid);
                self.system.refresh_processes(ProcessesToUpdate::Some(&[game_pid]),true);
                self.system.process(game_pid).is_some()
            });
            if !current_is_running{self.selected_pid=None;self.active_game=None;}
            let foreground=if self.selected_pid.is_none(){foreground_game()}else{None};
            self.active_game=select_active(self.selected_pid,self.active_game.as_ref(),&self.visible_games,foreground);

        } else {
            self.visible_games.clear();
        }
        if let Some(g)=&self.active_game {
            let game_pid=Pid::from_u32(g.pid);
            self.system.refresh_processes(ProcessesToUpdate::Some(&[game_pid]),true);
            if let Some(p)=self.system.process(game_pid){
                self.cpu_percent=p.cpu_usage();
                self.game_ram_mb=p.memory()/1_048_576;
            } else {
                self.active_game=None;
                self.cpu_percent=0.0;self.game_ram_mb=0;self.selected_pid=None;
            }
        }else{self.cpu_percent=0.0;self.game_ram_mb=0;}
        if previous!=self.active_game.as_ref().map(|g|g.pid){self.cpu_history.clear();}
        if self.active_game.is_some(){self.cpu_history.push(self.cpu_percent);}else{self.cpu_history.clear();}
        if self.cpu_history.len()>90{self.cpu_history.remove(0);}
    }
}
fn own_window(title:&str)->Handle{
 struct Match<'a>{title:&'a str,hwnd:Handle}
 unsafe extern "system" fn cb(hwnd:Handle,data:isize)->i32{
  let out=unsafe{&mut *(data as *mut Match<'_>)};let mut pid=0;
  unsafe{GetWindowThreadProcessId(hwnd,&mut pid);}
  if pid!=std::process::id(){return 1;}
  let mut b=[0u16;256];let n=unsafe{GetWindowTextW(hwnd,b.as_mut_ptr(),b.len() as i32)};
  if n>0&&String::from_utf16_lossy(&b[..n as usize])==out.title{out.hwnd=hwnd;return 0;}1
 }
 let mut data=Match{title,hwnd:std::ptr::null_mut()};unsafe{EnumWindows(cb,&mut data as *mut _ as isize);}data.hwnd
}
fn wstr(s:&str)->Vec<u16>{s.encode_utf16().chain(std::iter::once(0)).collect()}
unsafe fn game_info_from_window(hwnd:Handle)->Option<GameInfo>{unsafe{window_info(hwnd,true)}}
unsafe fn window_info(hwnd:Handle,game_only:bool)->Option<GameInfo>{
    if hwnd.is_null(){return None;}
    let mut pid=0u32;
    unsafe{GetWindowThreadProcessId(hwnd,&mut pid);}
    if pid==0||pid==std::process::id(){return None;}
    let mut name=[0u16;512];
    let n=unsafe{GetWindowTextW(hwnd,name.as_mut_ptr(),512)};
    if n<=0{return None;}
    let title=String::from_utf16_lossy(&name[..n as usize]);
    if game_only&&!allowed_game_window_title(&title){return None;}
    let handle=unsafe{OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,0,pid)};
    if handle.is_null(){return None;}
    let mut buffer=[0u16;32768];
    let mut len=buffer.len() as u32;
    let ok=unsafe{QueryFullProcessImageNameW(handle,0,buffer.as_mut_ptr(),&mut len)};
    unsafe{CloseHandle(handle);}
    if ok==0{return None;}
    let path=PathBuf::from(String::from_utf16_lossy(&buffer[..len as usize]));
    let mut class=[0u16;256];
    let c=unsafe{GetClassNameW(hwnd,class.as_mut_ptr(),256)};
    let class_name=if c>0{String::from_utf16_lossy(&class[..c as usize])}else{String::new()};
    if game_only&&!is_probable_game(&path)&&!is_game_engine_path(&path)&&!is_minecraft_java_window(&path,&title,&class_name){return None;}
    let mut rect=WinRect::default();
    let good=unsafe{GetWindowRect(hwnd,&mut rect)}!=0;
    let (sw,sh)=screen_size();
    let covers=good && rect.left<=2 && rect.top<=2 &&
        rect.right>=sw as i32-8 && rect.bottom>=sh as i32-8;
    let maximized=unsafe{IsZoomed(hwnd)}!=0;
    Some(GameInfo{pid,title,path,class_name,fullscreen_like:covers||maximized})
}
unsafe extern "system" fn enumerate(hwnd:Handle,state:isize)->i32{
    if unsafe{IsWindowVisible(hwnd)}==0{return 1;}
    // Ignore owned popups and invisible helper windows.
    if !unsafe{GetWindow(hwnd,4)}.is_null(){return 1;}
    if let Some(info)=unsafe{game_info_from_window(hwnd)} {
        let list=unsafe{&mut *(state as *mut Vec<GameInfo>)};
        if !list.iter().any(|g|g.pid==info.pid){list.push(info);}
    }
    1
}
pub fn is_minecraft_java_window(path:&Path,title:&str,class:&str)->bool{
    let binary=path.file_stem().map(|v|v.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    if binary!="javaw"&&binary!="java"{return false;}
    let name=title.to_ascii_lowercase();
    let class=class.to_ascii_lowercase();
    name.contains("minecraft")||class.contains("glfw30")||class.contains("lwjgl")
}
fn allowed_manual_window(g:&GameInfo)->bool{
    if !allowed_game_window_title(&g.title){return false;}
    let exe=g.path.file_stem().map(|x|x.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    !["explorer","dwm","winlogon","taskmgr","powershell","cmd","conhost","searchhost","applicationframehost","systemsettings","horizon-boost-rust"].contains(&exe.as_str())
}
unsafe extern "system" fn enumerate_manual(hwnd:Handle,state:isize)->i32{
    if unsafe{IsWindowVisible(hwnd)}==0||!unsafe{GetWindow(hwnd,4)}.is_null(){return 1;}
    if let Some(g)=unsafe{window_info(hwnd,false)} {
        let list=unsafe{&mut *(state as *mut Vec<GameInfo>)};
        if allowed_manual_window(&g)&&!list.iter().any(|x|x.pid==g.pid){list.push(g);}
    }
    1
}
pub fn scan_captureable_windows()->Vec<GameInfo>{
    let mut out=Vec::new();
    unsafe{EnumWindows(enumerate_manual,&mut out as *mut Vec<GameInfo> as isize);}
    out.truncate(100);out
}
pub fn scan_visible_games()->Vec<GameInfo>{
    let mut out:Vec<GameInfo>=Vec::new();
    unsafe{EnumWindows(enumerate,&mut out as *mut Vec<GameInfo> as isize);}
    out
}
pub fn foreground_hud_app()->Option<(u32,String)>{
 let g=unsafe{window_info(GetForegroundWindow(),false)}?;
 Some((g.pid,g.path.file_stem()?.to_string_lossy().into_owned()))
}
pub fn f12_down()->bool{unsafe{GetAsyncKeyState(0x7b) as u16&0x8000!=0}}
pub fn shortcut_modifiers_down()->bool{[0x10,0x11,0x12,0x5b,0x5c].iter().any(|&k|unsafe{GetAsyncKeyState(k) as u16&0x8000!=0})}
pub fn foreground_game()->Option<GameInfo>{
    unsafe{game_info_from_window(GetForegroundWindow())}
}
pub fn allowed_game_window_title(title:&str)->bool{
    let s=title.to_ascii_lowercase();
    !s.trim().is_empty() && ![
        "mod manager","launcher","patcher","updater",
        "game settings","installer","crash reporter"
    ].iter().any(|bad|s.contains(bad))
}
pub fn is_game_engine_path(path:&Path)->bool{
    let Some(dir)=path.parent()else{return false};
    let stem=path.file_stem().map(|x|x.to_string_lossy().to_string()).unwrap_or_default();
    dir.join(format!("{stem}_Data")).is_dir()||
    dir.join(format!("{stem}.pck")).is_file()||
    ["UnityPlayer.dll","GameAssembly.dll","steam_api64.dll",
        "EOSSDK-Win64-Shipping.dll","GodotSharp.dll"].iter().any(|n|dir.join(n).is_file())
}
pub fn is_probable_game(path:&Path)->bool{
    let name=path.file_stem().map(|s|s.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    let full=path.to_string_lossy().to_ascii_lowercase().replace('/',"\\");
    if ["explorer","chrome","firefox","msedge","opera","discord","steam",
        "steamwebhelper","notepad","code","cursor","powershell","cmd","taskmgr",
        "horizon-boost-rust","epicgameslauncher","gamebar","obs64","magpie",
        "presentmon-2.6.0-x64","horizon-launcher"].contains(&name.as_str())
        || name.contains("modmanager")||name.contains("mod_manager")
        || name.contains("launcher")||name.contains("updater") {return false;}
    full.contains("\\steamapps\\common\\")||
    full.contains("\\games\\")||
    full.contains("\\epic games\\")||
    full.contains("\\gog galaxy\\")||
    full.contains("\\minecraft\\")||
    full.contains("\\horizon\\")||
    full.contains("\\roblox\\versions\\")||
    name.contains("shipping")||name.contains("minecraft")||
    name.starts_with("horizon-client")||name.starts_with("totk")
}
pub fn overlay_hotkey_down()->bool{
    unsafe{
        let alt_z=(GetAsyncKeyState(0x12) as u16 & 0x8000)!=0 &&
            (GetAsyncKeyState(0x5A) as u16 & 0x8000)!=0 &&
            (GetAsyncKeyState(0x10) as u16 & 0x8000)==0;
        let ctrl_shift_h=(GetAsyncKeyState(0x11) as u16 & 0x8000)!=0 &&
            (GetAsyncKeyState(0x10) as u16 & 0x8000)!=0 &&
            (GetAsyncKeyState(0x48) as u16 & 0x8000)!=0;
        alt_z||ctrl_shift_h
    }
}
/// Alt+Shift+Z: unlock the compact mouse-pass-through HUD into an
/// explicitly clickable editor. The ordinary Alt+Z handler excludes Shift.
pub fn overlay_editor_hotkey_down()->bool{
    unsafe{
        (GetAsyncKeyState(0x12) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x10) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x5A) as u16&0x8000)!=0
    }
}
/// Ctrl+Shift+J toggles Windows display color even if the control window is out of focus.
/// Ctrl+Shift+K requests AMD driver-level image sharpening toggling.
/// Ctrl+Shift+B toggles opt-in Windows game priority, even if the overlay is hidden.
/// Ctrl+Shift+P: capture current display with no key injection.
pub fn capture_hotkey_down()->bool{
    unsafe{
        (GetAsyncKeyState(0x11) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x10) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x50) as u16&0x8000)!=0
    }
}
/// Ctrl+Shift+R: start / stop own desktop recorder.
pub fn record_hotkey_down()->bool{
    unsafe{
        (GetAsyncKeyState(0x11) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x10) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x52) as u16&0x8000)!=0
    }
}
/// Ctrl+Shift+G starts or stops the independent external GPU renderer.
/// This never injects input or code into the game process.
pub fn gpu_renderer_hotkey_down()->bool{chord(0x47)}
/// Ctrl+Shift+X shows/hides the independent click-through crosshair.
/// Ctrl+Shift+Y: compare two stored per-game graphical looks.
pub fn compare_hotkey_down()->bool{chord(0x59)}
/// Ctrl+Shift+T: toggle independent game stopwatch.
pub fn timer_hotkey_down()->bool{chord(0x54)}
/// Ctrl+Shift+M: add a stopwatch split.
pub fn split_hotkey_down()->bool{chord(0x4D)}
pub fn replay_hotkey_down()->bool{chord(0x4C)}
pub fn chord(key:i32)->bool{
    unsafe{
        (GetAsyncKeyState(0x11) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x10) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(key) as u16&0x8000)!=0
    }
}
pub fn crosshair_hotkey_down()->bool{
    unsafe{
        (GetAsyncKeyState(0x11) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x10) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x58) as u16&0x8000)!=0
    }
}
pub fn game_boost_hotkey_down()->bool{
    unsafe{
        (GetAsyncKeyState(0x11) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x10) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x42) as u16&0x8000)!=0
    }
}
pub fn driver_ris_hotkey_down()->bool{
    unsafe{
        (GetAsyncKeyState(0x11) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x10) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x4B) as u16&0x8000)!=0
    }
}
pub fn display_color_hotkey_down()->bool{
    unsafe {
        (GetAsyncKeyState(0x11) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x10) as u16&0x8000)!=0 &&
        (GetAsyncKeyState(0x4A) as u16&0x8000)!=0
    }
}
pub fn screen_size()->(u32,u32){
    unsafe{(GetSystemMetrics(0).max(640) as u32,GetSystemMetrics(1).max(480) as u32)}
}
#[link(name="shell32")]
unsafe extern "system" {
    fn ShellExecuteW(hwnd:Handle,operation:*const u16,file:*const u16,
        parameters:*const u16,directory:*const u16,show:i32)->Handle;
}
pub const WINDOWS_GAME_TOOLS:&[(&str,&str)]=&[
    ("Game Mode","ms-settings:gaming-gamemode"),
    ("Windows Graphics Preferences","ms-settings:display-advancedgraphics"),
    ("Display / Refresh Rate","ms-settings:display"),
    ("Game Audio Mixer","ms-settings:apps-volume")
];
pub fn open_game_tool(index:usize)->Result<(),String>{
    let Some((_,uri))=WINDOWS_GAME_TOOLS.get(index)else{
        return Err("Invalid Windows settings shortcut".into());
    };
    let target=wstr(uri);
    let action=wstr("open");
    let result=unsafe{ShellExecuteW(std::ptr::null_mut(),action.as_ptr(),
        target.as_ptr(),std::ptr::null(),std::ptr::null(),1)};
    if (result as isize)<=32{
        return Err(format!("Windows could not open this Settings page (error {})",
            result as isize));
    }
    Ok(())
}
pub fn gpu_name()->String{
    let output=Command::new("powershell.exe").args([
        "-NoProfile","-NonInteractive","-Command",
        "(Get-CimInstance Win32_VideoController | Where-Object {$_.Name -match 'AMD|Radeon|NVIDIA|Intel'} | Select-Object -First 1 -ExpandProperty Name)"
    ]).creation_flags(CREATE_NO_WINDOW)
    .stdout(Stdio::piped()).stderr(Stdio::null()).output();
    match output {
        Ok(v)=>{
            let name=String::from_utf8_lossy(&v.stdout).trim().to_owned();
            if name.is_empty(){"GPU telemetry pending".into()}else{name}
        }
        Err(_)=>"GPU telemetry pending".into()
    }
}
#[repr(C)]
struct MessageTarget{pid:u32,message:u32,count:u32}
unsafe extern "system" fn post_message_owned_window(hwnd:Handle,state:isize)->i32{
    let ctx=unsafe{&mut *(state as *mut MessageTarget)};
    let mut pid=0u32;
    unsafe{GetWindowThreadProcessId(hwnd,&mut pid);}
    if pid==ctx.pid{
        if unsafe{PostMessageW(hwnd,ctx.message,0,0)}!=0{ctx.count+=1;}
    }
    1
}
/// Sends a registered IPC message ONLY to top-level windows owned by the requested
/// process. Never broadcast a quit message to unrelated Magpie instances.
pub fn post_registered_message_to_pid(pid:u32,registered_name:&str)->bool{
    let msg=unsafe{RegisterWindowMessageW(wstr(registered_name).as_ptr())};
    if msg==0{return false;}
    let mut target=MessageTarget{pid,message:msg,count:0};
    unsafe{EnumWindows(post_message_owned_window,&mut target as *mut MessageTarget as isize);}
    target.count>0
}
/// Magpie's documented, stable scaling-window class (no game hooks).
pub const MAGPIE_SCALING_CLASS:&str="Window_Magpie_967EB565-6F73-4E94-AE53-00CC42592A22";
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct MagpieScalingInfo {
    pub windowed:bool,
    pub source_pid:u32,
}
/// Detects a REAL visible scaling output and matches the original source HWND
/// to the selected game PID. Mere Magpie.exe process startup is not enough.
pub fn magpie_scaling_info(game_pid:u32)->Option<MagpieScalingInfo>{
    if game_pid==0{return None;}
    let class=wstr(MAGPIE_SCALING_CLASS);
    let hwnd=unsafe{FindWindowW(class.as_ptr(),std::ptr::null())};
    if hwnd.is_null()||unsafe{IsWindowVisible(hwnd)}==0{return None;}
    let source=unsafe{GetPropW(hwnd,wstr("Magpie.SrcHWND").as_ptr())};
    if source.is_null(){return None;} // not initialized / not actively scaling
    let mut source_pid=0u32;
    unsafe{GetWindowThreadProcessId(source,&mut source_pid);}
    if source_pid!=game_pid{return None;}
    let windowed=!unsafe{GetPropW(hwnd,wstr("Magpie.Windowed").as_ptr())}.is_null();
    Some(MagpieScalingInfo{windowed,source_pid})
}
/// Magpie may create a later TOPMOST scaling window; restore our OWN overlay's
/// Z-order without clicking, focusing the game or injecting anything.
pub fn keep_horizon_overlay_above_magpie()->bool{
    let hwnd=own_window(HUD_TITLE);
    if hwnd.is_null()||unsafe{IsWindowVisible(hwnd)}==0{return false;}
    const SWP_NOSIZE:u32=0x0001;
    const SWP_NOMOVE:u32=0x0002;
    const SWP_NOACTIVATE:u32=0x0010;
    let topmost=(-1isize) as Handle;
    let updated=unsafe{SetWindowPos(hwnd,topmost,0,0,0,0,SWP_NOSIZE|SWP_NOMOVE|SWP_NOACTIVATE)};
    updated!=0
}

/// Set ONLY the Horizon HUD top-level window, never the game's HWND.
/// Passive mode uses a translucent layered WS_EX_TRANSPARENT + NOACTIVATE
/// surface; Windows passes game mouse hit tests through it. Switching to
/// the on-demand editor removes those flags so the controls can be clicked.
/// Do not use a global low-level mouse hook or tamper with game input.
pub fn show_passive_hud()->bool{
    let hwnd=own_window(HUD_TITLE);
    if hwnd.is_null(){return false;}
    let required=0x08080020;
    if unsafe{GetWindowLongW(hwnd,-20)}&required!=required{return false;}
    (unsafe{SetWindowPos(hwnd,(-1isize) as Handle,0,0,0,0,0x0001|0x0002|0x0010|0x0040)})!=0
}
pub fn configure_overlay_interaction(passive:bool,opacity:f32)->Result<(),String>{
 configure_named_overlay(HUD_TITLE,passive,opacity)
}
pub fn configure_named_overlay(title:&str,passive:bool,opacity:f32)->Result<(),String>{
    if title!=HUD_TITLE&&title!=MACRO_HUD_TITLE{return Err("Not an owned Vanta overlay title".into());}
    let hwnd=own_window(title);
    if hwnd.is_null(){return Err("Horizon HUD window does not exist yet".into());}
    const GWL_EXSTYLE:i32=-20;
    const WS_EX_LAYERED:i32=0x00080000;
    const WS_EX_TRANSPARENT:i32=0x00000020;
    const WS_EX_NOACTIVATE:i32=0x08000000;
    const WS_EX_TOOLWINDOW:i32=0x00000080;
    const LWA_ALPHA:u32=2;
    let old=unsafe{GetWindowLongW(hwnd,GWL_EXSTYLE)};
    let mut wanted=old;
    // Compact mode is intentionally mouse-through and does NOT take focus.
    if passive {
        wanted|=WS_EX_LAYERED|WS_EX_TRANSPARENT|WS_EX_NOACTIVATE|WS_EX_TOOLWINDOW;
    }else{
        wanted &= !(WS_EX_TRANSPARENT|WS_EX_NOACTIVATE|WS_EX_TOOLWINDOW);
        if opacity>=0.999{wanted &= !WS_EX_LAYERED;}else{wanted|=WS_EX_LAYERED;}
    }
    if wanted!=old{
        unsafe{SetWindowLongW(hwnd,GWL_EXSTYLE,wanted);}
        let actual=unsafe{GetWindowLongW(hwnd,GWL_EXSTYLE)};
        let mask=WS_EX_LAYERED|WS_EX_TRANSPARENT|WS_EX_NOACTIVATE|WS_EX_TOOLWINDOW;
        if (actual&mask)!=(wanted&mask){
            return Err("Windows refused HUD click-through window flags".into());
        }
    }
    if wanted&WS_EX_LAYERED!=0 {
        let alpha=if passive {opacity.min(0.86)}else{opacity}.clamp(0.72,1.0);
        let a=(alpha*255.0).round() as u8;
        if unsafe{SetLayeredWindowAttributes(hwnd,0,a,LWA_ALPHA)}==0{
            return Err("Windows refused HUD transparency".into());
        }
    }
    // Recompute native hit-test styles without activating/focusing the HUD.
    const SWP_NOSIZE:u32=0x0001;
    const SWP_NOMOVE:u32=0x0002;
    const SWP_NOACTIVATE:u32=0x0010;
    const SWP_FRAMECHANGED:u32=0x0020;
    let topmost=(-1isize) as Handle;
    if unsafe{SetWindowPos(hwnd,topmost,0,0,0,0,
        SWP_NOSIZE|SWP_NOMOVE|SWP_NOACTIVATE|if wanted!=old{SWP_FRAMECHANGED}else{0})}==0{
        return Err("Windows failed to refresh HUD hit-test styles".into());
    }
    Ok(())
}
pub fn set_overlay_opacity(opacity:f32)->Result<(),String>{
    // Only changes Horizon Boost's own overlay window, never the game.
    let hwnd=own_window(HUD_TITLE);
    if hwnd.is_null(){return Err("Overlay window is not created yet".into());}
    const GWL_EXSTYLE:i32=-20;
    const WS_EX_LAYERED:i32=0x00080000;
    const LWA_ALPHA:u32=0x00000002;
    let old=unsafe{GetWindowLongW(hwnd,GWL_EXSTYLE)};
    let alpha=(opacity.clamp(0.72,1.0)*255.0).round() as u8;
    // Fully opaque: avoid layered-window composition entirely. This makes
    // smooth DWM antialiasing possible again and saves a compositor surface.
    if alpha==255{
        if old&WS_EX_LAYERED!=0{
            unsafe{SetWindowLongW(hwnd,GWL_EXSTYLE,old&!WS_EX_LAYERED);}
        }
        return Ok(());
    }
    if old&WS_EX_LAYERED==0{
        unsafe{SetWindowLongW(hwnd,GWL_EXSTYLE,old|WS_EX_LAYERED);}
    }
    if unsafe{SetLayeredWindowAttributes(hwnd,0,alpha,LWA_ALPHA)}==0{
        return Err("Windows could not apply overlay opacity".into());
    }
    Ok(())
}
// Load Windows 11 DWM corner API at runtime. Linking dwmapi directly is not
// available in the project's x86_64-pc-windows-gnu build toolchain; loading
// the system DLL through kernel32 avoids hard build/runtime dependencies.
#[link(name="kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name:*const u16)->Handle;
    fn LoadLibraryW(name:*const u16)->Handle;
    fn GetProcAddress(module:Handle,name:*const u8)->*const std::ffi::c_void;
}
fn request_dwm_corners(hwnd:Handle,overlay:bool,maximized:bool)->bool{
    let dll=wstr("dwmapi.dll");
    let mut module=unsafe{GetModuleHandleW(dll.as_ptr())};
    if module.is_null(){module=unsafe{LoadLibraryW(dll.as_ptr())};}
    if module.is_null(){return false;}
    let ptr=unsafe{GetProcAddress(module,b"DwmSetWindowAttribute\0".as_ptr())};
    if ptr.is_null(){return false;}
    type Setter=unsafe extern "system" fn(Handle,u32,*const i32,u32)->i32;
    let f:Setter=unsafe{std::mem::transmute(ptr)};
    // Attribute 33: window corner preference. 1=do not round,
    // 2=rounded, 3=small round. Windows 10 returns a failed HRESULT.
    let preference=if maximized{1i32}else if overlay{3}else{2};
    let result=unsafe{f(hwnd,33,&preference,4)};
    result>=0
}
// Native round window clip and DWM corner policy. Never applies to game HWND.
#[link(name="gdi32")]
unsafe extern "system" {
    fn CreateRoundRectRgn(left:i32,top:i32,right:i32,bottom:i32,ellipse_w:i32,ellipse_h:i32)->Handle;
    fn DeleteObject(object:Handle)->i32;
}
#[link(name="user32")]
unsafe extern "system" {
    fn SetWindowRgn(hwnd:Handle,region:Handle,redraw:i32)->i32;
}
static MAIN_ROUND_STYLE:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
static OVERLAY_ROUND_STYLE:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
/// Native Windows 11 DWM round corners first (smooth antialiasing). GDI
/// region fallback only on unsupported OS. Never alters game window HWND.
pub fn round_horizon_window(overlay:bool)->bool{
    use std::sync::atomic::Ordering;
    let cache=if overlay{&OVERLAY_ROUND_STYLE}else{&MAIN_ROUND_STYLE};
    let name=if overlay{HUD_TITLE}else{MAIN_TITLE};
    let hwnd=own_window(name);
    if hwnd.is_null(){return false;}
    let mut boxrect=WinRect::default();
    if unsafe{GetWindowRect(hwnd,&mut boxrect)}==0{return false;}
    let w=boxrect.right-boxrect.left;
    let h=boxrect.bottom-boxrect.top;
    if w<=0||h<=0{return false;}
    let max=unsafe{IsZoomed(hwnd)}!=0;
    let layered=(unsafe{GetWindowLongW(hwnd,-20)} as u32&0x00080000)!=0;
    let marker=(hwnd as usize as u64).wrapping_mul(11400714819323198485)
        ^((w as u64)<<20)^((h as u64)<<5)^u64::from(max)
        ^((layered as u64)<<61);
    if cache.load(Ordering::Relaxed)==marker{return true;}
    // DWM cannot reliably round per-pixel or constant-alpha layered popup
    // windows. If opacity enabled, use explicit rounded clipping instead.
    // When fully opaque, clear old GDI region and let Win11 DWM antialias.
    let ok=if !layered&&request_dwm_corners(hwnd,overlay,max){
        (unsafe{SetWindowRgn(hwnd,std::ptr::null_mut(),1)})!=0
    }else if max{
        (unsafe{SetWindowRgn(hwnd,std::ptr::null_mut(),1)})!=0
    }else{
        let radius=if overlay{18}else{22};
        let region=unsafe{CreateRoundRectRgn(0,0,w+1,h+1,radius,radius)};
        if region.is_null(){return false;}
        let status=unsafe{SetWindowRgn(hwnd,region,1)};
        if status==0{unsafe{DeleteObject(region);}}
        status!=0
    };
    if ok{
        cache.store(marker,Ordering::Relaxed);
        true
    }else{false}
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn detects_game_paths(){
        assert!(is_probable_game(Path::new(r"C:\Steam\steamapps\common\MyGame\game.exe")));
        assert!(is_probable_game(Path::new(r"D:\Games\zelda tears of kingdom\totk.exe")));
        assert!(!is_probable_game(Path::new(r"D:\Games\zelda tears of kingdom\TOTKRecompModManager.exe")));
        assert!(!is_probable_game(Path::new(r"C:\Program Files\Google\Chrome\chrome.exe")));
    }
    #[test]fn gaming_shortcuts_are_official_settings_uris(){
        assert_eq!(WINDOWS_GAME_TOOLS.len(),4);
        assert!(WINDOWS_GAME_TOOLS.iter().all(|(_,url)|url.starts_with("ms-settings:")));
        assert!(WINDOWS_GAME_TOOLS.iter().all(|(_,url)|!url.contains(' ')));
    }
    #[test]fn filters_helper_windows(){
        assert!(allowed_game_window_title("TotK Recomp"));
        assert!(!allowed_game_window_title("TotK Recomp Mod Manager"));
        assert!(!allowed_game_window_title("Game Launcher"));
    }
}

#[cfg(test)]
mod magpie_window_contract_tests{
    use super::*;
    #[test] fn magpie_scaling_contract_is_stable(){
        assert_eq!(MAGPIE_SCALING_CLASS,
            "Window_Magpie_967EB565-6F73-4E94-AE53-00CC42592A22");
        assert!(magpie_scaling_info(0).is_none());
    }
}

fn select_active(pinned:Option<u32>,prior:Option<&GameInfo>,visible:&[GameInfo],foreground:Option<GameInfo>)->Option<GameInfo>{
 if let Some(pid)=pinned{return visible.iter().find(|g|g.pid==pid).cloned().or_else(||prior.filter(|g|g.pid==pid).cloned());}
 foreground.or_else(||prior.and_then(|p|visible.iter().find(|g|g.pid==p.pid).cloned())).or_else(||visible.first().cloned())
}
#[cfg(test)]mod pinned_selection_tests{use super::*;fn fixture(pid:u32)->GameInfo{GameInfo{pid,title:format!("Game {pid}"),path:PathBuf::from(format!("game{pid}.exe")),class_name:"fixture".into(),fullscreen_like:false}}
 #[test]fn minimized_pinned_game_cannot_fall_back_to_another_game(){let a=fixture(1);let b=fixture(2);assert_eq!(select_active(Some(1),Some(&a),&[b.clone()],Some(b.clone())).unwrap().pid,1);assert_eq!(select_active(None,Some(&a),&[b.clone()],Some(b.clone())).unwrap().pid,2);assert!(select_active(Some(1),None,&[b],None).is_none());}
}

pub fn place_own_hud(target:&crate::display_probe::Target,anchor:[f32;2])->Result<Option<u32>,String>{
 let hwnd=own_window(HUD_TITLE);
 if hwnd.is_null(){return Ok(None);}
 let mut r=WinRect::default();if unsafe{GetWindowRect(hwnd,&mut r)}==0{return Err("Own HUD geometry unavailable".into());}
 let w=r.right-r.left;let h=r.bottom-r.top;
 if w<=0||h<=0{return Ok(None);}
 let(x,y)=crate::hud_layout::physical_anchor(target,(w,h),anchor);
 if (r.left-x).abs()>2||(r.top-y).abs()>2{
  if unsafe{SetWindowPos(hwnd,std::ptr::null_mut(),x,y,0,0,0x0001|0x0004|0x0010)}==0{return Err("Own HUD placement failed".into());}
 }
 let dpi=unsafe{GetDpiForWindow(hwnd)};Ok((dpi>0).then_some(dpi))
}

#[cfg(test)]mod minecraft_detect_v125{
    use super::*;
    #[test]fn java_game_classification_is_specific_to_mc(){
        assert!(is_minecraft_java_window(Path::new("C:/jdk/bin/javaw.exe"),"Minecraft 1.21.1","GLFW30"));
        assert!(is_minecraft_java_window(Path::new("C:/runtime/bin/javaw.exe"),"Game window","GLFW30"));
        assert!(!is_minecraft_java_window(Path::new("C:/runtime/bin/javaw.exe"),"Java IDE","SunAwtFrame"));
        assert!(!is_minecraft_java_window(Path::new("C:/other.exe"),"Minecraft","GLFW30"));
        assert!(!is_minecraft_java_window(Path::new("C:/runtime/bin/javaw.exe"),"Prism Launcher","Qt"));
    }
    #[test]fn manual_window_excludes_system_and_self(){
        let sample=|name:&str|GameInfo{pid:100,title:"Window".into(),path:PathBuf::from(name),class_name:String::new(),fullscreen_like:false};
        assert!(!allowed_manual_window(&sample("C:/Windows/explorer.exe")));
        assert!(!allowed_manual_window(&sample("C:/Horizon/horizon-boost-rust.exe")));
        assert!(allowed_manual_window(&sample("C:/runtime/javaw.exe")));
    }
}

/// A read-only target-window probe. Does not initialize WGC, GPU or modify z-order.
#[derive(Clone,Debug)]
pub struct CapturePreflight{
 pub pid:u32,pub title:String,pub class_name:String,pub width:u32,pub height:u32,
 pub foreground:bool,pub monitor_filling:bool,pub minecraft_java:bool,
 pub dpi:u32,pub monitor_width:u32,pub monitor_height:u32
}
pub fn capture_preflight(pid:u32)->Result<CapturePreflight,String>{
 if pid==0||pid==std::process::id(){return Err("Select a real game process".into());}
 struct State{pid:u32,hwnd:Handle,rect:WinRect,area:i64}
 unsafe extern "system" fn cb(hwnd:Handle,context:isize)->i32{
  let out=unsafe{&mut *(context as *mut State)};
  if unsafe{IsWindowVisible(hwnd)}==0||unsafe{IsIconic(hwnd)}!=0||is_dwm_cloaked(hwnd)||!unsafe{GetWindow(hwnd,4)}.is_null(){return 1;}
  let mut owner=0;unsafe{GetWindowThreadProcessId(hwnd,&mut owner);}
  if owner!=out.pid{return 1;}
  let mut r=WinRect::default();
  if unsafe{GetWindowRect(hwnd,&mut r)}==0{return 1;}
  let w=(r.right-r.left).max(0)as i64;let h=(r.bottom-r.top).max(0)as i64;
  if w>=160&&h>=90&&w*h>out.area{out.hwnd=hwnd;out.area=w*h;out.rect=r;}
  1
 }
 let mut state=State{pid,hwnd:std::ptr::null_mut(),rect:WinRect::default(),area:0};
 unsafe{EnumWindows(cb,&mut state as *mut State as isize);}
 if state.hwnd.is_null(){return Err("Game has no suitable visible non-minimized window".into());}
 let g=unsafe{window_info(state.hwnd,false)}.ok_or("Cannot read selected game window")?;
 let rect=state.rect;
 let width=(rect.right-rect.left).max(0)as u32;
 let height=(rect.bottom-rect.top).max(0)as u32;
 let (sw,sh)=monitor_dimensions(state.hwnd).unwrap_or_else(screen_size);
 let dpi=unsafe{GetDpiForWindow(state.hwnd)}.clamp(72,384);
 let foreground=unsafe{
  let hwnd=GetForegroundWindow();let mut active=0;
  if !hwnd.is_null(){GetWindowThreadProcessId(hwnd,&mut active);}
  active==pid
 };
 Ok(CapturePreflight{pid,title:g.title.clone(),class_name:g.class_name.clone(),width,height,
  foreground,monitor_filling:width>=sw.saturating_sub(8)&&height>=sh.saturating_sub(8),
  minecraft_java:is_minecraft_java_window(&g.path,&g.title,&g.class_name),
  dpi,monitor_width:sw,monitor_height:sh})
}
#[cfg(test)]mod preflight_v126_tests{
 use super::*;
 #[test]fn pure_minecraft_window_classification_is_not_title_dependent(){
  assert!(is_minecraft_java_window(Path::new("D:/java/javaw.exe"),"Project: Dreamshift v0.1.3","GLFW30"));
  assert!(!is_minecraft_java_window(Path::new("D:/java/javaw.exe"),"Java launcher","QtLauncher"));
 }
 #[test]fn invalid_pid_is_rejected_without_starting_capture(){
  assert!(capture_preflight(0).is_err());
  assert!(capture_preflight(std::process::id()).is_err());
 }
}

#[cfg(test)] mod v138_hud_title_test {
 #[test]fn hud_title_matches_eframe_creation(){assert_eq!(super::HUD_TITLE,"Vanta FX - HUD");assert_eq!(super::MAIN_TITLE,"Vanta FX - Native GPU Tools");}
}

#[cfg(test)]mod v138_native_hud_style_smoke{
 use super::*;
 #[link(name="user32")]unsafe extern "system"{
  fn CreateWindowExW(ex:u32,class:*const u16,title:*const u16,style:u32,
    x:i32,y:i32,w:i32,h:i32,parent:Handle,menu:Handle,instance:Handle,
    param:*mut std::ffi::c_void)->Handle;
  fn DestroyWindow(hwnd:Handle)->i32;
 }
 #[test]fn hidden_native_hud_is_discoverable_and_can_be_made_mouse_through(){
  let class=wstr("STATIC");let title=wstr(HUD_TITLE);
  let hwnd=unsafe{CreateWindowExW(0,class.as_ptr(),title.as_ptr(),0,
   0,0,10,10,std::ptr::null_mut(),std::ptr::null_mut(),
   std::ptr::null_mut(),std::ptr::null_mut())};
  assert!(!hwnd.is_null(),"Failed to create a safe hidden HUD fixture");
  let found=own_window(HUD_TITLE);
  let styles=configure_overlay_interaction(true,0.86);
  let flag=unsafe{GetWindowLongW(hwnd,-20)}as u32;
  unsafe{DestroyWindow(hwnd);}
  assert_eq!(found,hwnd,"Native title look-up did not find real own HWND");
  assert!(styles.is_ok(),"Could not apply mouse-through styles: {styles:?}");
  assert_eq!(flag&0x08080020,0x08080020,"Window must be layered, click-through and nonactivating");
 }
}
