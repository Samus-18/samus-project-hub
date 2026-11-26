use serde::{Serialize,Deserialize};
use std::collections::BTreeMap;
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Layers{
 pub strength:BTreeMap<usize,f32>,
 pub split:Option<f32>,
 pub ordered_ids:Vec<usize>,pub ordered_mix:Vec<f32>,
 pub groups:[bool;4],pub protect:Protection,
}
impl Default for Layers{fn default()->Self{Self{strength:Default::default(),split:None,ordered_ids:Vec::new(),ordered_mix:Vec::new(),groups:[true;4],protect:Default::default()}}}
pub const GROUPS:[&str;4]=["Color & tone","Light & shadow","Detail & cleanup","Lens & stylization"];
pub fn group(id:usize)->usize{match id{
  0|1|4|5|6|7|8|9|11|31|41|44|46|50|52|53|57=>0,
  12|13|14|15|16|21|28|29|30|33|34|56|60=>1,
  2|32|35|37|38|39|40|47|54|55|61=>2,_=>3}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]pub struct Protection{pub rectangle:bool,pub rect:[f32;4],pub edge_strength:f32,pub edge_threshold:f32}
impl Default for Protection{fn default()->Self{Self{rectangle:false,rect:[0.,0.82,1.,1.],edge_strength:0.,edge_threshold:0.18}}}
impl Protection{
 pub fn clamp(&mut self){let d=Self::default();for(i,v)in self.rect.iter_mut().enumerate(){*v=if v.is_finite(){v.clamp(0.,1.)}else{d.rect[i]};}self.rect[0]=self.rect[0].min(0.95);self.rect[1]=self.rect[1].min(0.95);self.rect[2]=self.rect[2].max(self.rect[0]+0.05);self.rect[3]=self.rect[3].max(self.rect[1]+0.05);
 self.edge_strength=if self.edge_strength.is_finite(){self.edge_strength.clamp(0.,1.)}else{0.};
 self.edge_threshold=if self.edge_threshold.is_finite(){self.edge_threshold.clamp(0.02,0.8)}else{d.edge_threshold};}
 pub fn active(&self)->bool{self.rectangle||self.edge_strength>0.}
 pub fn args(&self)->[f32;8]{let mut p=self.clone();p.clamp();[p.rect[0],p.rect[1],p.rect[2],p.rect[3],if p.rectangle{1.}else{0.},p.edge_strength,p.edge_threshold,0.]}
}
impl Layers{
 pub fn clamp(&mut self){
  self.protect.clamp();
  self.strength.retain(|k,v|crate::native_stack::valid_stage(*k)&&v.is_finite());
  for v in self.strength.values_mut(){*v=v.clamp(0.,1.);}
  self.strength.retain(|_,v|*v!=1.);
  if crate::native_stack::validate(&self.ordered_ids).is_err()||self.ordered_ids.len()!=self.ordered_mix.len(){self.ordered_ids.clear();self.ordered_mix.clear();}
  for v in &mut self.ordered_mix{*v=if v.is_finite(){v.clamp(0.,1.)}else{1.};}
  self.split=self.split.filter(|v|v.is_finite()).map(|v|v.clamp(0.,1.));
 }
 pub fn enabled_at(&self,ids:&[usize],slot:usize)->f32{if ids.get(slot).is_some_and(|id|!self.groups[group(*id)]){0.}else{self.strength_at(ids,slot)}}
 pub fn strength_at(&self,ids:&[usize],slot:usize)->f32{
  if self.ordered_ids.starts_with(ids)&&let Some(v)=self.ordered_mix.get(slot){return v.clamp(0.,1.);}
  ids.get(slot).map(|id|self.strength(*id)).unwrap_or(1.)
 }
 pub fn set_at(&mut self,ids:&[usize],slot:usize,value:f32){
  if crate::native_stack::validate(ids).is_err()||slot>=ids.len(){return;}
  if self.ordered_ids!=ids{self.ordered_mix=ids.iter().map(|i|self.strength(*i)).collect();self.ordered_ids=ids.to_vec();}
  self.ordered_mix[slot]=if value.is_finite(){value.clamp(0.,1.)}else{1.};
 }
 pub fn clear_ordered(&mut self){self.ordered_ids.clear();self.ordered_mix.clear();}
 pub fn strength(&self,id:usize)->f32{
  self.strength.get(&id).copied().filter(|v|v.is_finite()).unwrap_or(1.).clamp(0.,1.)
 }
}
#[cfg(test)]mod tests{
 #[test]fn clamp_and_old_defaults(){
  let mut l:super::Layers=serde_json::from_str("{}").unwrap();
  assert_eq!(l.strength(2),1.);assert!(l.split.is_none());
  l.strength.insert(2,-2.);l.strength.insert(999,0.);l.split=Some(3.);l.clamp();
  assert_eq!(l.strength(2),0.);assert!(!l.strength.contains_key(&999));assert_eq!(l.split,Some(1.));
 }
}

#[cfg(test)]mod instance_tests{use super::*;#[test]fn duplicates_are_independent_and_prefix_safe(){let mut l=Layers::default();l.set_at(&[7,2,7],0,0.);l.set_at(&[7,2,7],2,0.4);assert_eq!(l.strength_at(&[7,2,7],2),0.4);assert_eq!(l.strength_at(&[7,2],0),0.);assert_eq!(l.strength_at(&[2,7],1),1.);let mut q:Layers=serde_json::from_str(&serde_json::to_string(&l).unwrap()).unwrap();q.clamp();assert_eq!(q,l);q.clear_ordered();assert_eq!(q.strength_at(&[7,2,7],0),1.);}}

#[cfg(test)]mod group_tests{use super::*;#[test]fn group_toggle_preserves_strength_and_old_settings(){let mut l:Layers=serde_json::from_str("{}").unwrap();assert_eq!(l.groups,[true;4]);l.set_at(&[7,2,7],0,0.4);l.groups[0]=false;assert_eq!(l.enabled_at(&[7,2,7],0),0.);assert_eq!(l.enabled_at(&[7,2,7],1),1.);l.groups[0]=true;assert_eq!(l.enabled_at(&[7,2,7],0),0.4);l.protect.rect=[1.,1.,0.,0.];l.clamp();assert_eq!(l.protect.rect,[0.95,0.95,1.,1.]);}}

#[cfg(test)]mod v133_stage_strength_tests{
 use super::*;
 #[test]fn duplicate_stage_slot_weights_and_reset(){
  let mut x=Layers::default();
  let ids=[2usize,2usize,4usize];
  for i in 0..ids.len(){x.set_at(&ids,i,0.35);}
  for i in 0..ids.len(){assert!((x.strength_at(&ids,i)-0.35).abs()<0.0001);}
  x.set_at(&ids,1,0.75);
  assert!((x.strength_at(&ids,0)-0.35).abs()<0.0001);
  assert!((x.strength_at(&ids,1)-0.75).abs()<0.0001);
  x.set_at(&ids,1,1.);
  assert!((x.strength_at(&ids,1)-1.).abs()<0.0001);
 }
}
