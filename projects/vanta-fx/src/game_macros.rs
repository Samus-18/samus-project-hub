//! Explicit, finite keyboard tap sequences for offline games / accessibility.
 //! No key recording, mouse aiming, repeated loops, game memory or hooks.
use std::{ffi::c_void,sync::{Arc,atomic::{AtomicBool,Ordering},mpsc::{self,Receiver}},thread,time::{Duration,Instant}};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
#[serde(deny_unknown_fields)]
pub struct Step{pub key:String,pub delay_ms:u32,#[serde(default)]pub hold_ms:u32}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacroProfile{pub steps:Vec<Step>}
impl Default for MacroProfile{fn default()->Self{Self{steps:vec![Step{key:"E".into(),delay_ms:200,hold_ms:0}]}}}
pub fn key_code(key:&str)->Option<u16>{
    let key=key.trim().to_ascii_uppercase();
    match key.as_str(){
        "SPACE"=>Some(0x20),"ENTER"=>Some(0x0d),"TAB"=>Some(0x09),
        "UP"=>Some(0x26),"DOWN"=>Some(0x28),"LEFT"=>Some(0x25),"RIGHT"=>Some(0x27),
        "ESC"|"ESCAPE"=>Some(0x1b),"BACKSPACE"=>Some(0x08),"DELETE"|"DEL"=>Some(0x2e),
        "INSERT"|"INS"=>Some(0x2d),"HOME"=>Some(0x24),"END"=>Some(0x23),
        "PAGEUP"|"PGUP"=>Some(0x21),"PAGEDOWN"|"PGDN"=>Some(0x22),
        _ if key.starts_with('F')&&key.len()>1=>key[1..].parse::<u16>().ok().filter(|n|(1..=11).contains(n)).map(|n|0x6f+n),
        _ if key.len()==1 && key.as_bytes()[0].is_ascii_alphanumeric()=>Some(key.as_bytes()[0] as u16),
        _=>None
    }
}
impl MacroProfile{
    pub fn portable_json(&self)->Result<String,String>{
        self.validate()?;
        serde_json::to_string_pretty(&serde_json::json!({"format":"HorizonBoost.Macro.1","steps":self.steps})).map_err(|e|e.to_string())
    }
    pub fn from_portable_json(raw:&str)->Result<Self,String>{
        if raw.len()>8192{return Err("Macro JSON too large".into());}
        #[derive(Deserialize)]#[serde(deny_unknown_fields)]
        struct Portable{format:String,steps:Vec<Step>}
        let p:Portable=serde_json::from_str(raw).map_err(|e|e.to_string())?;
        if p.format!="HorizonBoost.Macro.1"{return Err("Unsupported macro format".into());}
        let profile=Self{steps:p.steps};profile.validate()?;Ok(profile)
    }
    pub fn timeline(&self)->Result<Vec<(u32,String)>,String>{
        self.validate()?;let mut t=0;
        Ok(self.steps.iter().map(|s|{let v=(t,s.key.clone());t+=s.delay_ms+s.hold_ms;v}).collect())
    }
    pub fn validate(&self)->Result<(),String>{
        if self.steps.is_empty()||self.steps.len()>12{return Err("Choose 1–12 taps".into());}
        if self.steps.iter().any(|s|key_code(&s.key).is_none()||!(100..=3000).contains(&s.delay_ms)||s.hold_ms>2000){
            return Err("Use A–Z, 0–9, Space, Enter, Tab or arrow keys; delay 100–3000 ms".into());
        }
        if self.steps.iter().map(|s|s.delay_ms as u64+s.hold_ms as u64).sum::<u64>()>15000{return Err("Maximum duration 15 seconds".into());}
        Ok(())
    }
}
type Handle=*mut c_void;
#[repr(C)]#[derive(Clone,Copy)]
struct Keyboard{vk:u16,scan:u16,flags:u32,time:u32,extra:usize}
#[repr(C)]#[derive(Clone,Copy)]
struct Mouse{dx:i32,dy:i32,data:u32,flags:u32,time:u32,extra:usize}
#[repr(C)]#[derive(Clone,Copy)]
union InputData{keyboard:Keyboard,mouse:Mouse}
#[repr(C)]#[derive(Clone,Copy)]
struct Input{kind:u32,data:InputData}
#[link(name="user32")]
unsafe extern "system"{
    fn GetForegroundWindow()->Handle;
    fn GetWindowThreadProcessId(hwnd:Handle,pid:*mut u32)->u32;
    fn GetAsyncKeyState(vkey:i32)->i16;
    fn SendInput(count:u32,inputs:*const Input,size:i32)->u32;
}
pub fn foreground_pid()->u32{
    let mut pid=0;
    unsafe{GetWindowThreadProcessId(GetForegroundWindow(),&mut pid);}
    pid
}
fn down(key:i32)->bool{unsafe{GetAsyncKeyState(key) as u16&0x8000!=0}}
pub fn trigger()->bool{down(0x11)&&down(0x10)&&down(0x55)} // Ctrl Shift U
fn cancel_key()->bool{down(0x7b)} // F12
fn modifiers()->bool{[0x10,0x11,0x12,0x5b,0x5c].iter().any(|&k|down(k))}
pub(crate) fn emit_key(code:u16,up:bool)->Result<(),String>{
 let flags=if matches!(code,0x21..=0x28|0x2d|0x2e){1}else{0};
 let event=Input{kind:1,data:InputData{keyboard:Keyboard{vk:code,scan:0,flags:flags|if up{2}else{0},time:0,extra:0}}};
 let n=unsafe{SendInput(1,&event,std::mem::size_of::<Input>() as i32)};
 if n==1{Ok(())}else{Err("Windows did not accept key event".into())}
}
/// Relative Win32 SendInput motion. Not equivalent to a raw HID event; a game
/// may ignore or transform it. Called only after foreground/focus permission.
pub(crate) fn emit_relative_mouse(dx:i32,dy:i32)->Result<(),String>{
 if dx==0&&dy==0{return Ok(());}
 if dx.abs()>32767||dy.abs()>32767{return Err("Camera delta out of safe range".into());}
 let e=Input{kind:0,data:InputData{mouse:Mouse{
  dx,dy,data:0,flags:0x0001,time:0,extra:0
 }}};
 if unsafe{SendInput(1,&e,std::mem::size_of::<Input>() as i32)}==1{
  Ok(())
 }else{Err("Windows refused relative mouse input".into())}
}
pub(crate) fn emit_mouse(button:u8,up:bool)->Result<(),String>{
 let flags=match(button,up){(0,false)=>2,(0,true)=>4,(1,false)=>8,(1,true)=>16,_=>return Err("Unsupported button".into())};
 let e=Input{kind:0,data:InputData{mouse:Mouse{dx:0,dy:0,data:0,flags,time:0,extra:0}}};
 if unsafe{SendInput(1,&e,std::mem::size_of::<Input>()as i32)}==1{Ok(())}else{Err("Windows refused mouse event".into())}
}
struct Held(u16,bool);
impl Held{fn begin(code:u16)->Result<Self,String>{emit_key(code,false)?;Ok(Self(code,true))}fn release(&mut self)->Result<(),String>{if self.1{emit_key(self.0,true)?;self.1=false;}Ok(())}}
impl Drop for Held{fn drop(&mut self){if self.1{let _=self.release();}}}
fn wait_guarded(ms:u32,guard:impl Fn()->Result<(),String>)->Result<(),String>{
 let start=Instant::now();let duration=Duration::from_millis(ms as u64);
 loop{guard()?;let remain=duration.saturating_sub(start.elapsed());if remain.is_zero(){return Ok(());}thread::sleep(remain.min(Duration::from_millis(10)));}
}
fn tap(code:u16)->Result<(),String>{
    let flags=if matches!(code,0x21..=0x28|0x2d|0x2e){1}else{0};
    let input=|up|Input{kind:1,data:InputData{keyboard:Keyboard{
        vk:code,scan:0,flags:flags|if up{2}else{0},time:0,extra:0
    }}};
    let events=[input(false),input(true)];
    let sent=unsafe{SendInput(2,events.as_ptr(),std::mem::size_of::<Input>() as i32)};
    if sent==2{Ok(())}else{
        if sent==1{let release=[input(true)];unsafe{SendInput(1,release.as_ptr(),std::mem::size_of::<Input>() as i32);}}
        Err("Windows did not accept the complete tap (UIPI or application restrictions)".into())
    }
}
#[derive(Default)]
pub struct Runner{cancel:Option<Arc<AtomicBool>>,rx:Option<Receiver<Result<usize,String>>>}
impl Runner{
    pub fn running(&self)->bool{self.rx.is_some()}
    pub fn start(&mut self,pid:u32,profile:MacroProfile)->Result<(),String>{self.start_with_key(pid,profile,0x55)}
    pub fn start_with_key(&mut self,pid:u32,profile:MacroProfile,trigger_key:u16)->Result<(),String>{
        profile.validate()?;
        if self.running(){return Err("A sequence is already running".into());}
        if pid==0||pid==std::process::id()||foreground_pid()!=pid{return Err("Focus the explicitly selected offline game first".into());}
        let run=Arc::new(AtomicBool::new(true));let active=run.clone();
        let (tx,rx)=mpsc::channel();
        thread::Builder::new().name("Horizon one-shot macro".into()).spawn(move||{
            let guard=||->Result<(),String>{
                if !active.load(Ordering::Acquire)||cancel_key(){return Err("Macro cancelled".into());}
                if foreground_pid()!=pid{return Err("Macro stopped: game lost focus".into());}
                Ok(())
            };
            let result=(||{
                let start=Instant::now();
                while modifiers()||down(trigger_key as i32){
                    guard()?;
                    if start.elapsed()>Duration::from_secs(3){return Err("Release hotkey modifiers".into());}
                    thread::sleep(Duration::from_millis(15));
                }
                let mut count=0;
                for step in profile.steps{
                    guard()?;
                    let code=key_code(&step.key).ok_or("Invalid key")?;
                    if modifiers()||down(code as i32){return Err("Macro stopped: physical key held".into());}
                    if step.hold_ms==0{tap(code)?;}else{
                     let mut held=Held::begin(code)?;
                     let result=wait_guarded(step.hold_ms,||{guard()?;if modifiers(){return Err("Macro stopped: modifier pressed".into());}Ok(())});
                     let release=held.release();result?;release?;
                    }
                    count+=1;wait_guarded(step.delay_ms,&guard)?;
                }
                Ok(count)
            })();
            let _=tx.send(result);
        }).map_err(|e|e.to_string())?;
        self.cancel=Some(run);self.rx=Some(rx);Ok(())
    }
    pub fn cancel(&self){if let Some(x)=&self.cancel{x.store(false,Ordering::Release);}}
    pub fn poll(&mut self)->Option<Result<usize,String>>{
        match self.rx.as_ref()?.try_recv(){
            Ok(result)=>{self.rx=None;self.cancel=None;Some(result)}
            Err(mpsc::TryRecvError::Disconnected)=>{self.rx=None;self.cancel=None;Some(Err("Macro worker stopped".into()))}
            Err(mpsc::TryRecvError::Empty)=>None
        }
    }
}
impl Drop for Runner{fn drop(&mut self){self.cancel();}}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn portable_macro_timeline_and_validation(){
        let p=MacroProfile{steps:vec![Step{key:"E".into(),delay_ms:150,hold_ms:0},Step{key:"SPACE".into(),delay_ms:200,hold_ms:0}]};
        let other=MacroProfile::from_portable_json(&p.portable_json().unwrap()).unwrap();
        assert_eq!(other.timeline().unwrap(),vec![(0,"E".into()),(150,"SPACE".into())]);
        assert!(MacroProfile::from_portable_json(r#"{"format":"HorizonBoost.Macro.1","steps":[{"key":"E","delay_ms":100,"command":"run"}]}"#).is_err());
    }
    #[test]fn keyboard_abi_matches_win64(){assert_eq!(std::mem::size_of::<Input>(),40);}
    #[test]fn sequence_validation_is_bounded(){
        assert!(MacroProfile::default().validate().is_ok());
        assert!(MacroProfile{steps:vec![Step{key:"ALT+F4".into(),delay_ms:200,hold_ms:0}]}.validate().is_err());
        assert!(MacroProfile{steps:vec![Step{key:"E".into(),delay_ms:3000,hold_ms:0};6]}.validate().is_err());
    }
    #[test]fn refuses_own_process_without_sending(){
        assert!(Runner::default().start(std::process::id(),MacroProfile::default()).is_err());
    }
}

#[cfg(test)]mod hold_tests{use super::*;#[test]fn legacy_json_and_hold_limits(){let old=MacroProfile::from_portable_json(r#"{"format":"HorizonBoost.Macro.1","steps":[{"key":"E","delay_ms":100}]}"#).unwrap();assert_eq!(old.steps[0].hold_ms,0);let mut p=old;p.steps[0].hold_ms=2001;assert!(p.validate().is_err());p.steps[0].hold_ms=100;assert!(p.validate().is_ok());p.steps.push(Step{key:"SPACE".into(),delay_ms:100,hold_ms:0});assert_eq!(p.timeline().unwrap()[1].0,200);assert!(wait_guarded(200,||Err("cancelled".into())).is_err());}}

#[cfg(test)]mod v101_basic_key_tests{
 use super::*;
 #[test]fn function_key_names_with_reserved_cancel(){
  for n in 1..=11{assert_eq!(key_code(&format!("F{n}")),Some(0x6f+n));}
  for(k,v)in [("ESC",0x1b),("BACKSPACE",0x08),("DELETE",0x2e),("HOME",0x24),("PAGEUP",0x21),("INSERT",0x2d)]{
   assert_eq!(key_code(k),Some(v));
  }
  assert_eq!(key_code("F12"),None);assert_eq!(key_code("ALT+F4"),None);assert_eq!(key_code("F"),Some(0x46));
 }
}
