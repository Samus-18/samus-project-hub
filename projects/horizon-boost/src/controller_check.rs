use std::ffi::c_void;
type Handle=*mut c_void;
#[repr(C)]#[derive(Clone,Copy,Default)]struct Pad{buttons:u16,lt:u8,rt:u8,lx:i16,ly:i16,rx:i16,ry:i16}
#[repr(C)]#[derive(Clone,Copy,Default)]struct State{packet:u32,pad:Pad}
#[repr(C)]#[derive(Clone,Copy,Default)]struct Battery{kind:u8,level:u8}
#[link(name="kernel32")]
unsafe extern "system"{
 fn LoadLibraryExW(name:*const u16,file:Handle,flags:u32)->Handle;
 fn GetProcAddress(module:Handle,name:*const u8)->*const c_void;
 fn FreeLibrary(module:Handle)->i32;
}
#[derive(Clone,Debug)]
pub struct Reading{pub slot:u32,pub packet:u32,pub buttons:u16,pub left:[f32;2],pub right:[f32;2],
 pub triggers:[f32;2],pub battery:Option<&'static str>}
fn axis(x:i16)->f32{if x<0{x as f32/32768.}else{x as f32/32767.}}
pub fn magnitude(v:[f32;2])->f32{(v[0]*v[0]+v[1]*v[1]).sqrt()}
pub fn deadzone(v:[f32;2],zone:f32)->[f32;2]{
 let zone=if zone.is_finite(){zone.clamp(0.,0.95)}else{0.};if v.iter().any(|x|!x.is_finite()){return [0.,0.];}let m=magnitude(v);
 if m<=zone||m==0.{[0.,0.]}else{let a=((m-zone)/(1.-zone)).min(1.)/m;[v[0]*a,v[1]*a]}
}
pub fn read()->Result<Vec<Reading>,String>{
 static API:std::sync::OnceLock<Result<Api,String>>=std::sync::OnceLock::new();
 let api=API.get_or_init(load_api).as_ref().map_err(Clone::clone)?;
 read_with(|slot,s|unsafe{(api.get)(slot,s)},|slot,b|api.battery.map_or(1167,|f|unsafe{f(slot,0,b)}))
}
type GetState=unsafe extern "system" fn(u32,*mut State)->u32;
type GetBattery=unsafe extern "system" fn(u32,u8,*mut Battery)->u32;
struct Api{get:GetState,battery:Option<GetBattery>}
fn load_api()->Result<Api,String>{
 let name:Vec<u16>="xinput1_4.dll".encode_utf16().chain(Some(0)).collect();
 let module=unsafe{LoadLibraryExW(name.as_ptr(),std::ptr::null_mut(),0x800)};
 if module.is_null(){return Err("Windows XInput 1.4 unavailable".into());}
 let state=unsafe{GetProcAddress(module,b"XInputGetState\0".as_ptr())};
 let bat=unsafe{GetProcAddress(module,b"XInputGetBatteryInformation\0".as_ptr())};
 if state.is_null(){unsafe{FreeLibrary(module);};return Err("XInputGetState unavailable".into());}
 // Keep a single System32 DLL reference for the lifetime of the process.
 Ok(Api{get:unsafe{std::mem::transmute::<_,GetState>(state)},battery:if bat.is_null(){None}else{Some(unsafe{std::mem::transmute::<_,GetBattery>(bat)})}})
}
fn battery_label(b:Battery)->Option<&'static str>{match b.kind{1=>Some("Wired"),2|3=>match b.level{0=>Some("Empty"),1=>Some("Low"),2=>Some("Medium"),3=>Some("Full"),_=>None},_=>None}}
fn read_with(mut get:impl FnMut(u32,&mut State)->u32,mut get_battery:impl FnMut(u32,&mut Battery)->u32)->Result<Vec<Reading>,String>{
 let mut out=Vec::new();
 for slot in 0..4{
  let mut s=State::default();if get(slot,&mut s)!=0{continue;}
  let mut b=Battery::default();
  let battery=if get_battery(slot,&mut b)==0{battery_label(b)}else{None};
  let p=s.pad;out.push(Reading{slot,packet:s.packet,buttons:p.buttons,left:[axis(p.lx),axis(p.ly)],
   right:[axis(p.rx),axis(p.ry)],triggers:[p.lt as f32/255.,p.rt as f32/255.],battery});
 }
 Ok(out)
}
#[derive(Default)]
pub struct Drift{slot:Option<u32>,source_slot:Option<u32>,count:u32,pub maximum:[f32;2],sum:[f32;2],samples:Vec<[f32;2]>,completed:bool}
impl Drift{
 pub fn start(&mut self,slot:u32){*self=Self{slot:Some(slot),source_slot:Some(slot),..Default::default()};}
 pub fn stop(&mut self){self.slot=None;}
 pub fn running(&self)->bool{self.slot.is_some()}
 pub fn source_slot(&self)->Option<u32>{self.source_slot}
 pub fn reset(&mut self){*self=Self::default();}
 pub fn sample(&mut self,r:&Reading){
  if self.slot!=Some(r.slot){return;}
  let values=[magnitude(r.left),magnitude(r.right)];if values.iter().any(|v|!v.is_finite()||*v>1.415){return;}
  for i in 0..2{self.maximum[i]=self.maximum[i].max(values[i]);self.sum[i]+=values[i];}
  self.samples.push(values);self.count+=1;if self.count>=100{self.completed=true;self.stop();}
 }
 pub fn reconcile(&mut self,rows:&[Reading]){if self.source_slot.is_some_and(|slot|!rows.iter().any(|r|r.slot==slot)){self.reset();}}
 pub fn count(&self)->u32{self.count}
 pub fn completed(&self)->bool{self.completed}
 pub fn p95(&self)->Option<[f32;2]>{
  if !self.completed||self.samples.len()!=100{return None;}
  Some(std::array::from_fn(|i|{let mut v=self.samples.iter().map(|s|s[i]).collect::<Vec<_>>();v.sort_by(f32::total_cmp);v[94]}))
 }
 pub fn recommended(&self)->Option<f32>{let p=self.p95()?;Some((p[0].max(p[1])+0.025).clamp(0.03,0.95))}

 pub fn mean(&self)->Option<[f32;2]>{(self.count>0).then(||[self.sum[0]/self.count as f32,self.sum[1]/self.count as f32])}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn abi_and_normalization(){assert_eq!(std::mem::size_of::<State>(),16);assert_eq!(axis(-32768),-1.);assert_eq!(axis(32767),1.);}
 #[test]fn radial_deadzone(){assert_eq!(deadzone([0.1,0.1],0.2),[0.,0.]);let v=deadzone([1.,1.],0.2);assert!((magnitude(v)-1.).abs()<1e-5);}
 #[test]fn real_read_only_xinput_api(){assert!(read().is_ok());}
 #[test]fn drift_collects_only_selected_slot(){
  let mut d=Drift::default();d.start(2);let mut r=Reading{slot:0,packet:0,buttons:0,left:[0.1,0.],right:[0.,0.],triggers:[0.,0.],battery:None};
  d.sample(&r);assert_eq!(d.count(),0);r.slot=2;for _ in 0..100{d.sample(&r);}assert!(!d.running());assert_eq!(d.count(),100);assert!((d.mean().unwrap()[0]-0.1).abs()<1e-5);
 }
}

#[cfg(test)]mod distribution_tests{use super::*;#[test]fn no_recommendation_until_complete(){let mut d=Drift::default();d.start(0);let mut r=Reading{slot:0,packet:0,buttons:0,left:[0.04,0.],right:[0.03,0.],triggers:[0.,0.],battery:None};for _ in 0..95{d.sample(&r);}assert!(d.recommended().is_none());r.left=[0.4,0.];for _ in 0..5{d.sample(&r);}assert!(d.completed());assert!((d.recommended().unwrap()-0.065).abs()<0.0001);d.start(1);assert!(d.recommended().is_none());}}

#[cfg(test)]mod v88_tests{use super::*;
 #[test]fn four_slot_mock_transport_and_battery(){let rows=read_with(|i,s|{if i==1||i==3{*s=State{packet:42,pad:Pad{buttons:0x1000,lt:255,rt:0,lx:-32768,ly:32767,rx:0,ry:0}};0}else{1167}},|i,b|{*b=Battery{kind:if i==1{1}else{2},level:3};0}).unwrap();assert_eq!(rows.len(),2);assert_eq!(rows[0].slot,1);assert_eq!(rows[0].left,[-1.,1.]);assert_eq!(rows[0].triggers,[1.,0.]);assert_eq!(rows[0].battery,Some("Wired"));assert_eq!(rows[1].battery,Some("Full"));assert_eq!(battery_label(Battery{kind:255,level:3}),None);assert_eq!(battery_label(Battery{kind:2,level:4}),None);}
 #[test]fn disconnect_clears_result_and_invalid_samples_do_not_count(){let mut d=Drift::default();d.start(0);let mut r=Reading{slot:0,packet:1,buttons:0,left:[f32::NAN,0.],right:[0.,0.],triggers:[0.,0.],battery:None};d.sample(&r);assert_eq!(d.count(),0);r.left=[0.1,0.];for _ in 0..100{d.sample(&r);}assert!(d.recommended().is_some());d.reconcile(&[]);assert!(d.recommended().is_none());assert_eq!(d.source_slot(),None);assert_eq!(deadzone([f32::NAN,0.],0.2),[0.,0.]);}
}
