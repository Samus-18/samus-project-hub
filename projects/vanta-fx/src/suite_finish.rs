//! Local-only finish batch. Versioned settings, bounded input and explicit actions.
use serde::{Serialize,Deserialize};
use std::{path::{Path,PathBuf},fs,io};
pub const SCHEMA:u32=1;
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Pin{pub x:f32,pub y:f32,pub label:String,pub done:bool}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct MapNote{pub image:Option<PathBuf>,pub pins:Vec<Pin>}
impl MapNote{
 pub fn sanitize(&mut self){self.pins.truncate(100);self.pins.retain(|p|p.x.is_finite()&&p.y.is_finite());for p in &mut self.pins{p.x=p.x.clamp(0.,1.);p.y=p.y.clamp(0.,1.);p.label=p.label.chars().take(100).collect();}}
 pub fn add(&mut self,x:f32,y:f32,label:&str)->bool{if self.pins.len()>=100||!x.is_finite()||!y.is_finite(){return false;}self.pins.push(Pin{x:x.clamp(0.,1.),y:y.clamp(0.,1.),label:label.chars().take(100).collect(),done:false});true}
}
pub fn load_reference(p:&Path)->Result<image::RgbaImage,String>{
 let meta=fs::symlink_metadata(p).map_err(|e|e.to_string())?;if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>32*1024*1024{return Err("Select a local regular image under 32 MiB".into());}
 let mut reader=image::ImageReader::open(p).map_err(|e|e.to_string())?.with_guessed_format().map_err(|e|e.to_string())?;
 let mut lim=image::Limits::default();lim.max_image_width=Some(8192);lim.max_image_height=Some(8192);lim.max_alloc=Some(128*1024*1024);reader.limits(lim);
 let img=reader.decode().map_err(|e|e.to_string())?;Ok(if img.width()>2048||img.height()>2048{img.thumbnail(2048,2048).to_rgba8()}else{img.to_rgba8()})
}
pub fn local_copy(src:&Path,dest_dir:&Path)->Result<PathBuf,String>{
 let name=crate::clip_book::key(src).ok_or("Not an app-owned capture name")?;
 let meta=fs::symlink_metadata(src).map_err(|e|e.to_string())?;
 if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>16*1024*1024*1024{return Err("Unsupported source file".into());}
 if !dest_dir.is_dir()||!dest_dir.is_absolute()||dest_dir.to_string_lossy().starts_with("\\\\"){return Err("Choose a local destination directory".into());}
 let dest=dest_dir.join(name);let mut input=fs::File::open(src).map_err(|e|e.to_string())?;
 let mut output=fs::OpenOptions::new().write(true).create_new(true).open(&dest).map_err(|e|e.to_string())?;
 let result=(||{let n=io::copy(&mut input,&mut output).map_err(|e|e.to_string())?;if n!=meta.len(){return Err("Source changed during copy".into());}output.sync_all().map_err(|e|e.to_string())?;Ok(dest.clone())})();drop(output);
 if result.is_err(){let _=fs::remove_file(&dest);}result
}
pub fn gpu_pressure(ratio:Option<f32>,enabled:bool)->bool{enabled&&ratio.is_some_and(|v|v.is_finite()&&v>=0.90)}
pub const CAPABILITIES:[(&str,&str,&str);12]=[
 ("Capture and clips","Integrated","Explicit desktop capture, local files; not game-event detection."),
 ("Frame generation","Experimental","Own interpolated output only; no vendor AFMF unlock."),
 ("Spatial upscaling","Integrated","Own output; lower game resolution separately."),
 ("HUD and macros","Integrated / restricted","External HUD; macros only permitted local/offline targets."),
 ("AMD optical flow","Pending","Official SDK not integrated."),
 ("HDR / VRR","Partial","Physical display calibration and VRR not verified."),
 ("Vulkan / DX12","Partial","Independent proofs, not full live renderer."),
 ("Voice clip control","Experimental","Explicit one-shot local recognition; installed recognizer required."),
 ("OCR","Experimental","Explicit local image OCR; installed Windows language required."),
 ("Game event adapters","Pending","No unsupported game memory or event injection."),
 ("Save vault","Integrated","Verified copy, restore to new folder only."),
 ("Reference maps","Integrated","User-imported local images and manual pins.")
];
#[cfg(test)]mod tests{use super::*;
 #[test]fn pins_bounds_and_roundtrip(){let mut m=MapNote::default();assert!(!m.add(f32::NAN,0.,"x"));assert!(m.add(2.,-1.,"Boss"));assert_eq!((m.pins[0].x,m.pins[0].y),(1.,0.));let s=serde_json::to_string(&m).unwrap();assert_eq!(serde_json::from_str::<MapNote>(&s).unwrap(),m);}
 #[test]fn local_sharing_never_overwrites(){let root=std::env::temp_dir().join(format!("hb-share-{}",crate::media_tools::stamp()));let a=root.join("a");let b=root.join("b");fs::create_dir_all(&a).unwrap();fs::create_dir_all(&b).unwrap();let src=a.join("HBoost_sample.mp4");fs::write(&src,b"fixture").unwrap();let out=local_copy(&src,&b).unwrap();assert_eq!(fs::read(&out).unwrap(),b"fixture");assert!(local_copy(&src,&b).is_err());assert_eq!(fs::read(&src).unwrap(),b"fixture");let _=fs::remove_dir_all(root);}
 #[test]fn pressure_only_valid_measurement(){assert!(!gpu_pressure(None,true));assert!(!gpu_pressure(Some(f32::NAN),true));assert!(!gpu_pressure(Some(0.99),false));assert!(gpu_pressure(Some(0.90),true));}
 #[test]fn reference_decodes_local_png(){let p=std::env::temp_dir().join(format!("hb-map-{}.png",crate::media_tools::stamp()));image::RgbaImage::new(24,16).save(&p).unwrap();let im=load_reference(&p).unwrap();assert_eq!(im.dimensions(),(24,16));let _=fs::remove_file(p);}
}

pub fn free_bytes(p:&Path)->Option<u64>{
 use std::os::windows::ffi::OsStrExt;
 #[link(name="kernel32")]unsafe extern "system"{fn GetDiskFreeSpaceExW(dir:*const u16,free:*mut u64,total:*mut u64,total_free:*mut u64)->i32;}
 let dir=p.ancestors().find(|p|p.is_dir())?;let wide:Vec<u16>=dir.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
 let mut n=0u64;let ok=unsafe{GetDiskFreeSpaceExW(wide.as_ptr(),&mut n,std::ptr::null_mut(),std::ptr::null_mut())};(ok!=0).then_some(n)
}
#[cfg(test)]mod disk_tests{#[test]fn local_disk_capacity_read(){assert!(super::free_bytes(&std::env::temp_dir()).is_some());}}

#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]pub struct MeasuredOverhead{pub unix:u64,pub feature:u8,pub baseline:f32,pub active:f32,pub baseline_low:Option<f32>,pub active_low:Option<f32>,pub fg:u8}
impl MeasuredOverhead{
 pub fn valid(&self)->bool{self.feature<5&&self.baseline.is_finite()&&self.active.is_finite()&&self.baseline>0.&&self.active>0.&&self.baseline_low.is_none_or(|x|x.is_finite()&&x>=0.)&&self.active_low.is_none_or(|x|x.is_finite()&&x>=0.)&&matches!(self.fg,1|2|3)}
}
#[cfg(test)]mod measurements_tests{#[test]fn measured_overhead_rejects_invalid_fps(){let mut v=super::MeasuredOverhead{baseline:60.,active:50.,fg:2,..Default::default()};assert!(v.valid());v.active=f32::NAN;assert!(!v.valid());}}

// Most recent valid measurement per actual feature; do not add unrelated samples.
pub fn measured_cost(rows:&[MeasuredOverhead],feature:u8)->Option<f32>{
 rows.iter().rev().find(|r|r.feature==feature&&r.valid()).map(|r|(1.-r.active/r.baseline)*100.)
}
#[cfg(test)]mod cost_tests{#[test]fn no_invented_budget_samples(){let a=super::MeasuredOverhead{baseline:60.,active:45.,fg:1,feature:0,..Default::default()};assert_eq!(super::measured_cost(&[],0),None);assert_eq!(super::measured_cost(&[a],0),Some(25.));}}

#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(default)]pub struct ModuleConfig{pub schema:u32,pub ocr:bool,pub voice:bool,pub reference:bool,pub highlights:bool}
impl Default for ModuleConfig{fn default()->Self{Self{schema:1,ocr:true,voice:true,reference:true,highlights:true}}}
impl ModuleConfig{pub fn sanitize(&mut self){if self.schema!=1{*self=Self{schema:1,ocr:false,voice:false,reference:false,highlights:false};}}}
#[cfg(test)]mod modules_tests{#[test]fn unknown_module_schema_fails_closed(){let mut m=super::ModuleConfig::default();m.schema=99;m.sanitize();assert!(!m.ocr&&!m.voice&&!m.reference&&!m.highlights);}}
