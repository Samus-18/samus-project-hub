use std::{fs,path::{Path,PathBuf},time::{SystemTime,UNIX_EPOCH},sync::atomic::{AtomicU64,Ordering}};
use crate::profile_portable::Portable;
fn directory()->PathBuf{crate::settings::folder().join("profile-backups")}
fn valid_name(s:&str)->bool{
 s.starts_with("profile-")&&s.ends_with(".json")&&s.len()<90&&
 s.strip_prefix("profile-").and_then(|v|v.strip_suffix(".json"))
 .is_some_and(|v|!v.is_empty()&&v.bytes().all(|c|c.is_ascii_digit()||c==b'-'))
}
fn save_to(dir:&Path,profile:&Portable)->Result<String,String>{
 static SEQ:AtomicU64=AtomicU64::new(0);
 let stamp=SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_millis();
 let name=format!("profile-{stamp}-{}-{}.json",std::process::id(),SEQ.fetch_add(1,Ordering::Relaxed));
 fs::create_dir_all(dir).map_err(|e|e.to_string())?;
 let data=profile.encode()?;
 crate::settings::atomic_json_replace(&dir.join(&name),data.as_bytes())?;
 Ok(name)
}
fn load_from(dir:&Path,name:&str)->Result<Portable,String>{
 if !valid_name(name){return Err("Invalid backup name".into());}
 let path=dir.join(name);
 let meta=fs::symlink_metadata(&path).map_err(|e|e.to_string())?;
 if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>65536{return Err("Not a regular supported profile backup".into());}
 Portable::decode(&fs::read_to_string(path).map_err(|e|e.to_string())?)
}
pub fn save(profile:&Portable)->Result<String,String>{save_to(&directory(),profile)}
pub fn load(name:&str)->Result<Portable,String>{load_from(&directory(),name)}
pub fn list()->Result<Vec<String>,String>{
 let dir=directory();if !dir.exists(){return Ok(Vec::new());}
 let mut names=Vec::new();
 for entry in fs::read_dir(dir).map_err(|e|e.to_string())?.take(8192){
  let e=entry.map_err(|e|e.to_string())?;
  if let Some(name)=e.file_name().to_str(){
   if valid_name(name)&&e.file_type().map_err(|e|e.to_string())?.is_file()&&e.metadata().is_ok_and(|m|m.len()<=65536){names.push(name.to_owned());}
  }
 }
 names.sort_unstable_by(|a,b|b.cmp(a));names.truncate(100);Ok(names)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn backup_roundtrip_and_no_path_traversal(){
  let dir=std::env::temp_dir().join(format!("hb-profile-backup-{}",std::process::id()));
  let p=Portable::capture(&crate::settings::Settings::default(),"g",60);
  let n=save_to(&dir,&p).unwrap();assert!(valid_name(&n));assert_eq!(load_from(&dir,&n).unwrap().preset,p.preset);
  assert!(load_from(&dir,"../settings.json").is_err());assert!(load_from(&dir,"profile-abc.json").is_err());
  fs::remove_dir_all(dir).unwrap();
 }
}
