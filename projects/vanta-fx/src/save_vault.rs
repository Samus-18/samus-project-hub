use std::{fs,io::{Read,Write},path::{Path,PathBuf,Component},os::windows::fs::MetadataExt};
use serde::{Serialize,Deserialize};
const MAX_BYTES:u64=512*1024*1024;
const MAX_FILES:usize=4096;
const MAX_DEPTH:usize=32;
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq)]
struct FileRecord{path:String,bytes:u64,checksum:String}
#[derive(Clone,Debug,Serialize,Deserialize)]
struct Manifest{version:u32,files:Vec<FileRecord>}
#[derive(Clone,Debug)]
pub struct Summary{pub name:String,pub files:usize,pub bytes:u64}
pub fn root()->PathBuf{crate::settings::folder().join("save-vault")}
fn safe_meta(p:&Path)->Result<fs::Metadata,String>{
 let m=fs::symlink_metadata(p).map_err(|e|e.to_string())?;
 if m.file_attributes()&0x400!=0||m.file_type().is_symlink(){return Err("Links and reparse points are not supported".into());}
 Ok(m)
}
fn local(p:&Path)->bool{p.is_absolute()&&!p.to_string_lossy().starts_with("\\\\")}
fn safe_relative(s:&str)->bool{
 !s.is_empty()&&s.len()<1024&&s.split('/').all(|x|!x.is_empty()&&x!="."&&x!=".."&&!x.contains(['\\',':','\0'])&&!x.ends_with(['.',' '])&&!matches!(x.split('.').next().unwrap_or("").to_ascii_uppercase().as_str(),"CON"|"PRN"|"AUX"|"NUL"|"COM1"|"COM2"|"COM3"|"COM4"|"COM5"|"COM6"|"COM7"|"COM8"|"COM9"|"LPT1"|"LPT2"|"LPT3"|"LPT4"|"LPT5"|"LPT6"|"LPT7"|"LPT8"|"LPT9"))
}
fn relative(base:&Path,p:&Path)->Result<String,String>{
 let q=p.strip_prefix(base).map_err(|_|"Path escaped source")?;
 let mut parts=Vec::new();
 for x in q.components(){if let Component::Normal(v)=x{parts.push(v.to_str().ok_or("Non-Unicode path not supported")?);}else{return Err("Invalid relative path".into());}}
 let s=parts.join("/");if !safe_relative(&s){return Err("Unsafe relative file name".into());}Ok(s)
}
fn scan(base:&Path,p:&Path,depth:usize,files:&mut Vec<(String,u64)>,bytes:&mut u64,visited:&mut usize)->Result<(),String>{
 if depth>MAX_DEPTH{return Err("Directory nesting exceeds limit".into());}
 for e in fs::read_dir(p).map_err(|e|e.to_string())?{
  *visited+=1;if *visited>MAX_FILES*2{return Err("Too many directory entries".into());}
  let p=e.map_err(|e|e.to_string())?.path();let m=safe_meta(&p)?;
  if m.is_dir(){scan(base,&p,depth+1,files,bytes,visited)?;}
  else if m.is_file(){
   *bytes=bytes.checked_add(m.len()).ok_or("Size overflow")?;
   if *bytes>MAX_BYTES||files.len()>=MAX_FILES{return Err("Backup limit: 512 MiB / 4096 files".into());}
   files.push((relative(base,&p)?,m.len()));
  }else{return Err("Unsupported file type".into());}
 }Ok(())
}
// FNV-1a detects accidental corruption; not cryptographic authenticity.
fn copy_checked(source:&Path,target:Option<&Path>,expected:u64)->Result<String,String>{
 let m=safe_meta(source)?;if !m.is_file()||m.len()!=expected{return Err("Source changed or is not a file".into());}
 let mut input=fs::File::open(source).map_err(|e|e.to_string())?;
 let mut out=if let Some(p)=target{
  fs::create_dir_all(p.parent().ok_or("Missing parent")?).map_err(|e|e.to_string())?;
  Some(fs::OpenOptions::new().write(true).create_new(true).open(p).map_err(|e|e.to_string())?)
 }else{None};
 let mut h=0xcbf29ce484222325u64;let mut n=0u64;let mut buf=[0u8;65536];
 loop{let k=input.read(&mut buf).map_err(|e|e.to_string())?;if k==0{break;}n+=k as u64;
 if n>expected{return Err("Source grew during backup".into());}
 for &v in &buf[..k]{h^=v as u64;h=h.wrapping_mul(0x100000001b3);}
 if let Some(f)=out.as_mut(){f.write_all(&buf[..k]).map_err(|e|e.to_string())?;}}
 if n!=expected||safe_meta(source)?.modified().ok()!=m.modified().ok(){return Err("Source changed during copy; close the game and retry".into());}
 if let Some(f)=out{f.sync_all().map_err(|e|e.to_string())?;}
 Ok(format!("{h:016x}"))
}
fn valid_name(n:&str)->bool{n.starts_with("save-")&&n.len()<90&&n[5..].bytes().all(|v|v.is_ascii_digit()||v==b'-')}
fn create_at(base:&Path,source:&Path)->Result<Summary,String>{
 if !local(source)||!safe_meta(source)?.is_dir(){return Err("Select a local save directory".into());}
 let source=fs::canonicalize(source).map_err(|e|e.to_string())?;
 fs::create_dir_all(base).map_err(|e|e.to_string())?;safe_meta(base)?;
 let base=fs::canonicalize(base).map_err(|e|e.to_string())?;
 if base.starts_with(&source)||source.starts_with(&base){return Err("Source and backup vault must not overlap".into());}
 if source.parent().is_none(){return Err("Do not select an entire drive".into());}
 let mut files=Vec::new();let mut bytes=0;scan(&source,&source,0,&mut files,&mut bytes,&mut 0)?;
 if files.is_empty(){return Err("No files in selected directory".into());}
 files.sort();
 let name=format!("save-{}",crate::media_tools::stamp());let tmp=base.join(format!("{name}.partial"));let dest=base.join(&name);
 fs::create_dir(&tmp).map_err(|e|e.to_string())?;
 let result=(||{
  let mut records=Vec::new();
  for (p,n) in files{let checksum=copy_checked(&source.join(&p),Some(&tmp.join("data").join(&p)),n)?;records.push(FileRecord{path:p,bytes:n,checksum});}
  let manifest=Manifest{version:1,files:records};
  crate::settings::atomic_json_replace(&tmp.join("manifest.json"),&serde_json::to_vec(&manifest).map_err(|e|e.to_string())?)?;
  verify_at(&tmp,&manifest)?;
  fs::rename(&tmp,&dest).map_err(|e|e.to_string())?;
  Ok(Summary{name,files:manifest.files.len(),bytes})
 })();if result.is_err(){let _=fs::remove_dir_all(&tmp);}result
}
fn read_manifest(dir:&Path)->Result<Manifest,String>{
 if !safe_meta(dir)?.is_dir(){return Err("Invalid backup directory".into());}
 let p=dir.join("manifest.json");if safe_meta(&p)?.len()>2*1024*1024{return Err("Manifest too large".into());}
 let raw=fs::read(p).map_err(|e|e.to_string())?;
 let v:Manifest=serde_json::from_slice(&raw).map_err(|e|e.to_string())?;
 let mut seen=std::collections::BTreeSet::new();let mut bytes=0u64;
 if v.version!=1||v.files.is_empty()||v.files.len()>MAX_FILES{return Err("Invalid backup manifest".into());}
 for f in &v.files{
  bytes=bytes.checked_add(f.bytes).ok_or("Overflow")?;
  if !safe_relative(&f.path)||!seen.insert(f.path.to_lowercase())||bytes>MAX_BYTES||f.checksum.len()!=16{return Err("Unsafe backup manifest".into());}
 }Ok(v)
}
fn checked_path(base:&Path,rel:&str)->Result<PathBuf,String>{
 safe_meta(base)?;let mut p=base.to_path_buf();
 for part in rel.split('/'){p.push(part);safe_meta(&p)?;}Ok(p)
}
fn verify_at(dir:&Path,v:&Manifest)->Result<(),String>{
 for f in &v.files{let p=checked_path(&dir.join("data"),&f.path)?;if copy_checked(&p,None,f.bytes)?!=f.checksum{return Err("Backup checksum mismatch".into());}}Ok(())
}
pub fn create(source:&Path)->Result<Summary,String>{create_at(&root(),source)}
pub fn list()->Vec<String>{
 let mut v=Vec::new();if let Ok(entries)=fs::read_dir(root()){for e in entries.flatten().take(500){if let Some(n)=e.file_name().to_str(){if valid_name(n)&&safe_meta(&e.path()).is_ok_and(|m|m.is_dir()){v.push(n.to_owned());}}}}
 v.sort_by(|a,b|b.cmp(a));v.truncate(100);v
}
fn verify_named(base:&Path,name:&str)->Result<Summary,String>{
 if !valid_name(name){return Err("Invalid backup name".into());}
 let dir=base.join(name);let v=read_manifest(&dir)?;verify_at(&dir,&v)?;
 Ok(Summary{name:name.into(),files:v.files.len(),bytes:v.files.iter().map(|f|f.bytes).sum()})
}
pub fn verify(name:&str)->Result<Summary,String>{verify_named(&root(),name)}
fn restore_at(base:&Path,name:&str,parent:&Path)->Result<PathBuf,String>{
 verify_named(base,name)?;
 if !local(parent)||!safe_meta(parent)?.is_dir(){return Err("Select an existing local destination directory".into());}
 let parent=fs::canonicalize(parent).map_err(|e|e.to_string())?;
 let base=fs::canonicalize(base).map_err(|e|e.to_string())?;
 if parent.starts_with(&base){return Err("Cannot restore into the backup vault".into());}
 let dir=base.join(name);let v=read_manifest(&dir)?;
 let dest=parent.join(format!("HorizonRestored-{}",crate::media_tools::stamp()));
 fs::create_dir(&dest).map_err(|e|e.to_string())?;
 let result=(||{
 for f in &v.files{let p=checked_path(&dir.join("data"),&f.path)?;if copy_checked(&p,Some(&dest.join(&f.path)),f.bytes)?!=f.checksum{return Err("Restore checksum mismatch".into());}}Ok(dest.clone())
 })();if result.is_err(){let _=fs::remove_dir_all(&dest);}result
}
pub fn restore(name:&str,parent:&Path)->Result<PathBuf,String>{restore_at(&root(),name,parent)}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn real_file_backup_verify_restore_and_corruption(){
  let temp=std::env::temp_dir().join(format!("hb-vault-test-{}",crate::media_tools::stamp()));
  let source=temp.join("source");let base=temp.join("vault");let target=temp.join("restore");
  fs::create_dir_all(source.join("nested")).unwrap();fs::create_dir_all(&target).unwrap();
  fs::write(source.join("nested/save.dat"),b"game data").unwrap();fs::write(source.join("other"),b"other").unwrap();
  let s=create_at(&base,&source).unwrap();assert_eq!(s.files,2);assert_eq!(verify_named(&base,&s.name).unwrap().bytes,14);
  let p=restore_at(&base,&s.name,&target).unwrap();assert_eq!(fs::read(p.join("nested/save.dat")).unwrap(),b"game data");
  fs::write(base.join(&s.name).join("data/other"),b"wrong").unwrap();assert!(verify_named(&base,&s.name).is_err());
  assert!(restore_at(&base,&s.name,&target).is_err());assert_eq!(fs::read(source.join("other")).unwrap(),b"other");
  fs::remove_dir_all(temp).unwrap();
 }
 #[test]fn invalid_paths_rejected(){for s in ["../x","x/../z","C:/x","x\\y","NUL.txt","a/","x:ads","x."]{assert!(!safe_relative(s),"{s}");}assert!(safe_relative("world/level.dat"));}
}
