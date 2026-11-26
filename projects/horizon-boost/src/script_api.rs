//! Bounded read-only JSON batch CLI. Not Lua/Rhai or arbitrary code execution.
//! Explicit user invocation only; no process changes, network, macros or writes.
use serde::Deserialize;
use serde_json::{json,Value};
use std::{fs,path::Path};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Script{version:u32,commands:Vec<Action>}
#[derive(Deserialize)]
#[serde(tag="op",rename_all="snake_case",deny_unknown_fields)]
enum Action{
 ListShaders,
 ListPresets,
 ValidateStack{stages:Vec<usize>},
 ValidateGraph{graph:crate::node_graph::Graph},
 PreviewFrameBudget{fps:u32},
 EstimateReplayDisk{seconds:u32,bitrate_mbps:u32},
}
pub fn evaluate(input:&str)->Result<String,String>{
 if input.len()>65536{return Err("Script exceeds 64KiB limit".into());}
 let data:Script=serde_json::from_str(input).map_err(|e|format!("JSON script syntax: {e}"))?;
 if data.version!=1{return Err("Unsupported script version".into());}
 if data.commands.is_empty()||data.commands.len()>32{return Err("Choose 1-32 read-only actions".into());}
 let mut out=Vec::<Value>::with_capacity(data.commands.len());
 for action in data.commands{
  let row=match action{
   Action::ListShaders=>json!({"ok":true,"filters":crate::native_filter::PROGRAMS.iter().enumerate().map(|(id,(name,_))|json!({"id":id,"name":name})).collect::<Vec<_>>()}),
   Action::ListPresets=>json!({"ok":true,"presets":crate::native_presets::PRESETS.iter().enumerate().map(|(id,p)|json!({"id":id,"name":p.label})).collect::<Vec<_>>()}),
   Action::ValidateStack{stages}=>{
    if let Err(e)=crate::native_stack::validate(&stages){json!({"ok":false,"reason":e})}
    else{json!({"ok":true,"ordered_stages":stages})}
   },
   Action::ValidateGraph{graph}=>match graph.compile(){
    Ok(stages)=>json!({"ok":true,"ordered_stages":stages,"nodes":graph.nodes.len()}),
    Err(e)=>json!({"ok":false,"reason":e}),
   },
   Action::PreviewFrameBudget{fps}=>{
    if !crate::preview_pacer::LIMITS.contains(&fps){json!({"ok":false,"reason":"Unsupported own-output FPS limit"})}
    else{json!({"ok":true,"own_output_cap":fps,"deadline_ms":crate::preview_pacer::frame_budget(fps).map(|d|d.as_secs_f64()*1000.),"no_game_fps_control":true})}
   },
   Action::EstimateReplayDisk{seconds,bitrate_mbps}=>{
    if ![15,30,60].contains(&seconds)||![2,4,6,8,12,16].contains(&bitrate_mbps){json!({"ok":false,"reason":"Choose supported replay seconds and bitrate"})}
    else{
     let ring_seconds=(seconds+15)as f64;
     let estimated=ring_seconds*(bitrate_mbps as f64)*1_000_000./8.*1.2/1024./1024.;
     json!({"ok":true,"estimated_ring_mib":estimated.ceil(),"is_estimate":true,"no_recording_started":true})
    }
   }
  };
  out.push(row);
 }
 let result=json!({"format":"HorizonBoost.ReadOnlyScript.1","writes":false,"game_modified":false,"results":out});
 let text=serde_json::to_string_pretty(&result).map_err(|e|e.to_string())?;
 if text.len()>131072{return Err("Script response too large".into());}
 Ok(text)
}
pub fn evaluate_file(path:&Path)->Result<String,String>{
 let meta=fs::symlink_metadata(path).map_err(|e|e.to_string())?;
 if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>65536{return Err("Only regular scripts under 64KiB are supported".into());}
 let raw=fs::read_to_string(path).map_err(|e|e.to_string())?;
 evaluate(&raw)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn executes_five_real_read_only_contracts(){
  let script=r#"{"version":1,"commands":[{"op":"validate_stack","stages":[67,68,2]},{"op":"validate_graph","graph":{"nodes":[{"id":0,"stage":68}],"edges":[]}},{"op":"preview_frame_budget","fps":120},{"op":"estimate_replay_disk","seconds":30,"bitrate_mbps":8},{"op":"list_shaders"}]}"#;
  let parsed:Value=serde_json::from_str(&evaluate(script).unwrap()).unwrap();
  assert_eq!(parsed["writes"],false);assert_eq!(parsed["results"][0]["ordered_stages"],json!([67,68,2]));
  assert_eq!(parsed["results"][2]["own_output_cap"],120);
  assert!(parsed["results"][4]["filters"].as_array().unwrap().len()>=69);
 }
 #[test]fn unknown_actions_payloads_and_unsafe_requests_fail(){
  for raw in [
   r#"{"version":1,"commands":[{"op":"exec","command":"powershell"}]}"#,
   r#"{"version":1,"commands":[{"op":"validate_stack","stages":[1,999]}],"allow_write":true}"#,
   r#"{"version":99,"commands":[{"op":"list_presets"}]}"#,
   r#"{"version":1,"commands":[]}"#,
  ]{assert!(evaluate(raw).is_err()||serde_json::from_str::<Value>(&evaluate(raw).unwrap()).unwrap()["results"][0]["ok"]==false);}
 }
}
