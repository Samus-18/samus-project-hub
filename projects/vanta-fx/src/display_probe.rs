//! Read-only Windows display topology; no display settings changed.
use std::mem::size_of;
#[repr(C)]
struct Device { cb:u32,name:[u16;32],description:[u16;128],flags:u32,id:[u16;128],key:[u16;128] }
#[repr(C)]
struct Mode { name:[u16;32],spec:u16,driver:u16,size:u16,extra:u16,fields:u32,
 position:[i32;4],color:i16,duplex:i16,yres:i16,tt:i16,collate:i16,form:[u16;32],logpixels:u16,
 bits:u32,width:u32,height:u32,flags:u32,frequency:u32,icm:u32,intent:u32,media:u32,dither:u32,
 reserved1:u32,reserved2:u32,panwidth:u32,panheight:u32 }
#[link(name="user32")]
unsafe extern "system" {
 fn EnumDisplayDevicesW(device:*const u16,index:u32,out:*mut Device,flags:u32)->i32;
 fn EnumDisplaySettingsW(device:*const u16,index:u32,out:*mut Mode)->i32;
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Display {pub name:String,pub adapter:String,pub primary:bool,pub x:Option<i32>,pub y:Option<i32>,pub width:Option<u32>,pub height:Option<u32>,pub refresh:Option<u32>,pub bits:Option<u32>}
fn wide(s:&[u16])->String{String::from_utf16_lossy(&s[..s.iter().position(|v|*v==0).unwrap_or(s.len())])}
fn valid(v:u32,lo:u32,hi:u32)->Option<u32>{(lo..=hi).contains(&v).then_some(v)}
fn mode_row(d:&Device,m:Option<&Mode>)->Display{
 let field=|mask:u32|m.filter(|v|v.fields&mask!=0);
 Display{name:wide(&d.name),adapter:wide(&d.description),primary:d.flags&4!=0,
 x:field(0x20).map(|v|v.position[0]),y:field(0x20).map(|v|v.position[1]),
 width:field(0x80000).and_then(|v|valid(v.width,1,32768)),height:field(0x100000).and_then(|v|valid(v.height,1,32768)),
 refresh:field(0x400000).and_then(|v|valid(v.frequency,2,1000)),bits:field(0x40000).and_then(|v|valid(v.bits,1,128))}
}
pub fn read()->Vec<Display>{
 let mut out=Vec::new();
 for i in 0..32{
  let mut d:Device=unsafe{std::mem::zeroed()};d.cb=size_of::<Device>() as u32;
  if unsafe{EnumDisplayDevicesW(std::ptr::null(),i,&mut d,0)}==0{break;}
  if d.flags&1==0||d.flags&8!=0{continue;}
  let mut m:Mode=unsafe{std::mem::zeroed()};m.size=size_of::<Mode>() as u16;
  let mode=if unsafe{EnumDisplaySettingsW(d.name.as_ptr(),u32::MAX,&mut m)}!=0{Some(&m)}else{None};
  out.push(mode_row(&d,mode));
 }
 out.sort_by_key(|d|(!d.primary,d.name.clone()));out
}
pub fn summary()->String{
 let rows=read();let mut s=format!("DISPLAY_PROBE={} COUNT={} NOMINAL_ONLY=true VRR=UNKNOWN",if rows.is_empty(){"UNAVAILABLE"}else{"PASS"},rows.len());
 for d in rows{s.push_str(&format!("\n{} primary={} mode={:?}x{:?} refresh={:?} pos={:?},{:?} bits={:?} adapter={}",d.name,d.primary,d.width,d.height,d.refresh,d.x,d.y,d.bits,d.adapter));}s
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn windows_abi_sizes(){assert_eq!(size_of::<Device>(),840);assert_eq!(size_of::<Mode>(),220);}
 #[test]fn missing_or_default_refresh_is_not_invented(){let d:Device=unsafe{std::mem::zeroed()};let mut m:Mode=unsafe{std::mem::zeroed()};m.frequency=60;assert_eq!(mode_row(&d,Some(&m)).refresh,None);m.fields=0x400000;m.frequency=1;assert_eq!(mode_row(&d,Some(&m)).refresh,None);m.frequency=144;assert_eq!(mode_row(&d,Some(&m)).refresh,Some(144));assert_eq!(mode_row(&d,None).refresh,None);}
 #[test]fn topology_keeps_signed_coordinates(){let d:Device=unsafe{std::mem::zeroed()};let mut m:Mode=unsafe{std::mem::zeroed()};m.fields=0x20|0x80000|0x100000;m.position[0]=-1920;m.width=1920;m.height=1080;let v=mode_row(&d,Some(&m));assert_eq!(v.x,Some(-1920));assert_eq!(v.width,Some(1920));}
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Target {pub name:String,pub x:i32,pub y:i32,pub width:u32,pub height:u32,pub refresh:Option<u32>}
impl Display{
 pub fn target(&self)->Option<Target>{Some(Target{name:self.name.clone(),x:self.x?,y:self.y?,width:self.width?,height:self.height?,refresh:self.refresh})}
}
pub fn select(rows:&[Display],requested:Option<&str>)->Option<Target>{
 requested.and_then(|n|rows.iter().find(|r|r.name==n).and_then(Display::target))
  .or_else(||rows.iter().find(|r|r.primary).and_then(Display::target))
  .or_else(||rows.iter().find_map(Display::target))
}
impl Target{
 // Fit a normal preview within the selected display, preserving aspect ratio.
 pub fn window_rect(&self,w:u32,h:u32)->(i32,i32,i32,i32){
  let max_w=self.width.saturating_sub(80).max(1).min(1200);
  let max_h=self.height.saturating_sub(100).max(1).min(790);
  let scale=(max_w as f64/w.max(1) as f64).min(max_h as f64/h.max(1) as f64).min(1.);
  let w=(w as f64*scale).round().max(1.) as i32;let h=(h as f64*scale).round().max(1.) as i32;
  (self.x.saturating_add((self.width as i32-w).max(0)/2),self.y.saturating_add((self.height as i32-h).max(0)/2),w,h)
 }
}
#[derive(Default)]pub struct Watch{candidate:Option<Option<Target>>,count:u8}
impl Watch{
 // Two consecutive matching observations avoid restarting on a transient query failure.
 pub fn changed(&mut self,current:Option<&Target>,observed:Option<Target>)->bool{
  if current==observed.as_ref(){self.candidate=None;self.count=0;return false;}
  if self.candidate.as_ref()==Some(&observed){self.count=self.count.saturating_add(1);}
  else{self.candidate=Some(observed);self.count=1;}
  self.count>=2
 }
}
#[cfg(test)]mod placement_tests{
 use super::*;
 fn monitor(name:&str,primary:bool,x:i32,w:u32,h:u32)->Display{Display{name:name.into(),adapter:"test".into(),primary,x:Some(x),y:Some(0),width:Some(w),height:Some(h),refresh:Some(75),bits:Some(32)}}
 #[test]fn second_monitor_and_fallback(){let rows=[monitor("primary",true,0,1920,1080),monitor("secondary",false,-1280,1280,720)];let t=select(&rows,Some("secondary")).unwrap();assert_eq!(t.name,"secondary");let(x,y,w,h)=t.window_rect(1920,1080);assert!(x>=-1280&&x+w<=0&&y>=0&&y+h<=720);assert_eq!(select(&rows,Some("missing")).unwrap().name,"primary");assert!(select(&[],None).is_none());}
 #[test]fn watch_ignores_single_transient(){let a=monitor("a",true,0,1920,1080).target().unwrap();let b=monitor("b",true,-1280,1280,720).target().unwrap();let mut w=Watch::default();assert!(!w.changed(Some(&a),None));assert!(!w.changed(Some(&a),Some(a.clone())));assert!(!w.changed(Some(&a),Some(b.clone())));assert!(w.changed(Some(&a),Some(b)));}
}
