//! Local game library organization.
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Meta{pub tags:String,pub collection:String,pub favorite:bool,pub status:u8,pub rating:u8,pub checklist:Vec<crate::game_journal::Entry>,pub workspace:u8}
impl Meta{
 pub fn sanitize(&mut self){self.tags=self.tags.chars().take(160).collect();self.collection=self.collection.chars().take(80).collect();self.status=self.status.min(3);self.workspace=self.workspace.min(3);self.rating=self.rating.min(10);self.checklist.truncate(20);for v in &mut self.checklist{v.name=v.name.chars().take(80).collect();v.note=v.note.chars().take(1600).collect();}}
 pub fn matches(&self,name:&str,q:&str)->bool{format!("{name} {} {}",self.tags,self.collection).to_lowercase().contains(&q.to_lowercase())}
}
pub fn entry_key(e:&crate::game_library::Entry)->String{e.executable.as_ref().map(|p|p.to_string_lossy().into_owned()).unwrap_or_else(||format!("library:{}:{}",e.source,e.title))}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Backup{pub version:u32,pub utility:crate::utility_book::Book,pub journal:crate::game_journal::Book,pub meta:Meta,pub note:String,pub clips:std::collections::BTreeMap<String,crate::clip_book::Clip>}
impl Backup{
 pub fn from(s:&crate::settings::Settings,key:&str)->Self{let clips=s.clip_annotations.iter().filter(|(_,v)|v.game==key).map(|(k,v)|{let mut v=v.clone();v.game.clear();(k.clone(),v)}).collect();Self{version:1,utility:s.utility_profiles.get(key).cloned().unwrap_or_default(),journal:s.journal_profiles.get(key).cloned().unwrap_or_default(),meta:s.library_meta.get(key).cloned().unwrap_or_default(),note:s.game_notes.get(key).cloned().unwrap_or_default(),clips}}
 pub fn decode(raw:&str)->Result<Self,String>{if raw.len()>2*1024*1024{return Err("Backup exceeds 2 MiB".into());}let mut b:Self=serde_json::from_str(raw).map_err(|e|e.to_string())?;if b.version!=1{return Err("Unsupported backup version".into());}b.utility.sanitize();b.journal.sanitize();b.meta.sanitize();b.note=b.note.chars().take(4000).collect();crate::clip_book::sanitize_all(&mut b.clips);Ok(b)}
 pub fn apply(self,s:&mut crate::settings::Settings,key:&str){s.utility_profiles.insert(key.into(),self.utility);s.journal_profiles.insert(key.into(),self.journal);s.library_meta.insert(key.into(),self.meta);s.game_notes.insert(key.into(),self.note);for (k,mut v)in self.clips{if s.clip_annotations.len()>=2000&&!s.clip_annotations.contains_key(&k){break;}if s.clip_annotations.get(&k).is_none_or(|x|x.game==key){v.game=key.into();s.clip_annotations.insert(k,v);}}}
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn library_meta_search_and_limits(){let mut m=Meta{rating:99,status:99,tags:"RPG".into(),..Default::default()};m.sanitize();assert_eq!((m.rating,m.status),(10,3));assert!(m.matches("Game","rpg"));}
 #[test]fn local_backup_roundtrip_preserves_other_game(){let mut s=crate::settings::Settings::default();s.game_notes.insert("a".into(),"note".into());let b=Backup::from(&s,"a");let raw=serde_json::to_string(&b).unwrap();let mut dst=crate::settings::Settings::default();dst.game_notes.insert("other".into(),"other note".into());Backup::decode(&raw).unwrap().apply(&mut dst,"b");assert_eq!(dst.game_notes["b"],"note");assert_eq!(dst.game_notes["other"],"other note");}
}
