//! Explicit local manifest import; never execute manifest-provided commands.
use serde::{Serialize,Deserialize};
use std::{path::{Path,PathBuf},fs,io::Read,process::Command};
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq)]
pub struct Entry{pub title:String,pub executable:Option<PathBuf>,pub source:String,#[serde(default)]pub banner:Option<String>}
fn title(raw:&str)->String{raw.trim().chars().filter(|c|!c.is_control()).take(100).collect()}
impl Entry{
 pub fn manual(name:&str,path:PathBuf)->Result<Self,String>{
  let name=title(name);
  if name.is_empty(){return Err("Enter a game name".into());}
  check_exe(&path)?;
  Ok(Self{title:name,executable:Some(path),source:"Manual".into(),banner:None})
 }
 pub fn valid(&self)->bool{!self.title.is_empty()&&self.title==title(&self.title)&&self.source.len()<=24&&!self.source.chars().any(char::is_control)&&self.banner.as_ref().is_none_or(|n|valid_banner(n))&&self.executable.as_ref().is_none_or(|p|p.is_absolute()&&p.extension().is_some_and(|x|x.eq_ignore_ascii_case("exe")))}
}
fn check_exe(path:&Path)->Result<(),String>{
 if !path.is_absolute()||!path.extension().is_some_and(|x|x.eq_ignore_ascii_case("exe"))||!crate::launcher_discovery::safe_file(path){return Err("Choose an existing local .exe without reparse points".into());}
 // Refuse remote UNC paths and Windows device namespaces.
 if path.to_string_lossy().starts_with("\\\\"){return Err("Remote or device paths are not supported".into());}
 Ok(())
}
pub fn insert(entries:&mut Vec<Entry>,entry:Entry)->Result<(),String>{
 if !entry.valid(){return Err("Invalid library entry".into());}
 if entries.len()>=200{return Err("Library is limited to 200 entries".into());}
 if entries.iter().any(|v|v.title.eq_ignore_ascii_case(&entry.title)&&match (&v.executable,&entry.executable){(Some(a),Some(b))=>a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy()),(None,None)=>true,_=>false}){return Err("Entry already exists".into());}
 entries.push(entry);Ok(())
}
pub fn launch(entry:&Entry)->Result<(),String>{
 let p=entry.executable.as_ref().ok_or("Choose the game's actual executable first")?;
 check_exe(p)?;
 // No shell, arguments, elevation, injection, environment override or forced window mode.
 Command::new(p).current_dir(p.parent().ok_or("Executable directory missing")?).spawn().map(|_|()).map_err(|e|e.to_string())
}
pub(crate) fn steam_tokens(s:&str)->Result<Vec<String>,String>{
 let mut chars=s.chars().peekable();let mut out=Vec::new();
 while let Some(ch)=chars.next(){
  if ch.is_whitespace(){continue;}
  if ch=='/'&&chars.peek()==Some(&'/'){chars.next();while chars.next().is_some_and(|c|c!='\n'){}continue;}
  if ch=='{'||ch=='}'{out.push(ch.to_string());}
  else if ch=='"'{
   let mut v=String::new();let mut closed=false;
   while let Some(c)=chars.next(){match c{
    '"'=>{closed=true;break;},
    '\\'=>match chars.next(){Some('"')=>v.push('"'),Some('\\')=>v.push('\\'),Some(x)=>{v.push('\\');v.push(x);},None=>break},
    x=>v.push(x)
   }if v.len()>4096{return Err("Manifest token too long".into());}}
   if !closed{return Err("Unterminated manifest string".into());}out.push(v);
  }else{return Err("Unsupported Steam manifest token".into());}
  if out.len()>12000{return Err("Manifest too complex".into());}
 }
 Ok(out)
}
pub fn steam(raw:&str)->Result<Entry,String>{
 let t=steam_tokens(raw.trim_start_matches('\u{feff}'))?;
 if t.first().map(String::as_str)!=Some("AppState")||t.get(1).map(String::as_str)!=Some("{"){return Err("Not a Steam AppState manifest".into());}
 let mut i=2;let mut depth=1;let mut name=None;
 while i<t.len()&&depth>0{
  match t[i].as_str(){
   "{"=>{depth+=1;i+=1;},
   "}"=>{depth-=1;i+=1;},
   key=>{let v=t.get(i+1).ok_or("Truncated manifest")?;
    if v=="}"{return Err("Missing manifest value".into());}
    if depth==1&&key.eq_ignore_ascii_case("name")&&v!="{"{name=Some(title(v));}
    if v=="{"{depth+=1;}i+=2;
   }
  }
 }
 if depth!=0||i!=t.len(){return Err("Unbalanced manifest".into());}
 let name=name.filter(|v|!v.is_empty()).ok_or("Missing Steam game name")?;
 Ok(Entry{title:name,executable:None,source:"Steam manifest".into(),banner:None})
}
pub fn epic(raw:&str)->Result<Entry,String>{
 let v:serde_json::Value=serde_json::from_str(raw.trim_start_matches('\u{feff}')).map_err(|e|e.to_string())?;
 let name=v.get("DisplayName").and_then(|v|v.as_str()).map(title).filter(|s|!s.is_empty()).ok_or("Missing Epic DisplayName")?;
 if !v.get("InstallLocation").is_some_and(|v|v.is_string()){return Err("Not an Epic install manifest".into());}
 Ok(Entry{title:name,executable:None,source:"Epic manifest".into(),banner:None})
}
pub fn gog(raw:&str)->Result<Entry,String>{
 let v:serde_json::Value=serde_json::from_str(raw.trim_start_matches('\u{feff}')).map_err(|e|e.to_string())?;
 let id=v.get("gameId").ok_or("Missing GOG gameId")?;
 if !(id.as_u64().is_some()||id.as_str().is_some_and(|s|!s.is_empty()&&s.len()<=32&&s.bytes().all(|v|v.is_ascii_digit()))){return Err("Invalid GOG gameId".into());}
 let name=v.get("name").and_then(|v|v.as_str()).map(title).filter(|s|!s.is_empty()).ok_or("Missing GOG name")?;
 Ok(Entry{title:name,executable:None,source:"GOG manifest".into(),banner:None})
}
fn manifest_kind(path:&Path)->Option<&'static str>{
 let name=path.file_name()?.to_str()?.to_ascii_lowercase();
 if name.ends_with(".acf"){Some("steam")}else if name.ends_with(".item"){Some("epic")}
 else if name.strip_prefix("goggame-").and_then(|n|n.strip_suffix(".info")).is_some_and(|id|!id.is_empty()&&id.bytes().all(|b|b.is_ascii_digit())){Some("gog")}else{None}
}
pub fn read_manifest(path:&Path)->Result<Entry,String>{
 if !path.is_absolute()||path.to_string_lossy().starts_with("\\")||!crate::launcher_discovery::safe_file(path){return Err("Select an existing local manifest file".into());}
 let mut data=String::new();fs::File::open(path).map_err(|e|e.to_string())?.take(1024*1024+1).read_to_string(&mut data).map_err(|e|e.to_string())?;
 if data.len()>1024*1024{return Err("Manifest exceeds 1 MiB".into());}
 match manifest_kind(path){
 Some("steam")=>steam(&data),Some("epic")=>epic(&data),Some("gog")=>gog(&data),_=>Err("Choose Steam .acf, Epic .item or GOG goggame-ID.info".into())
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn steam_nested_name_not_executable(){
  let e=steam(r#""AppState" {"name" "Game \"Deluxe\"" "InstalledDepots" {"123" {"name" "Wrong"}} "appid" "123"}"#).unwrap();
  assert_eq!(e.title,"Game \"Deluxe\"");assert!(e.executable.is_none());
  assert!(steam(r#""AppState" {"name" "A""#).is_err());
 }
 #[test]fn epic_never_trusts_command(){
  let e=epic(r#"{"DisplayName":"Example","InstallLocation":"C:\\Game","LaunchExecutable":"cmd.exe","LaunchCommand":"bad"}"#).unwrap();
  assert_eq!(e.title,"Example");assert!(e.executable.is_none());assert!(launch(&e).is_err());
 }
 #[test]fn bounded_entries_and_duplicate(){
  let e=Entry{title:"Test".into(),executable:None,source:"Steam manifest".into(),banner:None};
  let mut v=Vec::new();insert(&mut v,e.clone()).unwrap();assert!(insert(&mut v,e.clone()).is_err());
  let raw=serde_json::to_string(&v).unwrap();assert_eq!(serde_json::from_str::<Vec<Entry>>(&raw).unwrap(),v);
  assert!(check_exe(Path::new("cmd.exe")).is_err());
  assert!(check_exe(Path::new(r"\\server\share\app.exe")).is_err());
 }
}

#[derive(Default)]pub struct Scan{pub entries:Vec<Entry>,pub rejected:usize,pub inspected:usize,pub folders:usize,pub truncated:bool}
pub fn scan_dir(dir:&Path)->Result<Scan,String>{
 if !crate::launcher_discovery::safe_dir(dir){return Err("Select a local manifest directory without junctions".into());}
 let mut result=Scan{folders:1,..Default::default()};
 for (index,entry) in fs::read_dir(dir).map_err(|e|e.to_string())?.take(1001).enumerate(){
  if index==1000{result.truncated=true;break;}
  let Ok(entry)=entry else{result.rejected+=1;continue;};
  if !entry.file_type().is_ok_and(|v|v.is_file()){continue;}
  let p=entry.path();if manifest_kind(&p).is_none(){continue;}
  result.inspected+=1;
  match read_manifest(&p){Ok(e)=>{if insert(&mut result.entries,e).is_err(){result.rejected+=1;}},Err(_)=>result.rejected+=1}
 }
 Ok(result)
}
fn valid_banner(n:&str)->bool{n.starts_with("banner-")&&n.ends_with(".png")&&n.len()>11&&n.len()<90&&n[7..n.len()-4].bytes().all(|v|v.is_ascii_digit()||v==b'-')}
pub fn load_banner(n:&str)->Result<image::RgbaImage,String>{
 if !valid_banner(n){return Err("Invalid banner name".into());}
 let p=crate::settings::folder().join("banners").join(n);
 if !crate::launcher_discovery::safe_file(&p)||fs::metadata(&p).map_err(|e|e.to_string())?.len()>4*1024*1024{return Err("Invalid local banner file".into());}
 let mut reader=image::ImageReader::open(p).map_err(|e|e.to_string())?;
 let mut limits=image::Limits::default();limits.max_image_width=Some(512);limits.max_image_height=Some(288);limits.max_alloc=Some(4*1024*1024);reader.limits(limits);
 Ok(reader.decode().map_err(|e|e.to_string())?.to_rgba8())
}
pub fn import_banner(path:&Path)->Result<String,String>{
 if !crate::launcher_discovery::safe_file(path){return Err("Choose a local image without reparse points".into());}
 if fs::metadata(path).map_err(|e|e.to_string())?.len()>16*1024*1024{return Err("Banner exceeds 16 MiB".into());}
 let mut reader=image::ImageReader::open(path).map_err(|e|e.to_string())?.with_guessed_format().map_err(|e|e.to_string())?;
 let mut limits=image::Limits::default();limits.max_image_width=Some(4096);limits.max_image_height=Some(4096);limits.max_alloc=Some(96*1024*1024);reader.limits(limits);
 let img=reader.decode().map_err(|e|e.to_string())?.resize_to_fill(512,288,image::imageops::FilterType::Lanczos3);
 let dir=crate::settings::folder().join("banners");fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
 let name=format!("banner-{}.png",crate::media_tools::stamp());let mut bytes=std::io::Cursor::new(Vec::new());
 img.write_to(&mut bytes,image::ImageFormat::Png).map_err(|e|e.to_string())?;
 crate::settings::atomic_json_replace(&dir.join(&name),&bytes.into_inner())?;Ok(name)
}
#[cfg(test)]mod batch_tests{
 use super::*;
 #[test]fn batch_import_fixture(){let p=std::env::temp_dir().join(format!("hb-scan-{}",crate::media_tools::stamp()));fs::create_dir(&p).unwrap();
 fs::write(p.join("a.acf"),r#""AppState" {"name" "A"}"#).unwrap();fs::write(p.join("b.item"),r#"{"DisplayName":"B","InstallLocation":"x"}"#).unwrap();fs::write(p.join("bad.acf"),"bad").unwrap();fs::write(p.join("skip.txt"),"ignored").unwrap();
 let s=scan_dir(&p).unwrap();assert_eq!(s.entries.len(),2);assert_eq!(s.rejected,1);assert_eq!(s.inspected,3);fs::remove_dir_all(p).unwrap();}
 #[test]fn old_library_compatible_and_banner_names(){let e:Entry=serde_json::from_str(r#"{"title":"A","executable":null,"source":"Manual"}"#).unwrap();assert!(e.valid()&&e.banner.is_none());assert!(!valid_banner("../a.png"));assert!(valid_banner("banner-123-2-0.png"));}
}

#[cfg(test)]mod gog_tests{use super::*;#[test]fn gog_is_metadata_only(){let e=gog(r#"{"name":"GOG Example","gameId":"123","playTasks":[{"path":"bad.exe"}]}"#).unwrap();assert_eq!(e.source,"GOG manifest");assert!(e.executable.is_none());assert!(gog(r#"{"name":"Not a game","gameId":"oops"}"#).is_err());assert_eq!(manifest_kind(Path::new("goggame-123.info")),Some("gog"));assert_eq!(manifest_kind(Path::new("other.info")),None);}}

#[cfg(test)]mod numeric_gog_tests{
 #[test]fn numeric_id_and_wrong_path(){assert!(super::gog(r#"{"name":"A","gameId":123}"#).is_ok());assert!(super::gog(r#"{"name":"A","gameId":-123}"#).is_err());assert!(super::read_manifest(std::path::Path::new("goggame-1.info")).is_err());}
}

pub fn launch_proof()->Result<String,String>{
 let dir=std::env::current_dir().map_err(|e|e.to_string())?.join("build-tests").join("launch-fixture");
 let exe=dir.join("fixture.exe");let ok=dir.join("launch-ok.txt");
 if ok.exists(){fs::remove_file(&ok).map_err(|e|e.to_string())?;}
 let e=Entry::manual("Own test fixture",exe)?;launch(&e)?;
 let end=std::time::Instant::now()+std::time::Duration::from_secs(5);
 while std::time::Instant::now()<end{
  if fs::read_to_string(&ok).ok().as_deref()==Some("OWN_FIXTURE_NO_ARGS"){return Ok("LIBRARY_LAUNCH=PASS OWN_FIXTURE=true WORKDIR_EXACT=true NO_GAME_LAUNCH=true NO_SHELL=true".into());}
  std::thread::sleep(std::time::Duration::from_millis(25));
 }
 Err("Own fixture launch did not produce expected sentinel".into())
}
