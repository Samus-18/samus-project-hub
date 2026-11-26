//! Explicit user-started local launcher manifest discovery. No executable trust or launch from metadata.
use std::{path::{Path,PathBuf,Component,Prefix},fs,io::Read,os::windows::fs::MetadataExt};
use crate::game_library::{self,Scan};
const REPARSE:u32=0x400;
fn local(p:&Path)->bool{
 matches!(p.components().next(),Some(Component::Prefix(v)) if matches!(v.kind(),Prefix::Disk(_)))&&p.is_absolute()&&!p.components().any(|v|matches!(v,Component::ParentDir))
}
fn safe_ancestors(p:&Path)->bool{
 local(p)&&p.ancestors().all(|v|fs::symlink_metadata(v).is_ok_and(|m|m.is_dir()&&m.file_attributes()&REPARSE==0))
}
pub fn safe_dir(p:&Path)->bool{safe_ancestors(p)}
pub fn safe_file(p:&Path)->bool{
 local(p)&&p.parent().is_some_and(safe_dir)&&fs::symlink_metadata(p).is_ok_and(|m|m.is_file()&&m.file_attributes()&REPARSE==0)
}
fn read_text(p:&Path)->Option<String>{
 if !safe_file(p){return None;}let mut data=String::new();
 fs::File::open(p).ok()?.take(1024*1024+1).read_to_string(&mut data).ok()?;
 (data.len()<=1024*1024).then_some(data)
}
pub fn steam_libraries(raw:&str)->Result<Vec<PathBuf>,String>{
 let t=game_library::steam_tokens(raw.trim_start_matches('\u{feff}'))?;
 if !t.first().is_some_and(|s|s.eq_ignore_ascii_case("libraryfolders"))||t.get(1).map(String::as_str)!=Some("{"){return Err("Not a Steam libraryfolders manifest".into());}
 let mut depth=1usize;let mut i=2;let mut numeric_block=false;let mut out=Vec::new();
 while i<t.len()&&depth>0{
  match t[i].as_str(){
   "}"=>{depth-=1;if depth==1{numeric_block=false;}i+=1;},
   "{"=>return Err("Unexpected opening brace".into()),
   key=>{
    let value=t.get(i+1).ok_or("Missing Steam library value")?;
    if value=="}"{return Err("Missing Steam library value".into());}
    let numeric=!key.is_empty()&&key.bytes().all(|b|b.is_ascii_digit());
    if value=="{"{if depth==1{numeric_block=numeric;}depth+=1;if depth>16{return Err("Steam library nesting exceeds limit".into());}}
    else if (depth==1&&numeric)||(depth==2&&numeric_block&&key.eq_ignore_ascii_case("path")){
     let p=PathBuf::from(value);if local(&p)&&!out.contains(&p)&&out.len()<32{out.push(p);}
    }
    i+=2;
   }
  }
 }
 if depth!=0||i!=t.len(){return Err("Unbalanced Steam libraryfolders".into());}Ok(out)
}
#[link(name="advapi32")]unsafe extern "system"{
 fn RegGetValueW(key:isize,sub:*const u16,name:*const u16,flags:u32,ty:*mut u32,data:*mut std::ffi::c_void,size:*mut u32)->i32;
}
fn steam_registry()->Option<PathBuf>{
 let sub="Software\\Valve\\Steam".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
 let name="SteamPath".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
 let mut data=[0u16;2048];let mut size=std::mem::size_of_val(&data) as u32;
 // Read-only HKCU, REG_SZ only; no registry modifications or shell.
 let status=unsafe{RegGetValueW(0x80000001u32 as i32 as isize,sub.as_ptr(),name.as_ptr(),2,std::ptr::null_mut(),data.as_mut_ptr().cast(),&mut size)};
 if status!=0||size<2||size as usize>std::mem::size_of_val(&data){return None;}
 let end=data.iter().position(|v|*v==0)?;let path=PathBuf::from(String::from_utf16(&data[..end]).ok()?);local(&path).then_some(path)
}
fn add_root(roots:&mut Vec<PathBuf>,p:PathBuf){if roots.len()<256&&local(&p)&&!roots.iter().any(|v|v.to_string_lossy().eq_ignore_ascii_case(&p.to_string_lossy())){roots.push(p);}}
fn scan_roots(roots:Vec<PathBuf>)->Scan{
 let mut out=Scan::default();
 for root in roots{
  if out.entries.len()>=200||out.inspected>=4000||out.folders>=256{out.truncated=true;break;}
  if !safe_dir(&root){continue;}
  out.folders+=1;
  let Ok(dir)=fs::read_dir(&root) else{out.rejected+=1;continue;};
  for (i,entry) in dir.take(1001).enumerate(){
   if i==1000||out.inspected>=4000||out.entries.len()>=200{out.truncated=true;break;}
   let Ok(entry)=entry else{out.rejected+=1;continue;};let p=entry.path();
   let name=entry.file_name().to_string_lossy().to_ascii_lowercase();
   if !(name.ends_with(".acf")||name.ends_with(".item")||(name.starts_with("goggame-")&&name.ends_with(".info"))){continue;}
   out.inspected+=1;
   match game_library::read_manifest(&p){Ok(e)=>{if game_library::insert(&mut out.entries,e).is_err(){out.rejected+=1;}},Err(_)=>out.rejected+=1}
  }
 }
 out
}
fn add_gog(roots:&mut Vec<PathBuf>,root:PathBuf){
 if !safe_dir(&root){return;}
 add_root(roots,root.clone());
 if let Ok(entries)=fs::read_dir(root){for e in entries.take(200).flatten(){let p=e.path();if safe_dir(&p){add_root(roots,p);}}}
}
pub fn discover()->Result<Scan,String>{
 let mut roots=Vec::new();let mut steam=Vec::new();
 if let Some(p)=steam_registry(){add_root(&mut steam,p);}
 for key in ["ProgramFiles(x86)","ProgramFiles"]{
  if let Some(v)=std::env::var_os(key){let p=PathBuf::from(v);add_root(&mut steam,p.join("Steam"));}
 }
 for base in steam{
  let apps=base.join("steamapps");add_root(&mut roots,apps.clone());
  if let Some(raw)=read_text(&apps.join("libraryfolders.vdf")){if let Ok(libs)=steam_libraries(&raw){for p in libs{add_root(&mut roots,p.join("steamapps"));}}}
 }
 if let Some(v)=std::env::var_os("ProgramData"){add_root(&mut roots,PathBuf::from(v).join("Epic/EpicGamesLauncher/Data/Manifests"));}
 for key in ["ProgramFiles(x86)","ProgramFiles"]{if let Some(v)=std::env::var_os(key){let p=PathBuf::from(v);add_gog(&mut roots,p.join("GOG Galaxy/Games"));add_gog(&mut roots,p.join("GOG Games"));}}
 if let Some(v)=std::env::var_os("SystemDrive"){add_gog(&mut roots,PathBuf::from(format!("{}\\GOG Games",v.to_string_lossy())));}
 Ok(scan_roots(roots))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn steam_paths_only_numeric_library_blocks(){
  let a=steam_libraries(r#""libraryfolders" {"0" {"path" "C:\\Steam" "apps" {"123" "1" "path" "C:\\Wrong"}} "1" "D:\\Games" "2" {"path" "\\\\server\\share"} "unrelated" {"path" "E:\\Wrong"}}"#).unwrap();
  assert_eq!(a,vec![PathBuf::from(r"C:\Steam"),PathBuf::from(r"D:\Games")]);
  assert!(steam_libraries(r#""libraryfolders" {"1" {"path" "C:\\Games"}"#).is_err());
 }
 #[test]fn bounded_explicit_roots_and_no_commands(){
  let root=std::env::temp_dir().join(format!("hb-discovery-{}",crate::media_tools::stamp()));fs::create_dir(&root).unwrap();
  fs::write(root.join("appmanifest_1.acf"),r#""AppState" {"name" "Steam Game"}"#).unwrap();
  fs::write(root.join("x.item"),r#"{"DisplayName":"Epic Game","InstallLocation":"X","LaunchExecutable":"cmd.exe"}"#).unwrap();
  fs::write(root.join("goggame-1.info"),r#"{"name":"GOG Game","gameId":1}"#).unwrap();
  fs::write(root.join("bad.item"),"invalid").unwrap();
  let s=scan_roots(vec![root.clone()]);assert_eq!(s.entries.len(),3);assert_eq!(s.inspected,4);assert_eq!(s.rejected,1);assert!(s.entries.iter().all(|v|v.executable.is_none()));assert!(!s.truncated);
  fs::remove_dir_all(root).unwrap();assert!(!safe_dir(Path::new(r"\\server\share")));
 }
}
