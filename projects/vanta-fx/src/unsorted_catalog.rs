//! Read-only source of truth: original 200 roadmap entries, unchanged IDs.
use serde::Deserialize;
use std::sync::OnceLock;
#[derive(Clone,Debug,Deserialize)]
pub struct Entry{
 pub id:String,
 pub request:String,
 pub status:String,
 pub workspace:String,
}
pub const AREAS:[(&str,&str);4]=[
 ("GPU Studio","Grafika / filtry"),
 ("FPS & Stability","FPS / stabilita"),
 ("Game Extras","Herní nástroje"),
 ("Overview","Přehled a správa"),
];
pub fn entries()->&'static [Entry]{
 static ITEMS:OnceLock<Vec<Entry>>=OnceLock::new();
 ITEMS.get_or_init(||serde_json::from_str(include_str!("../assets/unsorted_roadmap_200.json"))
 .expect("Bundled original roadmap JSON is invalid")).as_slice()
}
pub fn counts()->[usize;4]{
 let mut c=[0;4];for v in entries(){
  let i=match v.status.as_str(){
   "INTEGRATED_NOT_GAME_VERIFIED"=>0,
   "PARTIAL"=>1,
   "LEGACY_ONLY_NOT_NATIVE"=>2,
   "PLANNED_NOT_IMPLEMENTED"=>3,
   _=>continue
  };c[i]+=1;
 }c
}
pub fn status_label(s:&str)->&'static str{
 match s{
  "INTEGRATED_NOT_GAME_VERIFIED"=>"Kód integrován (hra neověřena)",
  "PARTIAL"=>"Rozpracováno",
  "LEGACY_ONLY_NOT_NATIVE"=>"Starší verze / neportováno",
  "PLANNED_NOT_IMPLEMENTED"=>"Chybí",
  _=>"Neznámý stav",
 }
}
pub fn area_index(s:&str)->usize{
 match s{"GPU Studio"=>0,"FPS & Stability"=>1,"Game Extras"=>2,_=>3}
}
pub fn matching<'a>(items:&'a [Entry],filter:&str,state:u8)->Vec<&'a Entry>{
 let query=crate::command_search::fold(filter);
 items.iter().filter(|p|{
  (state==0||state==1&&p.status=="INTEGRATED_NOT_GAME_VERIFIED"
   ||state==2&&p.status=="PARTIAL"
   ||state==3&&matches!(p.status.as_str(),"PLANNED_NOT_IMPLEMENTED"|"LEGACY_ONLY_NOT_NATIVE"))&&
  (query.is_empty()||crate::command_search::fold(&format!("{} {} {} {}",p.id,p.request,p.workspace,p.status)).contains(&query))
 }).collect()
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn roadmap_ids_have_unique_exact_200_and_consistent_totals(){
  let a=entries();assert_eq!(a.len(),200);let mut ids=std::collections::HashSet::new();
  for v in a{assert!(ids.insert(v.id.clone()));assert!(v.id.len()==3&&v.id.chars().all(|c|c.is_ascii_digit()));assert!((1..=200).contains(&v.id.parse::<usize>().unwrap()));assert!(AREAS.iter().any(|x|x.0==v.workspace));assert_ne!(status_label(&v.status),"Neznámý stav");}
  assert_eq!(counts(),[180,17,1,2]);assert_eq!(counts().iter().sum::<usize>(),200);
 }
 #[test]fn all_states_searchable_and_missing_never_shown_as_integrated(){
  assert_eq!(matching(entries(),"",0).len(),200);
  assert_eq!(matching(entries(),"",1).len(),180);
  assert_eq!(matching(entries(),"",2).len(),17);
  assert_eq!(matching(entries(),"",3).len(),3);
  assert_eq!(matching(entries(),"010",1).len(),0);
  assert_eq!(matching(entries(),"010",3).len(),1);
  assert_eq!(matching(entries(),"200",0).len(),1);
 }
}

pub fn exposes(page:crate::Page,staged:u8,expected:crate::Page)->bool{
 page==expected||(page==crate::Page::Unsorted&&match expected{
  crate::Page::Extras=>staged==4,
  crate::Page::Capture=>staged==6,
  _=>false
 })
}
#[cfg(test)]mod staging_tests{
 use super::*;
 #[test]fn active_staging_tool_context_keeps_capture_and_extras_working(){
  use crate::Page;
  assert!(exposes(Page::Unsorted,4,Page::Extras));
  assert!(exposes(Page::Unsorted,6,Page::Capture));
  assert!(!exposes(Page::Unsorted,4,Page::Capture));
  assert!(!exposes(Page::Unsorted,255,Page::Extras));
  assert!(!exposes(Page::Unsorted,6,Page::Extras));
  assert!(exposes(Page::Capture,255,Page::Capture));
 }
}
