//! Bounded, portable node-based authoring of an ordered native D3D11 stack.
//! Edges express dependency order; fan-in is topologically scheduled, not
//! falsely presented as a multi-texture blend/compositor. No extra GPU queues.
use serde::{Serialize,Deserialize};
use std::collections::{BTreeSet,BTreeMap};
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node{pub id:u8,pub stage:usize}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize,Default)]
#[serde(deny_unknown_fields)]
pub struct Graph{pub nodes:Vec<Node>,pub edges:Vec<(u8,u8)>}
impl Graph{
 pub fn from_stages(stages:&[usize])->Self{
  let nodes=stages.iter().take(8).enumerate().map(|(i,&stage)|Node{id:i as u8,stage}).collect::<Vec<_>>();
  let edges=nodes.windows(2).map(|pair|(pair[0].id,pair[1].id)).collect();
  Self{nodes,edges}
 }
 pub fn add(&mut self,stage:usize)->Result<u8,String>{
  if self.nodes.len()>=8||!crate::native_stack::valid_stage(stage){return Err("Select at most 8 supported GPU stages".into());}
  let id=(0u8..=u8::MAX).find(|i|!self.nodes.iter().any(|n|n.id==*i)).ok_or("No available node id")?;
  self.nodes.push(Node{id,stage});Ok(id)
 }
 pub fn remove(&mut self,id:u8){
  self.nodes.retain(|n|n.id!=id);self.edges.retain(|&(a,b)|a!=id&&b!=id);
 }
 pub fn connect(&mut self,from:u8,to:u8)->Result<(),String>{
  let mut candidate=self.clone();candidate.edges.push((from,to));candidate.compile()?;
  *self=candidate;Ok(())
 }
 pub fn compile(&self)->Result<Vec<usize>,String>{
  if self.nodes.is_empty()||self.nodes.len()>8||self.edges.len()>24{return Err("Graph must contain 1-8 nodes and at most 24 connections".into());}
  let mut keys=BTreeSet::new();
  for n in &self.nodes{
   if !keys.insert(n.id)||!crate::native_stack::valid_stage(n.stage){return Err("Duplicate node id or unsupported shader".into());}
  }
  let mut dep=BTreeMap::<u8,usize>::new();for n in &self.nodes{dep.insert(n.id,0);}
  let mut links=BTreeSet::new();
  for &(from,to) in &self.edges{
   if from==to||!keys.contains(&from)||!keys.contains(&to)||!links.insert((from,to)){return Err("Invalid, self-referencing or duplicate node connection".into());}
   *dep.get_mut(&to).ok_or("Unknown destination")?+=1;
  }
  let mut out=Vec::with_capacity(self.nodes.len());
  let mut resolved=BTreeSet::new();
  while out.len()<self.nodes.len(){
   let Some(n)=self.nodes.iter().find(|n|!resolved.contains(&n.id)&&dep.get(&n.id)==Some(&0)) else{return Err("Shader graph has a cycle; remove a connection".into());};
   resolved.insert(n.id);out.push(n.stage);
   for &(from,to) in &self.edges{if from==n.id{let count=dep.get_mut(&to).ok_or("Invalid edge target")?;*count=count.saturating_sub(1);}}
  }
  crate::native_stack::validate(&out)?;Ok(out)
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn source_stack_roundtrip_and_topological_order(){
  let g=Graph::from_stages(&[37,2,38]);assert_eq!(g.compile().unwrap(),vec![37,2,38]);
  let recovered:Graph=serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();assert_eq!(g,recovered);
  let mut fan=Graph::from_stages(&[37,2,38]);fan.edges=vec![(0,2),(1,2)];
  assert_eq!(fan.compile().unwrap(),vec![37,2,38]);
 }
 #[test]fn cycle_duplicate_and_orphan_refused_without_mutating_graph(){
  let mut g=Graph::from_stages(&[37,2,38]);let original=g.clone();
  assert!(g.connect(2,0).is_err());assert_eq!(g,original);
  assert!(g.connect(2,2).is_err());assert!(g.connect(20,2).is_err());assert!(g.connect(0,1).is_err());
  g.remove(1);assert_eq!(g.nodes.len(),2);assert!(g.edges.is_empty());assert_eq!(g.compile().unwrap(),vec![37,38]);
 }
 #[test]fn bounded_nodes_and_explicit_unsupported_refused(){
  let mut g=Graph::default();assert!(g.compile().is_err());
  assert!(g.add(9999).is_err());
  for _ in 0..8{g.add(37).unwrap();}
  assert!(g.add(38).is_err());assert_eq!(g.compile().unwrap().len(),8);
 }
 #[test]fn stable_json_profile_and_settings_backwards_compatibility(){
  let mut s=crate::settings::Settings::default();let graph=Graph::from_stages(&[67,64,2]);
  s.node_graph_profiles.insert("test.exe".into(),graph.clone());
  let loaded:crate::settings::Settings=serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
  assert_eq!(loaded.node_graph_profiles["test.exe"],graph);
  let old:crate::settings::Settings=serde_json::from_str("{}").unwrap();assert!(old.node_graph_profiles.is_empty());
 }
}
