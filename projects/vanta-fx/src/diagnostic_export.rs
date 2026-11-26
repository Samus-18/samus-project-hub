pub fn report(s:&crate::settings::Settings)->serde_json::Value{
 serde_json::json!({
 "format":"HorizonBoost.Diagnostics.1","version":env!("CARGO_PKG_VERSION"),
 "native_programs":crate::native_filter::PROGRAMS.len(),"looks":crate::native_presets::PRESETS.len(),
 "scalers":crate::spatial::MODES.len(),"presentmon_available":crate::fps::find_presentmon().is_some(),
 "ffmpeg_available":crate::record::ffmpeg_exe().is_some(),
 "preview":{"scale_mode":s.native_scale_mode,"ratio":s.native_scale_ratio,"quality":s.native_quality,
 "processing_percent":s.processing_percent,"fps_cap":s.preview_fps_limit,"vsync":s.preview_vsync,"waitable":s.preview_low_latency,"direct_composition_requested":s.preview_composition,"tearing_requested":s.preview_tearing},
 "features":{"grade_enabled":s.lighting.grade.enabled,"light_enabled":s.lighting.enabled,
 "layer_overrides":s.lighting.layers.strength.len(),"ordered_layer_overrides":s.lighting.layers.ordered_mix.len(),"base_look_bypass":s.lighting.bypass_look,"cinema_enabled":s.lighting.cinema.active(),"cinema_quality":s.lighting.cinema.quality,"recipe_enabled":s.lighting.recipe.active(),"precision_finish":s.lighting.precision.enabled,"precision_force_fp32":s.lighting.precision.force_fp32,"experimental_fg_allowed":s.experimental_fg_allowed},
 "limitations":{"exclusive_fullscreen_verified":false,"game_fps_gain_verified":false,"includes_raw_logs":false}
 })
}
pub fn with_capture(s:&crate::settings::Settings,h:Option<crate::capture_health::Snapshot>)->serde_json::Value{
 let mut out=report(s);
 out["capture"]=h.map(|v|serde_json::json!({"dequeued":v.dequeued,"duplicate_frames":v.duplicate_frames,"stale_pre_filter":v.stale_pre_filter,"fg_pressure_skips":v.fg_pressure_skips,"fg_reason":v.fg_reason,"own_minimized":v.own_minimized,"capture_interval_active":v.capture_interval_active,"empty_polls":v.empty,"fg_deferred":v.deferred,"fg_present_failures":v.failed_fg,"fg_effective_factor":v.fg_effective,"fg_stale_deferrals":v.fg_stale,"fg_reduced_decisions":v.fg_reduced,"fg_gpu_errors":v.fg_gpu_errors,"paused":v.paused,"capture_age_ms":v.capture_age_ms,"present_return_age_ms":v.present_age_ms,"worker_heartbeat_age_ms":v.worker_age_ms,"non_submitted":v.non_submitted,"effective_passes":v.effective_passes,"gpu_phase_ms":v.gpu_phase_ms,"gpu_phase_labels":["copy","prefilter_and_filters","scale_reconstruction","temporal_or_fg_estimate","fg_synthesis"],"wait_timeouts":v.wait_timeouts,"tearing_capability":v.tearing_supported,"tearing_flag":v.tearing_active,"optional_stage_warning_bits":v.optional_warning})).unwrap_or(serde_json::Value::Null);
 out
}
#[cfg(test)]mod tests{
 #[test]fn report_excludes_private_profile_fields(){
  let mut s=crate::settings::Settings::default();s.game_notes.insert("PRIVATE_PATH".into(),"PRIVATE_NOTE".into());
  let raw=super::report(&s).to_string();assert!(!raw.contains("PRIVATE"));assert!(!raw.contains("game_notes"));assert!(raw.contains("includes_raw_logs"));let raw=super::with_capture(&s,Some(crate::capture_health::Health::new().snapshot())).to_string();assert!(!raw.contains("PRIVATE"));assert!(raw.contains("gpu_phase_ms"));
 }
}

fn crc32(v:&[u8])->u32{let mut c=!0u32;for b in v{c^=u32::from(*b);for _ in 0..8{c=(c>>1)^if c&1!=0{0xedb88320}else{0};}}!c}
fn u16le(v:&mut Vec<u8>,x:u16){v.extend_from_slice(&x.to_le_bytes());}
fn u32le(v:&mut Vec<u8>,x:u32){v.extend_from_slice(&x.to_le_bytes());}
fn zip_store(entries:&[(&str,&[u8])])->Result<Vec<u8>,String>{
 if entries.is_empty()||entries.len()>8||entries.iter().any(|(n,b)|n.is_empty()||n.len()>64||!n.bytes().all(|c|c.is_ascii_alphanumeric()||matches!(c,b'.'|b'_'|b'-'))||b.len()>65536){return Err("Invalid bounded diagnostic archive".into());}
 let mut out=Vec::new();let mut central=Vec::new();
 for(name,data)in entries{
  let offset=out.len()as u32;let len=data.len()as u32;let n=name.len()as u16;let crc=crc32(data);
  u32le(&mut out,0x04034b50);for a in [20,0,0,0,0x21]{u16le(&mut out,a);}for a in [crc,len,len]{u32le(&mut out,a);}u16le(&mut out,n);u16le(&mut out,0);out.extend_from_slice(name.as_bytes());out.extend_from_slice(data);
  u32le(&mut central,0x02014b50);for a in [20,20,0,0,0,0x21]{u16le(&mut central,a);}for a in [crc,len,len]{u32le(&mut central,a);}for a in [n,0,0,0,0]{u16le(&mut central,a);}u32le(&mut central,0);u32le(&mut central,offset);central.extend_from_slice(name.as_bytes());
 }
 let offset=out.len()as u32;let len=central.len()as u32;out.extend_from_slice(&central);u32le(&mut out,0x06054b50);for a in [0,0,entries.len()as u16,entries.len()as u16]{u16le(&mut out,a);}u32le(&mut out,len);u32le(&mut out,offset);u16le(&mut out,0);Ok(out)
}
pub fn bundle(s:&crate::settings::Settings,h:Option<crate::capture_health::Snapshot>)->Result<Vec<u8>,String>{
 let json=serde_json::to_vec_pretty(&with_capture(s,h)).map_err(|e|e.to_string())?;
 let mut csv=String::from("metric,value\n");
 if let Some(h)=h{for(k,v)in[("older_frames_discarded",h.dequeued),("empty_capture_polls",h.empty),("fg_deferred",h.deferred),("fg_failed",h.failed_fg),("own_present_non_submitted",h.non_submitted),("optional_warning_bits",u64::from(h.optional_warning)),("wait_timeouts",h.wait_timeouts)]{csv.push_str(&format!("{k},{v}\n"));}}
 let readme=b"Horizon Boost privacy-safe technical snapshot. No raw log files, game names, paths, PIDs, account information, images, audio or network targets. Numeric counters refer to own output. This does not certify game/anti-cheat compatibility.\n";
 zip_store(&[("README.txt",readme),("diagnostics.json",&json),("capture-summary.csv",csv.as_bytes())])
}
pub fn bundle_proof()->Result<String,String>{
 let mut s=crate::settings::Settings::default();s.game_notes.insert("PRIVATE_TEST_PATH".into(),"PRIVATE_TEST_NOTE".into());
 let data=bundle(&s,Some(crate::capture_health::Health::new().snapshot()))?;
 if data.windows(b"PRIVATE_TEST".len()).any(|v|v==b"PRIVATE_TEST"){return Err("Private fixture leaked".into());}
 let path=std::path::PathBuf::from("build-tests").join("diagnostics-v72.zip");std::fs::create_dir_all(path.parent().unwrap()).map_err(|e|e.to_string())?;std::fs::write(path,data).map_err(|e|e.to_string())?;
 Ok("DIAGNOSTIC_ZIP=PASS FIXED_ENTRIES=3 PRIVACY_FIXTURE_EXCLUDED=true".into())
}
#[cfg(test)]mod zip_tests{use super::*;#[test]fn crc_and_paths(){assert_eq!(crc32(b"123456789"),0xcbf43926);assert!(zip_store(&[("../bad",b"x")]).is_err());let x=zip_store(&[("a.txt",b"abc")]).unwrap();assert_eq!(&x[..4],&[80,75,3,4]);assert_eq!(&x[x.len()-22..x.len()-18],&[80,75,5,6]);}}
