//! Optional reversible system power-plan selection. Never applies on startup.
use std::{ffi::c_void,fs,path::{Path,PathBuf},io::Write};
#[repr(C)]
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
pub struct Guid{a:u32,b:u16,c:u16,d:[u8;8]}
const BALANCED:Guid=Guid{a:0x381b4222,b:0xf694,c:0x41f0,d:[0x96,0x85,0xff,0x5b,0xb2,0x60,0xdf,0x2e]};
const HIGH:Guid=Guid{a:0x8c5e7fda,b:0xe8bf,c:0x4a96,d:[0x9a,0x85,0xa6,0xe2,0x3a,0x8c,0x63,0x5c]};
const SAVER:Guid=Guid{a:0xa1841308,b:0x3541,c:0x4fab,d:[0xbc,0x81,0xf7,0x15,0x56,0xf2,0x0b,0x4a]};
type Handle=*mut c_void;
#[link(name="kernel32")]
unsafe extern "system"{
    fn LocalFree(ptr:Handle)->Handle;
    fn LoadLibraryExW(name:*const u16,file:Handle,flags:u32)->Handle;
    fn GetProcAddress(module:Handle,name:*const u8)->*const c_void;
    fn FreeLibrary(module:Handle)->i32;
}
struct PowerLibrary(Handle);
impl PowerLibrary{
    fn open()->Result<Self,String>{
        let name:Vec<u16>="powrprof.dll".encode_utf16().chain(Some(0)).collect();
        let module=unsafe{LoadLibraryExW(name.as_ptr(),std::ptr::null_mut(),0x800)};
        if module.is_null(){Err("Windows power API unavailable".into())}else{Ok(Self(module))}
    }
    fn symbol(&self,name:&[u8])->Result<*const c_void,String>{
        let address=unsafe{GetProcAddress(self.0,name.as_ptr())};
        if address.is_null(){Err("Windows power API entry point unavailable".into())}else{Ok(address)}
    }
}
impl Drop for PowerLibrary{fn drop(&mut self){unsafe{FreeLibrary(self.0);}}}

pub fn current()->Result<Guid,String>{
    let mut ptr=std::ptr::null_mut::<Guid>();
    let lib=PowerLibrary::open()?;
    let get:unsafe extern "system" fn(Handle,*mut *mut Guid)->u32=
        unsafe{std::mem::transmute(lib.symbol(b"PowerGetActiveScheme\0")?)};
    let code=unsafe{get(std::ptr::null_mut(),&mut ptr)};
    if code!=0||ptr.is_null(){return Err(format!("PowerGetActiveScheme Windows code {code}"));}
    let value=unsafe{*ptr};
    unsafe{LocalFree(ptr.cast());}
    Ok(value)
}
impl Guid{
    pub fn label(self)->&'static str{
        if self==BALANCED{"Balanced"}else if self==HIGH{"High performance"}else if self==SAVER{"Power saver"}else{"Custom Windows plan"}
    }
}
fn set(value:Guid)->Result<(),String>{
    let lib=PowerLibrary::open()?;
    let set:unsafe extern "system" fn(Handle,*const Guid)->u32=
        unsafe{std::mem::transmute(lib.symbol(b"PowerSetActiveScheme\0")?)};
    let code=unsafe{set(std::ptr::null_mut(),&value)};
    if code==0{Ok(())}else{Err(format!("Windows refused the plan (code {code}); it may not exist on this PC"))}
}
#[derive(Default)]
pub struct PowerPlan{
    pub observed:Option<Guid>,original:Option<Guid>,applied:Option<Guid>,
}
impl PowerPlan{
    pub fn active(&self)->bool{self.applied.is_some()}
    pub fn recovery_hint(&self)->Result<Option<String>,String>{
        let Some(j)=pending_record()? else{return Ok(None)};
        let now=current()?;
        Ok(Some(if now==j.applied{
            format!("Recovery available: Windows still uses {}. Original plan: {}. Restore requires explicit click.",j.applied.label(),j.original.label())
        }else{format!("Power plan changed externally ({}); a stale Horizon journal can be safely cleared without changing Windows.",now.label())}))
    }
    pub fn recover_previous(&mut self)->Result<String,String>{
        if self.active(){return Err("Horizon still owns an active power-plan change; use Restore original".into());}
        let Some(j)=pending_record()? else{return Ok("No previous power-plan journal to restore".into())};
        let current_plan=current()?;
        if current_plan!=j.applied{remove_record()?;self.observed=Some(current_plan);return Ok("Windows plan differs from the previous Horizon selection; only the stale journal was cleared".into());}
        set(j.original)?;
        let confirmed=current()?;
        if confirmed!=j.original{return Err("Windows did not confirm restoration; journal preserved".into());}
        remove_record()?;self.observed=Some(confirmed);
        Ok(format!("Restored previous {} plan after interrupted session",confirmed.label()))
    }
    pub fn refresh(&mut self)->Result<(),String>{self.observed=Some(current()?);Ok(())}
    pub fn apply(&mut self,index:u8)->Result<String,String>{
        if self.applied.is_some(){return Err("Restore the current Horizon power plan before selecting another".into());}
        if pending_record()?.is_some(){return Err("A previous power-plan recovery journal exists. Check/restore it first using the recovery control".into());}
        let target=match index{0=>BALANCED,1=>HIGH,2=>SAVER,_=>return Err("Invalid plan".into())};
        let before=current()?;
        if self.applied.is_some_and(|x|x!=before){self.original=None;self.applied=None;}
        if before==target{self.observed=Some(before);return Ok(format!("Already using {}",target.label()));}
        create_record(&Journal{format:"HorizonBoost.PowerRecovery.1".into(),original:before,applied:target})?;
        if let Err(e)=set(target){let _=remove_record();return Err(e);}
        self.original=Some(self.original.unwrap_or(before));
        self.applied=Some(target);
        self.observed=Some(current()?);
        if self.observed!=Some(target){return Err("Windows did not confirm selected plan".into());}
        Ok(format!("Windows confirmed {}",target.label()))
    }
    pub fn restore(&mut self)->Result<String,String>{
        let (Some(original),Some(applied))=(self.original,self.applied)else{return Ok("No Horizon power-plan change to restore".into());};
        let actual=current()?;
        if actual!=applied{
            self.original=None;self.applied=None;self.observed=Some(actual);
            let _=remove_record();
            return Ok("Power plan changed externally; left it untouched".into());
        }
        set(original)?;
        self.observed=Some(current()?);
        if self.observed!=Some(original){return Err("Could not confirm original power plan".into());}
        self.original=None;self.applied=None;
        remove_record()?;
        Ok(format!("Restored {}",original.label()))
    }
}
impl Drop for PowerPlan{fn drop(&mut self){let _=self.restore();}}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn read_only_windows_current_plan(){assert!(current().is_ok());}
    #[test]fn predefined_plans_distinct(){assert_ne!(BALANCED,HIGH);assert_ne!(HIGH,SAVER);}
}


#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal{format:String,original:Guid,applied:Guid}
fn record_path()->PathBuf{crate::settings::folder().join("power-plan-recovery.json")}
fn read_record_at(path:&Path)->Result<Option<Journal>,String>{
 let meta=match fs::symlink_metadata(path){
  Ok(m)=>m,
  Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Ok(None),
  Err(e)=>return Err(e.to_string()),
 };
 if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>1024{
  return Err("Power recovery journal is not an ordinary small file".into());
 }
 let text=fs::read_to_string(path).map_err(|e|e.to_string())?;
 let row:Journal=serde_json::from_str(&text).map_err(|e|format!("Invalid power recovery journal: {e}"))?;
 if row.format!="HorizonBoost.PowerRecovery.1"||row.original==row.applied||
   ![BALANCED,HIGH,SAVER].contains(&row.applied){
  return Err("Unsupported or invalid power recovery record; no OS action performed".into());
 }
 Ok(Some(row))
}
fn pending_record()->Result<Option<Journal>,String>{read_record_at(&record_path())}
fn create_record(row:&Journal)->Result<(),String>{
 let path=record_path();
 fs::create_dir_all(path.parent().ok_or("No recovery location")?).map_err(|e|e.to_string())?;
 let bytes=serde_json::to_vec(row).map_err(|e|e.to_string())?;
 if bytes.len()>1024{return Err("Recovery record too large".into());}
 let mut out=fs::OpenOptions::new().write(true).create_new(true).open(&path)
  .map_err(|e|format!("Cannot atomically reserve power recovery journal: {e}"))?;
 if let Err(e)=out.write_all(&bytes).and_then(|_|out.sync_all()){
  let _=fs::remove_file(path);
  return Err(format!("Cannot persist power recovery record: {e}"));
 }
 Ok(())
}
fn remove_record()->Result<(),String>{
 match fs::remove_file(record_path()){
  Ok(())=>Ok(()),
  Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(()),
  Err(e)=>Err(format!("Recovery journal still on disk: {e}")),
 }
}
#[cfg(test)]mod recovery_tests{
 use super::*;
 #[test]fn durable_recovery_marker_validates_and_does_not_touch_windows(){
  let path=std::env::temp_dir().join(format!("hb-recovery-fixture-{}-{}.json",std::process::id(),12345));
  let row=Journal{format:"HorizonBoost.PowerRecovery.1".into(),original:BALANCED,applied:HIGH};
  let raw=serde_json::to_vec(&row).unwrap();
  fs::write(&path,&raw).unwrap();
  let restored=read_record_at(&path).unwrap().unwrap();
  assert_eq!(restored.original,BALANCED);assert_eq!(restored.applied,HIGH);
  let mut value:serde_json::Value=serde_json::from_slice(&raw).unwrap();
  value["format"]=serde_json::json!("evil");fs::write(&path,value.to_string()).unwrap();
  assert!(read_record_at(&path).is_err());
  value["format"]=serde_json::json!("HorizonBoost.PowerRecovery.1");
  value["command"]=serde_json::json!("execute");fs::write(&path,value.to_string()).unwrap();
  assert!(read_record_at(&path).is_err());
  fs::remove_file(&path).unwrap();
  assert!(read_record_at(&path).unwrap().is_none());
 }
}

