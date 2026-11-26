//! Local clip annotations, no media mutation or network.
use serde::{Serialize,Deserialize};
use std::collections::BTreeMap;
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Marker{pub ms:u64,pub label:String}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Clip{pub title:String,pub tags:String,pub collection:String,pub game:String,pub favorite:bool,pub markers:Vec<Marker>}
pub fn key(path:&std::path::Path)->Option<String>{
 let name=path.file_name()?.to_str()?;
 if !name.starts_with("HBoost_")||name.len()>240||!matches!(path.extension()?.to_str()?.to_ascii_lowercase().as_str(),"mp4"|"mkv"|"png"){return None;}
 Some(name.to_owned())
}
impl Clip{
 pub fn sanitize(&mut self){
  for (v,n) in [(&mut self.title,120),(&mut self.tags,240),(&mut self.collection,80),(&mut self.game,300)]{*v=v.chars().take(n).collect();}
  self.markers.truncate(128);for v in &mut self.markers{v.ms=v.ms.min(86_400_000);v.label=v.label.chars().take(100).collect();}
  self.markers.sort_by_key(|v|v.ms);
 }
 pub fn add(&mut self,seconds:f64,duration:f64,label:&str)->bool{
  if self.markers.len()>=128||!seconds.is_finite()||!duration.is_finite()||seconds<0.||seconds>duration||seconds>86400.{return false;}
  let label=label.trim().chars().take(100).collect::<String>();
  self.markers.push(Marker{ms:(seconds*1000.).round() as u64,label:if label.is_empty(){"Bookmark".into()}else{label}});
  self.markers.sort_by_key(|v|v.ms);true
 }
 pub fn matches(&self,filename:&str,query:&str)->bool{
  let q=query.to_lowercase();
  format!("{filename} {} {} {} {}",self.title,self.tags,self.collection,self.game).to_lowercase().contains(&q)
 }
}
pub fn sanitize_all(m:&mut BTreeMap<String,Clip>){
 m.retain(|k,v|{v.sanitize();key(std::path::Path::new(k)).as_ref()==Some(k)});
 // Bound imported settings, keep normal existing clip metadata.
 while m.len()>2000{if let Some(k)=m.keys().next_back().cloned(){m.remove(&k);}else{break;}}
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn bookmarks_are_sorted_and_bounded(){let mut c=Clip::default();assert!(c.add(4.,5.,"Late"));assert!(c.add(1.,5.,"Early"));assert!(!c.add(6.,5.,"bad"));assert!(!c.add(f64::NAN,5.,"bad"));assert_eq!(c.markers[0].ms,1000);}
 #[test]fn metadata_roundtrip_and_search(){let c=Clip{title:"Boss clear".into(),favorite:true,..Default::default()};let x:Clip=serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();assert_eq!(x,c);assert!(x.matches("HBoost_1.mp4","BOSS"));assert!(!x.matches("HBoost_1.mp4","other"));}
 #[test]fn clip_keys_restrict_local_capture_names(){assert_eq!(key(std::path::Path::new("HBoost_1.mp4")).as_deref(),Some("HBoost_1.mp4"));assert!(key(std::path::Path::new("other.exe")).is_none());}
}
