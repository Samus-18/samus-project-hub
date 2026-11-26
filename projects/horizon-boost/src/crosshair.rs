//! Optional click-through native Windows crosshair overlay.
//! A separate layered GDI topmost HWND. No game process memory, DLL or graphics hook.
//! Supported in ordinary desktop/borderless modes; not guaranteed exclusive fullscreen.
use serde::{Deserialize,Serialize};
use std::{ffi::c_void,ptr,thread,sync::{Arc,Mutex,OnceLock,atomic::{AtomicBool,AtomicU32,Ordering}}};
type H=*mut c_void;
#[repr(C)]#[derive(Clone,Copy,Default)]struct Rect{l:i32,t:i32,r:i32,b:i32}
#[repr(C)]#[derive(Clone,Copy,Default)]struct Point{x:i32,y:i32}
#[repr(C)]struct WndClass{
    style:u32,wnd_proc:Option<unsafe extern "system" fn(H,u32,usize,isize)->isize>,
    cls_extra:i32,wnd_extra:i32,instance:H,icon:H,cursor:H,brush:H,
    menu_name:*const u16,class_name:*const u16,
}
#[repr(C)]#[derive(Default)]struct Paint{
    dc:H,erase:i32,area:Rect,restore:i32,update:i32,reserved:[u8;32],
}
#[repr(C)]#[derive(Default)]struct Msg{hwnd:H,message:u32,wparam:usize,lparam:isize,time:u32,pt:Point,private:u32}
#[link(name="user32")]
unsafe extern "system"{
    fn RegisterClassW(class:*const WndClass)->u16;
    fn CreateWindowExW(ex:u32,class:*const u16,name:*const u16,style:u32,
        x:i32,y:i32,w:i32,h:i32,parent:H,menu:H,instance:H,param:H)->H;
    fn DefWindowProcW(hwnd:H,message:u32,wparam:usize,lparam:isize)->isize;
    fn DestroyWindow(hwnd:H)->i32;
    fn ShowWindow(hwnd:H,cmd:i32)->i32;
    fn SetWindowPos(hwnd:H,insert_after:H,x:i32,y:i32,w:i32,h:i32,flags:u32)->i32;
    fn SetLayeredWindowAttributes(hwnd:H,key:u32,alpha:u8,flags:u32)->i32;
    fn BeginPaint(hwnd:H,paint:*mut Paint)->H;
    fn FillRect(dc:H,rect:*const Rect,brush:H)->i32;
    fn EndPaint(hwnd:H,paint:*const Paint)->i32;
    fn SetTimer(hwnd:H,id:usize,millis:u32,proc:usize)->usize;
    fn KillTimer(hwnd:H,id:usize)->i32;
    fn GetMessageW(msg:*mut Msg,hwnd:H,min:u32,max:u32)->i32;
    fn TranslateMessage(msg:*const Msg)->i32;
    fn DispatchMessageW(msg:*const Msg)->isize;
    fn PostQuitMessage(code:i32);
    fn InvalidateRect(hwnd:H,rect:*const Rect,erase:i32)->i32;
    fn GetSystemMetrics(index:i32)->i32;
    fn GetForegroundWindow()->H;
    fn FindWindowW(class:*const u16,name:*const u16)->H;
    fn GetWindowThreadProcessId(hwnd:H,pid:*mut u32)->u32;
    fn GetWindowRect(hwnd:H,rect:*mut Rect)->i32;
    fn GetClientRect(hwnd:H,rect:*mut Rect)->i32;
    fn ClientToScreen(hwnd:H,point:*mut Point)->i32;
    fn IsWindowVisible(hwnd:H)->i32;
}
#[link(name="kernel32")]
unsafe extern "system" { fn GetModuleHandleW(name:*const u16)->H; }
#[link(name="gdi32")]
unsafe extern "system"{
    fn CreateSolidBrush(color:u32)->H;
    fn CreatePen(style:i32,width:i32,color:u32)->H;
    fn SelectObject(dc:H,obj:H)->H;
    fn DeleteObject(obj:H)->i32;
    fn GetStockObject(index:i32)->H;
    fn Ellipse(dc:H,left:i32,top:i32,right:i32,bottom:i32)->i32;
    fn MoveToEx(dc:H,x:i32,y:i32,old:*mut Point)->i32;
    fn LineTo(dc:H,x:i32,y:i32)->i32;
}
thread_local!{static RETICLE_CACHE:std::cell::RefCell<crate::reticle_update::State>=Default::default();}
const SIDE:i32=196;
const KEY_COLOR:u32=0x00040203; // BGR for translucent color-keyed background
const EXSTYLE:u32=0x00080000|0x00000020|0x00000080|0x00000008|0x08000000;
// LAYERED | TRANSPARENT | TOOLWINDOW | TOPMOST | NOACTIVATE
const WM_PAINT:u32=0x0F;
const WM_DESTROY:u32=0x02;
const WM_TIMER:u32=0x113;
fn wide(text:&str)->Vec<u16>{text.encode_utf16().chain(Some(0)).collect()}
fn color_rgb(r:u8,g:u8,b:u8)->u32{r as u32|((g as u32)<<8)|((b as u32)<<16)}
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq)]
#[serde(default)]
pub struct CrosshairOptions{
    pub enabled:bool,
    pub style:usize, // 0 dot, 1 gap cross, 2 circle, 3 circle+dot, 4 T
    pub color:usize, // 0 white, 1 scarlet, 2 lime, 3 cyan, 4 gold
    pub size:i32,
    pub thickness:i32,
    pub gap:i32,
    pub outline:bool,
    pub opacity:u8,
}
impl Default for CrosshairOptions{
    fn default()->Self{Self{enabled:false,style:1,color:1,size:18,thickness:2,gap:5,outline:false,opacity:255}}
}
impl CrosshairOptions{
    pub fn clamp(&mut self){
        self.style=self.style.min(STYLE_LABELS.len()-1);self.color=self.color.min(4);
        self.opacity=self.opacity.clamp(70,255);
        self.size=self.size.clamp(3,68);
        self.thickness=self.thickness.clamp(1,6);
        self.gap=self.gap.clamp(0,20);
    }
    pub fn color_value(&self)->u32{
        match self.color{
            1=>color_rgb(255,62,86),2=>color_rgb(108,252,137),
            3=>color_rgb(74,212,252),4=>color_rgb(255,203,87),
            _=>color_rgb(255,255,255)
        }
    }
}
pub const STYLE_LABELS:&[&str]=&[
    "Dot","Cross + gap","Ring","Ring + dot","T-reticle",
    "Scope","Corner brackets","Chevron","Diagonal X","Dual rings",
    "Aim grid","Micro precision","Classic T",
    "Bracket Scope","Precision Diamond","Split Chevron","Rangefinder",
    "Radar Target","Fine Dot + Marks","Halo Target",
];
#[derive(Debug,PartialEq,Eq)]
pub struct ReticleGeometry{
    pub dot:i32,
    pub rings:Vec<i32>,
    pub lines:Vec<([i32;2],[i32;2])>,
}
pub fn geometry(style:usize,size:i32,gap:i32)->ReticleGeometry{
    let s=size.clamp(3,68);
    let g=gap.clamp(0,20).min(s.saturating_sub(1));
    let mut result=ReticleGeometry{dot:0,rings:vec![],lines:vec![]};
    let mut add=|x:i32,y:i32,x2:i32,y2:i32|{
        result.lines.push(([x,y],[x2,y2]));
    };
    match style{
        0=>result.dot=(s/3).clamp(1,8),
        1=>{add(-s,0,-g,0);add(g,0,s,0);add(0,-s,0,-g);add(0,g,0,s);},
        2=>result.rings.push(s),
        3=>{result.dot=3;result.rings.push(s);},
        4=>{add(-s,0,-g,0);add(g,0,s,0);add(0,g,0,s);},
        5=>{
            result.rings.push(s);
            add(-s-7,0,-g,0);add(g,0,s+7,0);
            add(0,-s-7,0,-g);add(0,g,0,s+7);
        },
        6=>{
            let c=s/3;
            for (sx,sy) in [(-1,-1),(1,-1),(-1,1),(1,1)]{
                let x=sx*s;let y=sy*s;
                add(x,y,x-sx*c,y);
                add(x,y,x,y-sy*c);
            }
        },
        7=>{result.dot=2;add(-s,-s/2,0,-s);add(0,-s,s,-s/2);add(0,g,0,s);},
        8=>{add(-s,-s,-g,-g);add(g,g,s,s);add(-s,s,-g,g);add(g,-g,s,-s);},
        9=>{result.rings.push(s);result.rings.push((s/2).max(5));},
        10=>{
            for x in [-s/3,s/3]{add(x,-s,x,s);}
            for y in [-s/3,s/3]{add(-s,y,s,y);}
            result.dot=2;
        },
        11=>{
            result.dot=2;
            let q=(s/2).max(7);let g=g.min(q-1);
            add(-q,0,-g,0);add(g,0,q,0);
            add(0,-q,0,-g);add(0,g,0,q);
        },
        12=>{result.dot=2;add(-s,-s/2,s,-s/2);add(0,-s/2,0,s);},
        13=>{
            for (sx,sy) in [(-1,-1),(1,-1),(-1,1),(1,1)]{
                add(sx*s,sy*s,sx*s,sy*(s-7));
                add(sx*s,sy*s,sx*(s-7),sy*s);
            }
            result.dot=2;
        },
        14=>{
            add(0,-s,s,0);add(s,0,0,s);add(0,s,-s,0);add(-s,0,0,-s);
            result.dot=2;
        },
        15=>{
            add(-s,-s/2,0,-g);add(0,-g,s,-s/2);
            add(-s,s/2,0,g);add(0,g,s,s/2);
        },
        16=>{
            result.rings.push((s*2/3).max(4));
            add(-s,0,-g,0);add(g,0,s,0);
            add(0,-s,0,-g);add(0,g,0,s);
            add(-s,-s/3,-s+5,-s/3);add(s-5,s/3,s,s/3);
        },
        17=>{
            result.rings.push(s);
            result.rings.push((s/3).max(3));
            add(-s-4,0,-s/2,0);add(s/2,0,s+4,0);
        },
        18=>{
            result.dot=1;
            let q=(s/3).max(4);
            add(-s,0,-q,0);add(q,0,s,0);
            add(0,-s,0,-q);add(0,q,0,s);
        },
        19=>{
            result.rings.push(s);
            add(-s-6,0,-s+5,0);add(s-5,0,s+6,0);
            add(0,-s-6,0,-s+5);add(0,s-5,0,s+6);
            result.dot=2;
        },
        _=>{}
    }
    result
}
struct Shared{options:Mutex<CrosshairOptions>,running:AtomicBool,target_pid:AtomicU32,visible:AtomicBool}
static INSTANCE:OnceLock<Arc<Shared>>=OnceLock::new();
pub struct CrosshairHud{shared:Arc<Shared>,worker:Option<thread::JoinHandle<()>>}
impl CrosshairHud{
    pub fn new()->Self{
        Self{shared:INSTANCE.get_or_init(||Arc::new(Shared{
            options:Mutex::new(CrosshairOptions::default()),running:AtomicBool::new(true),
            target_pid:AtomicU32::new(0),visible:AtomicBool::new(false)
        })).clone(),worker:None}
    }
    pub fn hwnd_exists()->bool{
        let cls=wide("HorizonBoostCrosshairHud");
        !unsafe{FindWindowW(cls.as_ptr(),ptr::null())}.is_null()
    }
    pub fn visible(&self)->bool{self.shared.visible.load(Ordering::Acquire)}
    pub fn set_target(&self,pid:Option<u32>){
        self.shared.target_pid.store(pid.unwrap_or(0),Ordering::Relaxed);
    }
    pub fn set(&mut self,settings:&CrosshairOptions){
        let mut valid=settings.clone();
        valid.clamp();
        if let Ok(mut lock)=self.shared.options.lock(){*lock=valid.clone();}
        if !valid.enabled{
            self.shared.running.store(false,Ordering::Release);
            return;
        }
        if self.worker.as_ref().is_some_and(|w|w.is_finished()||!self.shared.running.load(Ordering::Acquire)){
            if let Some(t)=self.worker.take(){let _=t.join();}
        }
        if self.worker.is_none(){
            self.shared.running.store(true,Ordering::Release);
            let shared=self.shared.clone();
            self.worker=Some(thread::spawn(move||{unsafe{run_window()};shared.running.store(false,Ordering::Release);}));
        }
    }
    pub fn stop(&mut self){
        if let Ok(mut opt)=self.shared.options.lock(){opt.enabled=false;}
        self.shared.running.store(false,Ordering::Relaxed);
        if let Some(t)=self.worker.take(){let _=t.join();}
    }
}
impl Drop for CrosshairHud{fn drop(&mut self){self.stop();}}
fn options()->CrosshairOptions{
    INSTANCE.get().and_then(|s|s.options.lock().ok().map(|g|g.clone()))
        .unwrap_or_default()
}
unsafe fn paint_shape(dc:H,style:usize,size:i32,gap:i32,width:i32,color:u32){
    let shape=geometry(style,size,gap);
    let pen=unsafe{CreatePen(0,width,color)};
    if pen.is_null(){return;}
    let previous=unsafe{SelectObject(dc,pen)};
    let previous_brush=unsafe{SelectObject(dc,GetStockObject(5))}; // NULL_BRUSH: rings must not erase underlying reticle lines
    let center=SIDE/2;
    for (a,b) in shape.lines {
        unsafe{MoveToEx(dc,center+a[0],center+a[1],ptr::null_mut());
            LineTo(dc,center+b[0],center+b[1]);}
    }
    for radius in shape.rings {
        unsafe{Ellipse(dc,center-radius,center-radius,center+radius,center+radius);}
    }
    if shape.dot>0{
        let brush=unsafe{CreateSolidBrush(color)};
        let old=unsafe{SelectObject(dc,brush)};
        let radius=shape.dot+if width>3{1}else{0};
        unsafe{Ellipse(dc,center-radius,center-radius,center+radius+1,center+radius+1);
            SelectObject(dc,old);DeleteObject(brush);}
    }
    unsafe{SelectObject(dc,previous_brush);SelectObject(dc,previous);DeleteObject(pen);}
}
unsafe fn draw(hwnd:H){
    let mut ps=Paint::default();
    let dc=unsafe{BeginPaint(hwnd,&mut ps)};
    if dc.is_null(){return;}
    let base=unsafe{CreateSolidBrush(KEY_COLOR)};
    let old_brush=unsafe{SelectObject(dc,base)};
    let bounds=Rect{l:0,t:0,r:SIDE,b:SIDE};
    unsafe{FillRect(dc,&bounds,base);}
    let opts=options();
    if opts.outline{
        unsafe{paint_shape(dc,opts.style,opts.size,opts.gap,opts.thickness+3,
            color_rgb(2,2,6));}
    }
    unsafe{paint_shape(dc,opts.style,opts.size,opts.gap,opts.thickness,
        opts.color_value());}
    unsafe{SelectObject(dc,old_brush);DeleteObject(base);EndPaint(hwnd,&ps);}
}
unsafe extern "system" fn window_proc(hwnd:H,msg:u32,w:usize,l:isize)->isize{
    match msg{
        WM_PAINT=>{unsafe{draw(hwnd)};0},
        WM_TIMER=>{
            let Some(shared)=INSTANCE.get()else{return 0;};
            if !shared.running.load(Ordering::Relaxed){
                shared.visible.store(false,Ordering::Release);
                unsafe{DestroyWindow(hwnd);}
                return 0;
            }
            let opt=options();
            let pid=shared.target_pid.load(Ordering::Relaxed);
            let foreground=unsafe{GetForegroundWindow()};
            let mut actual=0u32;
            unsafe{GetWindowThreadProcessId(foreground,&mut actual);}
            let mut window=Rect::default();let mut origin=Point::default();
            let valid=pid!=0&&actual==pid&&unsafe{GetClientRect(foreground,&mut window)}!=0&&unsafe{ClientToScreen(foreground,&mut origin)}!=0&&window.r>window.l&&window.b>window.t;
            RETICLE_CACHE.with(|cell|{
                let mut cache=cell.borrow_mut();
                if opt.enabled&&valid{
                    let x=origin.x+(window.r-window.l-SIDE)/2;let y=origin.y+(window.b-window.t-SIDE)/2;
                    let was_visible=unsafe{IsWindowVisible(hwnd)}!=0;
                    let next=cache.observe((x,y),&opt,was_visible,std::time::Instant::now());
                    if next.alpha{unsafe{SetLayeredWindowAttributes(hwnd,KEY_COLOR,opt.opacity,3);}}
                    if next.position{unsafe{SetWindowPos(hwnd,(-1isize) as H,x,y,SIDE,SIDE,0x0010|0x0040);}}
                    if next.paint{unsafe{InvalidateRect(hwnd,ptr::null(),1);}}
                    shared.visible.store(unsafe{IsWindowVisible(hwnd)}!=0,Ordering::Release);
                }else{
                    if unsafe{IsWindowVisible(hwnd)}!=0{unsafe{ShowWindow(hwnd,0);}}
                    cache.clear();shared.visible.store(false,Ordering::Release);
                }
            });
            0
        },
        WM_DESTROY=>{unsafe{KillTimer(hwnd,1);PostQuitMessage(0);}0},
        _=>unsafe{DefWindowProcW(hwnd,msg,w,l)}
    }
}
unsafe fn run_window(){
    let cls=wide("HorizonBoostCrosshairHud");
    let title=wide("Horizon Boost crosshair HUD");
    let module=unsafe{GetModuleHandleW(ptr::null())};
    let desc=WndClass{style:0,wnd_proc:Some(window_proc),cls_extra:0,wnd_extra:0,
        instance:module,icon:ptr::null_mut(),cursor:ptr::null_mut(),
        brush:ptr::null_mut(),menu_name:ptr::null(),class_name:cls.as_ptr()};
    unsafe{RegisterClassW(&desc);}
    let hwnd=unsafe{CreateWindowExW(EXSTYLE,cls.as_ptr(),title.as_ptr(),0x80000000,
        (GetSystemMetrics(0)-SIDE)/2,(GetSystemMetrics(1)-SIDE)/2,
        SIDE,SIDE,ptr::null_mut(),ptr::null_mut(),module,ptr::null_mut())};
    if hwnd.is_null(){return;}
    unsafe{
        SetLayeredWindowAttributes(hwnd,KEY_COLOR,255,1); // color-key transparency
        // Do not show the HWND until the selected game really has focus.
        SetWindowPos(hwnd,(-1isize) as H,0,0,0,0,0x0001|0x0002|0x0010);
        if SetTimer(hwnd,1,100,0)==0{DestroyWindow(hwnd);return;}
    }
    let mut msg=Msg::default();
    while unsafe{GetMessageW(&mut msg,ptr::null_mut(),0,0)}>0{
        unsafe{TranslateMessage(&msg);DispatchMessageW(&msg);}
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn colors_and_shapes_clamp_safely(){
        let mut opt=CrosshairOptions{enabled:false,style:99,color:99,size:1000,thickness:0,gap:-9,outline:true,opacity:0};
        opt.clamp();
        assert_eq!(opt.style,19);assert_eq!(opt.color,4);assert_eq!(opt.opacity,70);
        assert_eq!(opt.size,68);assert_eq!(opt.thickness,1);assert_eq!(opt.gap,0);
        assert_eq!(color_rgb(255,0,0),0xff);
    }
    #[test]fn all_crosshair_styles_have_real_drawing_geometry(){
        assert_eq!(STYLE_LABELS.len(),20);
        for style in 0..STYLE_LABELS.len(){
            let shape=geometry(style,20,5);
            assert!(shape.dot>0||!shape.lines.is_empty()||!shape.rings.is_empty());
            assert_eq!(geometry(style,20,5),shape);
        }
    }
    #[test]fn crosshair_targets_only_a_selected_game(){
        let hud=CrosshairHud::new();
        hud.set_target(None);
        assert_eq!(hud.shared.target_pid.load(Ordering::Relaxed),0);
        hud.set_target(Some(1245));
        assert_eq!(hud.shared.target_pid.load(Ordering::Relaxed),1245);
        // No window spawned: option defaults disabled, no effect on desktop.
        assert!(hud.worker.is_none());
    }
    #[test]fn default_crosshair_does_not_show(){
        assert!(!CrosshairOptions::default().enabled);
    }
}

#[cfg(test)]mod v88_geometry{use super::*;#[test]fn small_cross_and_micro_never_reverse_segments(){for style in [1,11]{let g=geometry(style,3,20);for(a,b)in g.lines{assert_ne!(a,b);if a[1]==b[1]{assert!(a[0]<b[0]);}else{assert!(a[1]<b[1]);}}}let t=geometry(4,18,5);assert!(t.lines.contains(&([-18,0],[-5,0])));assert!(t.lines.contains(&([5,0],[18,0])));}}

#[cfg(test)]mod v88_gdi_test{
 use super::*;
 #[repr(C)]struct BitmapHeader{size:u32,width:i32,height:i32,planes:u16,bits:u16,compression:u32,image_size:u32,x:i32,y:i32,used:u32,important:u32}
 #[link(name="gdi32")]unsafe extern "system"{fn CreateCompatibleDC(h:H)->H;fn DeleteDC(h:H)->i32;fn CreateDIBSection(dc:H,info:*const BitmapHeader,usage:u32,bits:*mut *mut c_void,section:H,offset:u32)->H;fn GdiFlush()->i32;}
 #[test]fn actual_gdi_reticle_does_not_fill_ring(){unsafe{
 let dc=CreateCompatibleDC(ptr::null_mut());assert!(!dc.is_null());let head=BitmapHeader{size:40,width:SIDE,height:-SIDE,planes:1,bits:32,compression:0,image_size:0,x:0,y:0,used:0,important:0};let mut bits=ptr::null_mut();let bmp=CreateDIBSection(dc,&head,0,&mut bits,ptr::null_mut(),0);assert!(!bmp.is_null()&&!bits.is_null());
 let old=SelectObject(dc,bmp);let data=std::slice::from_raw_parts_mut(bits.cast::<u32>(),(SIDE*SIDE)as usize);data.fill(0);paint_shape(dc,3,20,5,2,color_rgb(255,0,0));GdiFlush();
 let center=SIDE/2;let pixel=|x:i32,y:i32|data[(y*SIDE+x)as usize]&0xffffff;
 assert_eq!(pixel(center,center),0xff0000);assert_eq!(pixel(center+10,center),0);assert_eq!(pixel(center+19,center),0xff0000);
 SelectObject(dc,old);DeleteObject(bmp);DeleteDC(dc);
 }}
}
