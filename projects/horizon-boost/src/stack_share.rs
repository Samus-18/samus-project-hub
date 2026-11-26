use serde::{Deserialize,Serialize};
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Share{format:String,stages:Vec<usize>}
pub fn export(stages:&[usize])->Result<String,String>{
 crate::native_stack::validate(stages)?;
 serde_json::to_string_pretty(&Share{format:"HorizonBoost.NativeStack.1".into(),stages:stages.to_vec()}).map_err(|e|e.to_string())
}
pub fn import(s:&str)->Result<Vec<usize>,String>{
 if s.len()>8192{return Err("Stack JSON too large".into());}
 let share:Share=serde_json::from_str(s).map_err(|e|format!("Invalid stack JSON: {e}"))?;
 if share.format!="HorizonBoost.NativeStack.1"{return Err("Unsupported format".into());}
 crate::native_stack::validate(&share.stages)?;Ok(share.stages)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn ordered_roundtrip_and_strict_validation(){
  let s=export(&[39,40,39,43]).unwrap();assert_eq!(import(&s).unwrap(),vec![39,40,39,43]);
  assert!(import(r#"{"format":"HorizonBoost.NativeStack.1","stages":[999]}"#).is_err());
  assert!(import(r#"{"format":"HorizonBoost.NativeStack.1","stages":[39],"command":"run"}"#).is_err());
 }
}
