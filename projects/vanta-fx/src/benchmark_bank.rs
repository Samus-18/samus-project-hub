use crate::perf_insights::FrameHealth;
#[derive(Clone,Debug,serde::Serialize)]pub struct Run{pub label:String,pub health:FrameHealth,pub low_1:Option<f32>,pub low_01:Option<f32>,#[serde(skip)]pub source:String}
#[derive(Default)]pub struct Bank{pub rows:Vec<Run>}
impl Bank{
 pub fn push(&mut self,source:&str,label:&str,health:FrameHealth,low_1:Option<f32>,low_01:Option<f32>)->Result<(),String>{
  if source.is_empty()||health.count<45{return Err("Fresh selected game benchmark required".into());}
  let valid=|v:Option<f32>|v.filter(|v|v.is_finite()&&*v>0.);
  if self.rows.len()>=8{self.rows.remove(0);}
  self.rows.push(Run{source:source.into(),label:label.chars().filter(|c|!c.is_control()).take(32).collect(),health,low_1:valid(low_1),low_01:valid(low_01)});Ok(())
 }
 pub fn delta(&self,a:usize,b:usize)->Option<[f32;3]>{
  let a=self.rows.get(a)?;let b=self.rows.get(b)?;
  if a.source!=b.source{return None;}
  Some([b.health.mean_ms-a.health.mean_ms,b.health.p95_ms-a.health.p95_ms,b.health.p99_ms-a.health.p99_ms])
 }
 pub fn json(&self)->Result<String,String>{serde_json::to_string_pretty(&serde_json::json!({"format":"HorizonBoost.Benchmark.1","source":"Real ETW sample windows; not controlled whole-game benchmark","runs":self.rows})).map_err(|e|e.to_string())}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn bounded_same_game_and_private_export(){let mut b=Bank::default();let h=FrameHealth::from_ms(&[16.;60]).unwrap();let q=FrameHealth::from_ms(&[20.;60]).unwrap();b.push("PRIVATE_EXE","A",h.clone(),None,None).unwrap();b.push("PRIVATE_EXE","B",q,None,Some(f32::NAN)).unwrap();assert_eq!(b.delta(0,1),Some([4.,4.,4.]));assert!(b.rows[1].low_01.is_none());b.push("Other","C",h.clone(),None,None).unwrap();assert!(b.delta(0,2).is_none());for _ in 0..10{b.push("Other","D",h.clone(),None,None).unwrap();}assert_eq!(b.rows.len(),8);assert!(!b.json().unwrap().contains("PRIVATE_EXE"));}
}
