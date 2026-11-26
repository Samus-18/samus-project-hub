//! Explicit selected-window recording and playback, ordinary desktop / offline use.
use serde::{Serialize,Deserialize};
use std::{collections::{BTreeMap,BTreeSet},ffi::c_void,sync::{Arc,atomic::{AtomicBool,AtomicUsize,Ordering},mpsc::{self,Receiver}},thread,time::{Duration,Instant}};
const MAX:usize=512;const LIMIT:u32=60000;
const KEYS:&[u16]=&[0x20,0x0d,0x09,0x25,0x26,0x27,0x28,0x30,0x31,0x32,0x33,0x34,0x35,0x36,0x37,0x38,0x39,0x41,0x42,0x43,0x44,0x45,0x46,0x47,0x48,0x49,0x4a,0x4b,0x4c,0x4d,0x4e,0x4f,0x50,0x51,0x52,0x53,0x54,0x55,0x56,0x57,0x58,0x59,0x5a,0x1b,0x8,0x2d,0x2e,0x24,0x23,0x21,0x22,0x70,0x71,0x72,0x73,0x74,0x75,0x76,0x77,0x78,0x79,0x7a];
#[derive(Clone,Copy,Debug,Serialize,Deserialize,PartialEq,Eq)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Action{Move{x:u16,y:u16},Button{button:u8,down:bool},Key{code:u16,down:bool}}
impl Action{pub fn label(self)->String{match self{
 Self::Move{x,y}=>format!("Pointer {:.1}% / {:.1}%",x as f32/100.,y as f32/100.),
 Self::Button{button,down}=>format!("{} {}",if button==0{"Left click"}else{"Right click"},if down{"down"}else{"up"}),
 Self::Key{code,down}=>format!("{} {}",key_name(code),if down{"down"}else{"up"})}}}
fn key_name(k:u16)->String{match k{0x20=>"Space".into(),0x0d=>"Enter".into(),0x09=>"Tab".into(),0x25=>"Left".into(),0x26=>"Up".into(),0x27=>"Right".into(),0x28=>"Down".into(),0x1b=>"Esc".into(),0x08=>"Backspace".into(),
 0x2d=>"Insert".into(),0x2e=>"Delete".into(),0x24=>"Home".into(),0x23=>"End".into(),
 0x21=>"PageUp".into(),0x22=>"PageDown".into(),
 0x70..=0x7a=>format!("F{}",k-0x6f),0x30..=0x39|0x41..=0x5a=>(k as u8 as char).to_string(),_=>"?".into()}}
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
#[serde(deny_unknown_fields)]pub struct Event{pub wait_ms:u32,pub action:Action}
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
#[serde(default,deny_unknown_fields)]pub struct Plan{pub events:Vec<Event>,pub repeat:u8,pub speed:u16,pub between_ms:u32}
impl Default for Plan{fn default()->Self{Self{events:vec![],repeat:1,speed:100,between_ms:500}}}
impl Plan{
 /// Shift the first event in place; all edits are transactional and preserve valid plans.
 pub fn shift_start(&mut self,delta:i32)->Result<(),String>{
  self.validate()?;
  let mut p=self.clone();
  let first=p.events.first_mut().ok_or("No macro events")?;
  first.wait_ms=if delta>=0{first.wait_ms.checked_add(delta as u32).ok_or("Wait time overflow")?}
    else{first.wait_ms.saturating_sub(delta.unsigned_abs())};
  p.validate()?;*self=p;Ok(())
 }
 /// Rescale recorded waits without changing pointer positions or held-key transitions.
 pub fn scale_waits(&mut self,percent:u32)->Result<(),String>{
  self.validate()?;if !(50..=200).contains(&percent){return Err("Timing scale must be 50..200%".into());}
  let mut p=self.clone();
  for e in &mut p.events{let scaled=(u64::from(e.wait_ms)*u64::from(percent)+50)/100;
   e.wait_ms=u32::try_from(scaled).map_err(|_|"Timing overflow")?;}
  p.validate()?;*self=p;Ok(())
 }
 /// Horizontal/vertical path mirror; no input is sent until the user starts playback.
 pub fn mirror_pointer(&mut self,horizontal:bool,vertical:bool)->Result<(),String>{
  self.validate()?;if !horizontal&&!vertical{return Ok(());}
  let mut p=self.clone();
  for e in &mut p.events{if let Action::Move{x,y}=&mut e.action{
   if horizontal{*x=10000u16.saturating_sub(*x);}
   if vertical{*y=10000u16.saturating_sub(*y);}
  }}
  p.validate()?;*self=p;Ok(())
 }
 /// Append a second balanced cycle of the recorded event sequence.
 pub fn double_sequence(&mut self)->Result<(),String>{
  self.validate()?;
  let mut p=self.clone();
  if p.events.len()>MAX/2{return Err("Macro exceeds event capacity".into());}
  p.events.extend_from_slice(&self.events);
  p.validate()?;*self=p;Ok(())
 }
 /// Remove only immediately duplicate zero-wait pointer events. A drag path is never collapsed.
 pub fn compact_exact_duplicates(&mut self)->Result<usize,String>{
  self.validate()?;let mut p=self.clone();let before=p.events.len();
  let mut prev=None;
  p.events.retain(|e|{
   let a=match e.action{Action::Move{x,y}=>Some((x,y)),_=>None};
   let keep=!(a.is_some()&&a==prev&&e.wait_ms==0);
   if keep{prev=a;}keep
  });
  p.validate()?;let n=before-p.events.len();*self=p;Ok(n)
 }
 pub fn delay(&self,ms:u32)->u64{(u64::from(ms)*100).div_ceil(u64::from(self.speed.max(1)))}
 pub fn duration(&self)->u64{self.single_duration().saturating_mul(u64::from(self.repeat))+u64::from(self.repeat.saturating_sub(1))*u64::from(self.between_ms)}
 pub fn validate(&self)->Result<(),String>{
  if self.events.is_empty()||self.events.len()>MAX{return Err("1–512 events required".into());}
  if !(1..=10).contains(&self.repeat)||!(50..=200).contains(&self.speed)||self.between_ms>10000||self.events.iter().map(|e|u64::from(e.wait_ms)).sum::<u64>()>u64::from(LIMIT)||self.duration()>180000{return Err("Recording / repeat / timing exceeds limits".into());}
  let(mut keys,mut buttons)=(BTreeSet::new(),BTreeSet::new());let mut positioned=false;
  for e in &self.events{match e.action{
   Action::Move{x,y} if x<=10000&&y<=10000=>positioned=true,
   Action::Button{button,down} if button<=1&&positioned=>{if !(if down{buttons.insert(button)}else{buttons.remove(&button)}){return Err("Unbalanced mouse sequence".into());}},
   Action::Key{code,down} if KEYS.contains(&code)=>{if !(if down{keys.insert(code)}else{keys.remove(&code)}){return Err("Unbalanced key sequence".into());}},
   _=>return Err("Unsupported action / missing mouse position".into()),
  }}
  if !keys.is_empty()||!buttons.is_empty(){return Err("Release all inputs before saving or playback".into());}Ok(())
 }
 pub fn append_key(&mut self,key:&str,wait:u32,hold:u32)->Result<(),String>{
  let code=crate::game_macros::key_code(key).filter(|v|KEYS.contains(v)).ok_or("Unsupported key")?;
  let mut p=self.clone();p.events.extend([Event{wait_ms:wait,action:Action::Key{code,down:true}},Event{wait_ms:hold,action:Action::Key{code,down:false}}]);p.validate()?;*self=p;Ok(())
 }
 pub fn append_double_click(&mut self,x:u16,y:u16,button:u8,wait:u32,hold:u32,gap:u32)->Result<(),String>{
  if !(50..=500).contains(&gap)||!(10..=400).contains(&hold){return Err("Double click: hold 10..400 ms and gap 50..500 ms".into());}
  let mut p=self.clone();
  p.append_click(x,y,button,wait,hold)?;
  p.append_click(x,y,button,gap,hold)?;
  p.validate()?;*self=p;Ok(())
 }
 pub fn translate_pointer(&mut self,dx:i16,dy:i16)->Result<(),String>{
  self.validate()?;
  let mut p=self.clone();
  for e in &mut p.events{
   if let Action::Move{x,y}=&mut e.action{
    *x=(i32::from(*x)+i32::from(dx)).clamp(0,10000) as u16;
    *y=(i32::from(*y)+i32::from(dy)).clamp(0,10000) as u16;
   }
  }
  p.validate()?;*self=p;Ok(())
 }
 pub fn append_click(&mut self,x:u16,y:u16,button:u8,wait:u32,hold:u32)->Result<(),String>{
  let mut p=self.clone();p.events.extend([Event{wait_ms:wait,action:Action::Move{x,y}},Event{wait_ms:0,action:Action::Button{button,down:true}},Event{wait_ms:hold,action:Action::Button{button,down:false}}]);p.validate()?;*self=p;Ok(())
 }
 pub fn delete_preserve_time(&mut self,i:usize)->bool{
  if i>=self.events.len(){return false;}let e=self.events.remove(i);if let Some(next)=self.events.get_mut(i){next.wait_ms=next.wait_ms.saturating_add(e.wait_ms);}true
 }
 pub fn set_wait(&mut self,i:usize,wait:u32)->Result<(),String>{let mut candidate=self.clone();candidate.events.get_mut(i).ok_or("Event not found")?.wait_ms=wait;candidate.validate()?;*self=candidate;Ok(())}
 pub fn elapsed_at(&self,i:usize)->Option<u64>{self.events.get(i)?;Some(self.events[..=i].iter().map(|e|self.delay(e.wait_ms)).sum())}
 pub fn paired_index(&self,i:usize)->Option<usize>{
  let action=self.events.get(i)?.action;
  let matches=|a:Action|match(action,a){(Action::Key{code:a,down:x},Action::Key{code:b,down:y})=>a==b&&x!=y,(Action::Button{button:a,down:x},Action::Button{button:b,down:y})=>a==b&&x!=y,_=>false};
  match action{Action::Move{..}=>None,Action::Key{down:true,..}|Action::Button{down:true,..}=>self.events.iter().enumerate().skip(i+1).find(|(_,e)|matches(e.action)).map(|(j,_)|j),_=>self.events[..i].iter().enumerate().rev().find(|(_,e)|matches(e.action)).map(|(j,_)|j)}
 }
 pub fn remove_balanced(&mut self,i:usize)->Result<(),String>{
  self.validate()?;let action=self.events.get(i).ok_or("Event not found")?.action;
  let mut candidate=self.clone();
  if matches!(action,Action::Move{..}){candidate.delete_preserve_time(i);}else{let j=self.paired_index(i).ok_or("Matching release missing")?;candidate.delete_preserve_time(i.max(j));candidate.delete_preserve_time(i.min(j));}
  if !candidate.events.is_empty(){candidate.validate()?;}*self=candidate;Ok(())
 }
 pub fn swap_balanced(&mut self,a:usize,b:usize)->Result<(),String>{
  if a>=self.events.len()||b>=self.events.len(){return Err("Event not found".into());}
  let mut p=self.clone();p.events.swap(a,b);p.validate()?;*self=p;Ok(())
 }
 pub fn single_duration(&self)->u64{self.events.iter().map(|e|self.delay(e.wait_ms)).sum()}
 pub fn export(&self)->Result<String,String>{self.validate()?;serde_json::to_string_pretty(&serde_json::json!({"format":"HorizonBoost.WindowMacro.1","plan":self})).map_err(|e|e.to_string())}
 pub fn import(s:&str)->Result<Self,String>{
  if s.len()>65536{return Err("Macro exceeds 64 KiB".into());}
  #[derive(Deserialize)]#[serde(deny_unknown_fields)]struct Doc{format:String,plan:Plan}
  let d:Doc=serde_json::from_str(s).map_err(|e|e.to_string())?;if d.format!="HorizonBoost.WindowMacro.1"{return Err("Unknown macro format".into());}d.plan.validate()?;Ok(d.plan)
 }
 pub fn save(&self,name:&str,bank:&mut BTreeMap<String,Plan>)->Result<(),String>{self.validate()?;let n=name.trim();if n.is_empty()||n.chars().count()>48||n.chars().any(char::is_control)||(!bank.contains_key(n)&&bank.len()>=32){return Err("Name 1–48 characters, at most 32 macros".into());}bank.insert(n.into(),self.clone());Ok(())}
}
pub fn sanitize_profiles(profiles:&mut BTreeMap<String,BTreeMap<String,Plan>>){profiles.retain(|k,bank|{sanitize(bank);!k.trim().is_empty()&&k.len()<=512&&!k.chars().any(char::is_control)&&!bank.is_empty()});while profiles.len()>128{profiles.pop_last();}}
pub fn sanitize(bank:&mut BTreeMap<String,Plan>){bank.retain(|k,v|!k.trim().is_empty()&&k.chars().count()<=48&&!k.chars().any(char::is_control)&&v.validate().is_ok());while bank.len()>32{bank.pop_last();}}
#[derive(Default)]struct Record{events:Vec<Event>,last:u32,keys:BTreeSet<u16>,buttons:BTreeSet<u8>}
impl Record{
 fn push(&mut self,at:u32,a:Action)->bool{
  if self.events.len()>=MAX-KEYS.len()-2||at>LIMIT||at<self.last{return false;}
  match a{Action::Key{code,down}=>{if down{self.keys.insert(code);}else{self.keys.remove(&code);}},Action::Button{button,down}=>{if down{self.buttons.insert(button);}else{self.buttons.remove(&button);}},_=>{}}
  self.events.push(Event{wait_ms:at-self.last,action:a});self.last=at;true
 }
 fn finish(mut self)->Plan{for code in self.keys{self.events.push(Event{wait_ms:0,action:Action::Key{code,down:false}});}for button in self.buttons{self.events.push(Event{wait_ms:0,action:Action::Button{button,down:false}});}Plan{events:self.events,..Default::default()}}
}
#[derive(Default)]pub struct Preview{pub point:Option<[u16;2]>,pub keys:BTreeSet<u16>,pub buttons:BTreeSet<u8>,pub applied:usize,pub cycle:u8}
pub fn preview(p:&Plan,ms:u64)->Preview{
 let mut v=Preview::default();if p.validate().is_err(){return v;}let mut t=0;
 for i in 0..p.repeat{v.cycle=i+1;for e in &p.events{t+=p.delay(e.wait_ms);if ms<t{return v;}match e.action{Action::Move{x,y}=>v.point=Some([x,y]),Action::Key{code,down}=>{if down{v.keys.insert(code);}else{v.keys.remove(&code);}},Action::Button{button,down}=>{if down{v.buttons.insert(button);}else{v.buttons.remove(&button);}}}v.applied+=1;}t+=u64::from(p.between_ms);if ms<t{return v;}}v
}
type Hwnd=*mut c_void;
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Target{pub hwnd:usize,pub pid:u32,pub tid:u32,pub title:String}
#[repr(C)]#[derive(Clone,Copy,Default)]struct Point{x:i32,y:i32}
#[repr(C)]#[derive(Default)]struct Rect{left:i32,top:i32,right:i32,bottom:i32}
#[link(name="user32")]unsafe extern "system"{
 fn EnumWindows(cb:unsafe extern "system" fn(Hwnd,isize)->i32,data:isize)->i32;
 fn IsWindowVisible(hwnd:Hwnd)->i32;fn GetWindowTextW(hwnd:Hwnd,b:*mut u16,n:i32)->i32;
 fn GetWindowThreadProcessId(hwnd:Hwnd,pid:*mut u32)->u32;fn GetForegroundWindow()->Hwnd;
 fn GetClientRect(hwnd:Hwnd,rect:*mut Rect)->i32;fn ClientToScreen(hwnd:Hwnd,p:*mut Point)->i32;fn ScreenToClient(hwnd:Hwnd,p:*mut Point)->i32;
 fn GetCursorPos(p:*mut Point)->i32;fn SetCursorPos(x:i32,y:i32)->i32;fn GetAsyncKeyState(vkey:i32)->i16;
 fn WindowFromPoint(p:Point)->Hwnd;fn GetAncestor(hwnd:Hwnd,flags:u32)->Hwnd;
}
pub fn targets()->Vec<Target>{
 unsafe extern "system" fn cb(hwnd:Hwnd,data:isize)->i32{
  let rows=unsafe{&mut*(data as *mut Vec<Target>)};if rows.len()>=100{return 0;}
  if unsafe{IsWindowVisible(hwnd)}==0{return 1;}let mut pid=0;let tid=unsafe{GetWindowThreadProcessId(hwnd,&mut pid)};if pid==0||tid==0||pid==std::process::id(){return 1;}
  let mut b=[0u16;256];let n=unsafe{GetWindowTextW(hwnd,b.as_mut_ptr(),256)};if n<=0{return 1;}
  rows.push(Target{hwnd:hwnd as usize,pid,tid,title:String::from_utf16_lossy(&b[..n as usize])});1
 }
 let mut rows:Vec<Target>=Vec::new();unsafe{EnumWindows(cb,&mut rows as *mut _ as isize);}rows.sort_by_cached_key(|t|t.title.to_lowercase());rows
}
impl Target{
 fn identity(&self)->bool{let mut pid=0;let tid=unsafe{GetWindowThreadProcessId(self.hwnd as Hwnd,&mut pid)};self.pid!=0&&self.pid!=std::process::id()&&pid==self.pid&&tid==self.tid}
 fn focused(&self)->bool{self.identity()&&unsafe{GetForegroundWindow()} as usize==self.hwnd}
 fn size(&self)->Option<(i32,i32)>{let mut r=Rect::default();if unsafe{GetClientRect(self.hwnd as Hwnd,&mut r)}==0{return None;}let w=r.right-r.left;let h=r.bottom-r.top;(w>1&&h>1).then_some((w,h))}
 fn owns(&self,p:Point)->bool{let h=unsafe{WindowFromPoint(p)};!h.is_null()&&unsafe{GetAncestor(h,2)} as usize==self.hwnd}
 fn pointer(&self)->Option<[u16;2]>{let mut p=Point::default();if unsafe{GetCursorPos(&mut p)}==0||!self.owns(p){return None;}if unsafe{ScreenToClient(self.hwnd as Hwnd,&mut p)}==0{return None;}let(w,h)=self.size()?;if p.x<0||p.y<0||p.x>=w||p.y>=h{return None;}Some([normalize(p.x,w),normalize(p.y,h)])}
 fn origin(&self)->Option<(i32,i32)>{let mut p=Point::default();(unsafe{ClientToScreen(self.hwnd as Hwnd,&mut p)}!=0).then_some((p.x,p.y))}
 fn absolute(&self,x:u16,y:u16)->Option<Point>{let(w,h)=self.size()?;let mut p=Point{x:denormalize(x,w),y:denormalize(y,h)};if unsafe{ClientToScreen(self.hwnd as Hwnd,&mut p)}==0||!self.owns(p){return None;}Some(p)}
}
fn normalize(x:i32,s:i32)->u16{((i64::from(x.clamp(0,s.saturating_sub(1).max(0)))*10000)/i64::from(s.saturating_sub(1).max(1)))as u16}
fn denormalize(x:u16,s:i32)->i32{((i64::from(x.min(10000))*i64::from(s.saturating_sub(1).max(0))+5000)/10000)as i32}
fn down(k:u16)->bool{(unsafe{GetAsyncKeyState(k as i32)} as u16&0x8000)!=0}
fn cancelled(run:&AtomicBool)->bool{!run.load(Ordering::Acquire)||down(0x7b)}
fn modifiers()->bool{[0x10,0x11,0x12,0x5b,0x5c].into_iter().any(down)}
fn guard(t:&Target,run:&AtomicBool)->Result<(),String>{if cancelled(run){return Err("Cancelled".into());}if !t.focused(){return Err("Target lost focus or closed".into());}if modifiers(){return Err("Modifier pressed; stopped".into());}Ok(())}
fn wait_focus(t:&Target,run:&AtomicBool)->Result<(),String>{
 let end=Instant::now()+Duration::from_secs(20);
 while !t.focused(){if cancelled(run)||!t.identity()||Instant::now()>=end{return Err("Target not focused / cancelled".into());}thread::sleep(Duration::from_millis(25));}
 let end=Instant::now()+Duration::from_secs(3);while KEYS.iter().copied().chain([1,2,0x10,0x11,0x12,0x5b,0x5c]).any(down){if cancelled(run)||!t.focused()||Instant::now()>=end{return Err("Release inputs before starting".into());}thread::sleep(Duration::from_millis(15));}guard(t,run)
}
fn capture(t:&Target,run:&AtomicBool,progress:&AtomicUsize)->Result<Plan,String>{
 wait_focus(t,run)?;let start=Instant::now();let mut r=Record::default();let mut prev=None;
 loop{let at=start.elapsed().as_millis().min(u128::from(u32::MAX))as u32;if at>=LIMIT||guard(t,run).is_err(){break;}
  let p=t.pointer();let keys=KEYS.iter().copied().filter(|&k|down(k)).collect::<BTreeSet<_>>();
  let buttons=[(0,1),(1,2)].into_iter().filter_map(|(b,k)|down(k).then_some(b)).collect::<BTreeSet<u8>>();
  if guard(t,run).is_err(){break;}
  if p.is_none()&&(!buttons.is_empty()||!r.buttons.is_empty()){break;}
  if let Some([x,y])=p{if prev!=p{if !r.push(at,Action::Move{x,y}){break;}prev=p;}}
  let changes=r.keys.symmetric_difference(&keys).copied().map(|code|Action::Key{code,down:keys.contains(&code)}).chain(r.buttons.symmetric_difference(&buttons).copied().map(|button|Action::Button{button,down:buttons.contains(&button)})).collect::<Vec<_>>();
  let mut full=false;for a in changes{if !r.push(at,a){full=true;break;}}
  progress.store(r.events.len(),Ordering::Relaxed);if full{break;}thread::sleep(Duration::from_millis(16));
 }
 let p=r.finish();p.validate()?;Ok(p)
}
#[derive(Default)]struct Held{keys:BTreeSet<u16>,buttons:BTreeSet<u8>}
impl Held{
 fn emit(&mut self,a:Action)->Result<(),String>{match a{
 Action::Key{code,down}=>{crate::game_macros::emit_key(code,!down)?;if down{self.keys.insert(code);}else{self.keys.remove(&code);}},
 Action::Button{button,down}=>{crate::game_macros::emit_mouse(button,!down)?;if down{self.buttons.insert(button);}else{self.buttons.remove(&button);}},
 _=>return Err("Pointer handled separately".into()),}Ok(())}
 fn conflict(&self)->bool{KEYS.iter().copied().any(|k|!self.keys.contains(&k)&&down(k))||[(0,1),(1,2)].into_iter().any(|(b,k)|!self.buttons.contains(&b)&&down(k))}
}
impl Drop for Held{fn drop(&mut self){for &k in &self.keys{let _=crate::game_macros::emit_key(k,true);}for &b in &self.buttons{let _=crate::game_macros::emit_mouse(b,true);}}}
fn playback(t:&Target,p:&Plan,run:&AtomicBool,progress:&AtomicUsize)->Result<(),String>{
 p.validate()?;wait_focus(t,run)?;let geometry=(t.origin().ok_or("Target origin missing")?,t.size().ok_or("Target size missing")?);let mut held=Held::default();let mut expected=None;let mut n=0;
 let check=|held:&Held,expected:Option<[u16;2]>|->Result<(),String>{guard(t,run)?;if t.origin()!=Some(geometry.0)||t.size()!=Some(geometry.1){return Err("Window geometry changed; stopped".into());}if held.conflict(){return Err("Manual input detected".into());}if let Some(p)=expected{let q=t.pointer().ok_or("Pointer left target")?;if p[0].abs_diff(q[0])>180||p[1].abs_diff(q[1])>180{return Err("Pointer moved / target resized".into());}}Ok(())};
 let mut pause=PauseGate::default();
 let mut wait=|ms:u64,h:&Held,e:Option<[u16;2]>|->Result<(),String>{let mut end=Instant::now()+Duration::from_millis(ms);loop{check(h,e)?;
 let safe=h.keys.is_empty()&&h.buttons.is_empty();
 if pause.sample(down(0x13),safe){
  let start=Instant::now();while pause.paused{check(h,e)?;if start.elapsed()>=Duration::from_secs(30){return Err("Pause timeout; playback stopped".into());}thread::sleep(Duration::from_millis(20));pause.sample(down(0x13),true);}
  end+=start.elapsed();
 }
 let rem=end.saturating_duration_since(Instant::now());if rem.is_zero(){return Ok(());}thread::sleep(rem.min(Duration::from_millis(10)));}};
 for cycle in 0..p.repeat{if cycle>0{wait(u64::from(p.between_ms),&held,expected)?;}for e in &p.events{
  wait(p.delay(e.wait_ms),&held,expected)?;match e.action{
   Action::Move{x,y}=>{let pt=t.absolute(x,y).ok_or("Target point unavailable")?;guard(t,run)?;if unsafe{SetCursorPos(pt.x,pt.y)}==0{return Err("Pointer move refused".into());}expected=t.pointer();if expected.is_none(){return Err("Pointer outside selected window".into());}},
   Action::Button{..}=>{if expected.is_none()||t.pointer().is_none(){return Err("Mouse target unavailable".into());}check(&held,expected)?;held.emit(e.action)?;},
   Action::Key{..}=>{check(&held,expected)?;held.emit(e.action)?;},
  }n+=1;progress.store(n,Ordering::Relaxed);
 }}Ok(())
}
enum Finished{Recorded(Plan),Played}
struct Worker{cancel:Arc<AtomicBool>,progress:Arc<AtomicUsize>,rx:Receiver<Result<Finished,String>>,record:bool,thread:Option<thread::JoinHandle<()>>}
#[derive(Default)]pub struct History{past:Vec<Plan>,future:Vec<Plan>}
impl History{
 pub fn record(&mut self,p:Plan){if self.past.last()==Some(&p){return;}if self.past.len()>=32{self.past.remove(0);}self.past.push(p);self.future.clear();}
 pub fn undo(&mut self,p:&mut Plan)->bool{if let Some(v)=self.past.pop(){self.future.push(std::mem::replace(p,v));true}else{false}}
 pub fn redo(&mut self,p:&mut Plan)->bool{if let Some(v)=self.future.pop(){self.past.push(std::mem::replace(p,v));true}else{false}}
 pub fn can_undo(&self)->bool{!self.past.is_empty()}pub fn can_redo(&self)->bool{!self.future.is_empty()}
}
#[derive(Default)]pub struct Studio{pub windows:Vec<Target>,pub selected:Option<Target>,pub plan:Plan,pub name:String,pub raw:String,pub status:String,pub consent:bool,pub preview_at:Option<Instant>,pub history:History,worker:Option<Worker>}
impl Studio{
 pub fn busy(&self)->bool{self.worker.is_some()}
 pub fn progress(&self)->usize{self.worker.as_ref().map_or(0,|w|w.progress.load(Ordering::Relaxed))}
 pub fn refresh(&mut self){if !self.busy(){self.windows=targets();self.selected=self.selected.take().filter(|s|self.windows.iter().any(|t|t.hwnd==s.hwnd&&t.pid==s.pid&&t.tid==s.tid));self.consent=false;}}
 pub fn start(&mut self,record:bool)->Result<(),String>{
  if self.busy()||!self.consent{return Err("Select target and confirm permitted use".into());}
  let t=self.selected.clone().filter(Target::identity).ok_or("Select a currently available window")?;
  if crate::anticheat_guard::inspect(std::path::Path::new(""),&t.title).blocked{
   return Err("Protected multiplayer/anti-cheat title: macro recording and playback are disabled. This is a heuristic, not certification.".into());
  }
  let p=self.plan.clone();if !record{p.validate()?;}
  let cancel=Arc::new(AtomicBool::new(true));let progress=Arc::new(AtomicUsize::new(0));let(a,b)=(cancel.clone(),progress.clone());let(tx,rx)=mpsc::channel();
  let worker=thread::Builder::new().name("Horizon window macro".into()).spawn(move||{let result=if record{capture(&t,&a,&b).map(Finished::Recorded)}else{playback(&t,&p,&a,&b).map(|_|Finished::Played)};let _=tx.send(result);}).map_err(|e|e.to_string())?;
  self.consent=false;self.preview_at=None;self.worker=Some(Worker{cancel,progress,rx,record,thread:Some(worker)});self.status="Focus selected window within 20 seconds. F12 stops.".into();Ok(())
 }
 pub fn cancel(&mut self){self.preview_at=None;if let Some(w)=&self.worker{w.cancel.store(false,Ordering::Release);self.status="Stopping...".into();;}}
 pub fn poll(&mut self){let Some(w)=&self.worker else{return;};let result=match w.rx.try_recv(){Ok(v)=>v,Err(mpsc::TryRecvError::Empty)=>return,Err(_)=>Err("Worker disconnected".into())};if let Some(mut done)=self.worker.take(){if let Some(t)=done.thread.take(){let _=t.join();}}match result{Ok(Finished::Recorded(p))=>{self.status=format!("Recorded {} events (not saved)",p.events.len());self.history.record(self.plan.clone());self.plan=p;},Ok(Finished::Played)=>self.status="Playback complete".into(),Err(e)=>self.status=e}}
 pub fn mode(&self)->&'static str{self.worker.as_ref().map_or("Idle",|w|if w.record{"Recording"}else{"Playing"})}
}
impl Drop for Studio{fn drop(&mut self){self.cancel();if let Some(mut w)=self.worker.take(){if let Some(t)=w.thread.take(){let _=t.join();}}}}
pub fn proof()->Result<String,String>{
 let mut r=Record::default();r.push(100,Action::Move{x:2500,y:7500});r.push(150,Action::Button{button:0,down:true});r.push(200,Action::Key{code:65,down:true});r.push(300,Action::Move{x:9000,y:1000});let mut p=r.finish();p.repeat=2;p.speed=200;p.between_ms=500;p.validate()?;
 if Plan::import(&p.export()?)?!=p||p.duration()!=800{return Err("Roundtrip/timing".into());}
 let v=preview(&p,100);if !v.keys.contains(&65)||!v.buttons.contains(&0)||v.point!=Some([2500,7500]){return Err("Timeline".into());}
 let v=preview(&p,800);if !v.keys.is_empty()||!v.buttons.is_empty()||v.applied!=12{return Err("Release/repeat".into());}
 let mut bank=BTreeMap::new();p.save("Fixture",&mut bank)?;if bank.get("Fixture")!=Some(&p){return Err("Library".into());}
 Ok("WINDOW_MACRO=PASS ROUNDTRIP=true REPEAT=true PREVIEW=true BALANCED=true NO_REAL_INPUT=true".into())
}

// V137: opt-in, target-bound, background Alt+Fx macro trigger.
// It is independent from eframe's paint loop and therefore keeps listening
// while Vanta FX is minimized. No DLL injection, network, or keylogging.
struct GameHotkey{
 active:Arc<AtomicBool>,join:Option<thread::JoinHandle<()>>,
 status:Arc<std::sync::Mutex<String>>,
}
static GAME_HOTKEY:std::sync::OnceLock<std::sync::Mutex<Option<GameHotkey>>>=std::sync::OnceLock::new();
fn hotkey_slot()->&'static std::sync::Mutex<Option<GameHotkey>>{
 GAME_HOTKEY.get_or_init(||std::sync::Mutex::new(None))
}
pub fn hotkey_active()->bool{
 hotkey_slot().lock().is_ok_and(|s|s.as_ref().is_some_and(|h|h.active.load(Ordering::Acquire)))
}
pub fn hotkey_status()->String{
 hotkey_slot().lock().ok().and_then(|s|s.as_ref().map(|h|{
  h.status.lock().map(|v|v.clone()).unwrap_or_else(|_|"Status unavailable".into())
 })).unwrap_or_else(||"Disarmed".into())
}
pub fn hotkey_disarm(){
 if let Ok(mut guard)=hotkey_slot().lock(){
  if let Some(mut prev)=guard.take(){
   prev.active.store(false,Ordering::Release);
   if let Some(t)=prev.join.take(){let _=t.join();}
  }
 }
}
pub fn hotkey_arm(target:Target,plan:Plan,chord:crate::shortcut_registry::Chord)->Result<(),String>{
 plan.validate()?;
 if !chord.valid()||chord.mods!=1||!(0x70..=0x7a).contains(&chord.key)||chord.key==0x73{
  return Err("Choose Alt+F1..F11 except Alt+F4. F12 remains emergency stop.".into());
 }
 if !target.identity(){return Err("Selected window is no longer available".into());}
 if crate::anticheat_guard::inspect(std::path::Path::new(""),&target.title).blocked{
  return Err("Protected game title: macro not armed.".into());
 }
 hotkey_disarm();
 let active=Arc::new(AtomicBool::new(true));
 let status=Arc::new(std::sync::Mutex::new(format!("ARMED: {} / {}",
  chord.label(),target.title)));
 let run=active.clone();let result=status.clone();
 let worker=thread::Builder::new().name("Vanta FX offline macro hotkey".into()).spawn(move||{
  let progress=AtomicUsize::new(0);
  let mut was_down=chord.down();
  while run.load(Ordering::Acquire){
   if down(0x7b){run.store(false,Ordering::Release);break;} // F12
   if !target.identity(){run.store(false,Ordering::Release);break;}
   let pressed=chord.down();
   if pressed&&!was_down&&target.focused(){
    if let Ok(mut s)=result.lock(){*s="Playing in selected game...".into();}
    let playback_result=playback(&target,&plan,&run,&progress);
    if let Ok(mut s)=result.lock(){*s=match playback_result{
     Ok(())=>format!("Finished {} macro actions. Shortcut remains armed.",progress.load(Ordering::Relaxed)),
     Err(e)=>format!("Stopped: {e}. Shortcut remains armed."),
    };}
    if down(0x7b){run.store(false,Ordering::Release);break;}
   }
   was_down=pressed;
   thread::sleep(Duration::from_millis(16));
  }
 });
 match worker{
  Ok(join)=>{
   if let Ok(mut guard)=hotkey_slot().lock(){*guard=Some(GameHotkey{active,join:Some(join),status});Ok(())}
   else{active.store(false,Ordering::Release);let _=join.join();Err("Hotkey state lock unavailable".into())}
  },
  Err(e)=>{active.store(false,Ordering::Release);Err(e.to_string())},
 }
}
#[cfg(test)]mod v137_hotkey_tests{
 use super::*;
 #[test]fn safe_chord_definition_and_target_validation_are_explicit(){
  use crate::shortcut_registry::Chord;
  assert!(Chord{key:0x77,mods:1}.valid());
  assert!(!Chord{key:0x73,mods:1}.valid());
  assert!(!Chord{key:0x7b,mods:1}.valid());
 }
}

#[cfg(test)]mod tests{use super::*;
 #[test]fn deterministic_fixture(){proof().unwrap();}
 #[test]fn invalid_actions_and_timing(){let mut p=Plan{events:vec![Event{wait_ms:0,action:Action::Button{button:0,down:true}}],..Default::default()};assert!(p.validate().is_err());p.events=vec![Event{wait_ms:1,action:Action::Key{code:65,down:true}},Event{wait_ms:1,action:Action::Key{code:65,down:false}}];assert!(p.validate().is_ok());p.repeat=255;assert!(p.validate().is_err());assert!(Plan::import(r#"{"format":"HorizonBoost.WindowMacro.1","plan":{"events":[],"command":"run"}}"#).is_err());}
 #[test]fn coordinate_bounds(){assert_eq!(normalize(0,1920),0);assert_eq!(normalize(1919,1920),10000);assert_eq!(denormalize(10000,1920),1919);assert_eq!(denormalize(5000,1),0);}
 #[test]fn capped_record_closes_held_input(){let mut r=Record::default();r.push(1,Action::Key{code:65,down:true});for _ in 0..MAX{r.push(2,Action::Move{x:0,y:0});}let p=r.finish();assert!(p.events.len()<=MAX&&p.validate().is_ok());}
}

#[cfg(test)]mod editor_tests{use super::*;#[test]fn manual_builder_history_and_delete(){let mut p=Plan::default();let mut h=History::default();h.record(p.clone());p.append_key("E",100,200).unwrap();let prior=p.clone();h.record(prior.clone());p.append_click(10000,0,0,100,50).unwrap();assert_eq!(p.events.len(),5);assert!(h.undo(&mut p));assert_eq!(p,prior);assert!(h.redo(&mut p));assert_eq!(p.duration(),450);assert!(p.delete_preserve_time(2));assert!(p.validate().is_err());assert!(p.append_key("ALT+F4",0,0).is_err());}}

#[cfg(test)]mod balanced_editor_tests{use super::*;
 #[test]fn pair_remove_preserves_other_event_time(){let mut p=Plan::default();p.append_key("A",100,50).unwrap();p.append_key("B",200,70).unwrap();assert_eq!(p.paired_index(0),Some(1));let end=p.elapsed_at(3);p.remove_balanced(1).unwrap();assert_eq!(p.events.len(),2);assert_eq!(p.elapsed_at(1),end);assert!(p.validate().is_ok());}
 #[test]fn invalid_reorder_leaves_original_untouched(){let mut p=Plan::default();p.append_key("E",10,50).unwrap();let old=p.clone();assert!(p.swap_balanced(0,1).is_err());assert_eq!(p,old);p.remove_balanced(0).unwrap();assert!(p.events.is_empty());}
 #[test]fn mouse_click_requires_position_after_delete(){let mut p=Plan::default();p.append_click(0,0,0,50,100).unwrap();let old=p.clone();assert!(p.remove_balanced(0).is_err());assert_eq!(p,old);p.remove_balanced(2).unwrap();assert_eq!(p.events.len(),1);}
}

#[cfg(test)]mod v88_tests{use super::*;#[test]fn wait_edit_is_transactional_and_preserves_valid_plan(){let mut p=Plan::default();p.append_key("E",100,200).unwrap();let before=p.clone();assert!(p.set_wait(0,60000).is_err());assert_eq!(p,before);assert!(p.set_wait(3,5).is_err());p.set_wait(1,400).unwrap();assert_eq!(p.single_duration(),500);assert!(p.validate().is_ok());}}

#[cfg(test)]mod v88_profile_tests{use super::*;#[test]fn per_game_macro_library_isolated_and_legacy_preserved(){let mut s=crate::settings::Settings::default();let mut a=Plan::default();a.append_key("E",100,50).unwrap();let mut b=Plan::default();b.append_key("Q",50,50).unwrap();a.save("Same",&mut s.window_macros).unwrap();a.save("Same",s.window_macro_profiles.entry("game-a".into()).or_default()).unwrap();b.save("Same",s.window_macro_profiles.entry("game-b".into()).or_default()).unwrap();let decoded:crate::settings::Settings=serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();assert_eq!(decoded.window_macro_profiles["game-a"]["Same"],a);assert_eq!(decoded.window_macro_profiles["game-b"]["Same"],b);assert_eq!(decoded.window_macros["Same"],a);let old:crate::settings::Settings=serde_json::from_str("{}").unwrap();assert!(old.window_macro_profiles.is_empty());}}

#[cfg(test)]mod v101_timeline_tools{
 use super::*;
 fn fixture()->Plan{let mut p=Plan::default();p.append_click(1000,3000,0,200,100).unwrap();p}
 #[test]fn mirror_is_involution_and_preserves_buttons(){
  let mut p=fixture();let before=p.clone();
  p.mirror_pointer(true,true).unwrap();assert_eq!(p.events[0].action,Action::Move{x:9000,y:7000});
  p.mirror_pointer(true,true).unwrap();assert_eq!(p,before);
 }
 #[test]fn wait_scaling_does_not_break_down_up_pair(){
  let mut p=fixture();p.scale_waits(75).unwrap();assert_eq!(p.events[0].wait_ms,150);assert_eq!(p.events[2].wait_ms,75);p.validate().unwrap();
  let before=p.clone();assert!(p.scale_waits(0).is_err());assert_eq!(p,before);
 }
 #[test]fn initial_wait_shifts_and_stays_bounded(){
  let mut p=fixture();p.shift_start(250).unwrap();assert_eq!(p.events[0].wait_ms,450);
  p.shift_start(-1000).unwrap();assert_eq!(p.events[0].wait_ms,0);p.validate().unwrap();
 }
 #[test]fn duplicate_full_cycle_remains_balanced_and_valid(){
  let mut p=fixture();p.double_sequence().unwrap();assert_eq!(p.events.len(),6);p.validate().unwrap();
  assert_eq!(p.events[3].action,Action::Move{x:1000,y:3000});
 }
 #[test]fn exact_no_wait_duplicates_can_compact_without_drag_motion_loss(){
  let mut p=fixture();p.events.insert(1,Event{wait_ms:0,action:Action::Move{x:1000,y:3000}});
  p.validate().unwrap();assert_eq!(p.compact_exact_duplicates().unwrap(),1);assert_eq!(p.events.len(),3);
  let mut q=fixture();q.events.insert(1,Event{wait_ms:15,action:Action::Move{x:1000,y:3000}});
  assert_eq!(q.compact_exact_duplicates().unwrap(),0);
 }
 #[test]fn rejected_transform_keeps_original_plan(){
  let mut p=fixture();let before=p.clone();
  assert!(p.scale_waits(300).is_err());assert_eq!(p,before);
  let mut p=Plan::default();assert!(p.shift_start(10).is_err());assert!(p.double_sequence().is_err());
 }
}

#[cfg(test)]mod v101_keyboard_and_double_click_tests{
 use super::*;
 #[test]fn common_nonmodifier_keys_supported_with_f12_reserved(){
  for key in [0x1b,0x08,0x2d,0x2e,0x24,0x23,0x21,0x22,0x70,0x7a]{
   assert!(KEYS.contains(&key));assert_ne!(key_name(key),"?");
  }
  assert!(!KEYS.contains(&0x7b));
 }
 #[test]fn fast_double_click_has_two_balanced_presses(){
  let mut p=Plan::default();p.append_double_click(3000,4000,0,100,50,125).unwrap();
  assert_eq!(p.events.len(),6);assert_eq!(p.events[3].wait_ms,125);p.validate().unwrap();
  let old=p.clone();assert!(p.append_double_click(3000,4000,0,10,50,4).is_err());assert_eq!(p,old);
 }
 #[test]fn recorded_path_can_nudge_without_exceeding_window(){
  let mut p=Plan::default();p.append_click(9700,100,0,100,50).unwrap();
  p.translate_pointer(500,-500).unwrap();assert_eq!(p.events[0].action,Action::Move{x:10000,y:0});
  p.validate().unwrap();
 }
}

pub fn starter(index:usize)->Result<Plan,String>{let mut p=Plan::default();match index{0=>p.append_key("A",500,100)?,1=>{p.append_key("A",500,100)?;p.append_key("B",300,100)?;},2=>p.append_click(5000,5000,0,500,100)?,_=>return Err("Unknown template".into())};p.validate()?;Ok(p)}
#[cfg(test)]mod starter_tests{#[test]fn all_templates_balanced_and_preview_releases(){for i in 0..3{let p=super::starter(i).unwrap();let v=super::preview(&p,p.duration());assert!(v.keys.is_empty()&&v.buttons.is_empty());assert!(p.duration()<=2000);}assert!(super::starter(9).is_err());}}

#[derive(Default)]struct PauseGate{held:bool,requested:bool,paused:bool}
impl PauseGate{fn sample(&mut self,key:bool,safe:bool)->bool{if key&&!self.held{if self.paused{self.paused=false;self.requested=false;}else{self.requested=!self.requested;}}self.held=key;if safe&&self.requested&&!self.paused{self.paused=true;self.requested=false;}self.paused}}
#[cfg(test)]mod pause_tests{#[test]fn pause_waits_for_released_inputs(){let mut p=super::PauseGate::default();assert!(!p.sample(true,false));assert!(!p.sample(false,false));assert!(p.sample(false,true));assert!(!p.sample(true,true));assert!(!p.sample(false,true));}}
