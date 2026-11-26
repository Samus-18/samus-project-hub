//! Explicit local montage and crop. All inputs preserved.
use std::{path::{Path,PathBuf},process::Command,time::Duration,fs,io::Write};
use crate::media_tools::{bounded,probe,stamp,encoder_args};
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub struct Piece{pub path:PathBuf,pub start:f64,pub end:f64,pub caption:String}
pub fn total(p:&[Piece])->Result<f64,String>{
 if p.is_empty()||p.len()>8{return Err("Choose 1 to 8 clips".into());}
 let mut sum=0.;
 for v in p{let n=v.end-v.start;if !v.start.is_finite()||!v.end.is_finite()||v.start<0.||n<0.25||n>120.{return Err("Invalid clip range".into());}sum+=n;}
 if sum>120.{return Err("Montage maximum: 120 seconds".into());}Ok(sum)
}
fn srt_time(ms:u64)->String{format!("{:02}:{:02}:{:02},{:03}",ms/3600000,ms/60000%60,ms/1000%60,ms%1000)}
pub fn subtitles(p:&[Piece])->Result<String,String>{
 total(p)?;let mut elapsed=0.;let mut out=String::new();let mut index=1;
 for v in p{let end=elapsed+v.end-v.start;let text=v.caption.chars().filter(|c|!c.is_control()).take(160).collect::<String>().replace("-->","->");
  if !text.trim().is_empty(){out+=&format!("{index}\n{} --> {}\n{}\n\n",srt_time((elapsed*1000.).round() as u64),srt_time((end*1000.).round() as u64),text).replace("\n","\n");index+=1;}elapsed=end;
 }Ok(out)
}
fn unused(p:&Path)->Result<(),String>{if p.exists(){Err("Destination already exists".into())}else{Ok(())}}
fn range(input:&Path,start:f64,end:f64)->Result<(),String>{let v=probe(input)?;if !start.is_finite()||!end.is_finite()||start<0.||end-start<0.25||end-start>120.||end>v.duration+0.03{Err("Invalid source interval".into())}else{Ok(())}}
pub fn crop_filter(mode:usize)->Option<String>{
 let (w,h)=crate::media_tools::export_dims(mode)?;
 // Scale to fill, center crop, square pixels. No stretched geometry.
 Some(format!("scale={w}:{h}:force_original_aspect_ratio=increase:force_divisible_by=2,crop={w}:{h},setsar=1"))
}
pub fn crop(input:&Path,output:&Path,start:f64,end:f64,mode:usize,rate:u32)->Result<(),String>{
 unused(output)?;range(input,start,end)?;
 let (w,h)=crate::media_tools::export_dims(mode).ok_or("Invalid crop format")?;
 let filter=crop_filter(mode).ok_or("Invalid crop format")?;
 let ff=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;
 let result=bounded(Command::new(ff).args(["-hide_banner","-loglevel","error","-n","-ss",&format!("{start:.3}"),"-i"]).arg(input)
 .args(["-t",&format!("{:.3}",end-start),"-map","0:v:0","-map","0:a?","-vf",&filter]).args(encoder_args(false,rate)).args(["-c:a","aac","-b:a","160k","-movflags","+faststart"]).arg(output),Duration::from_secs(180))
 .and_then(|_|{let v=probe(output)?;if v.width!=w||v.height!=h||(v.duration-(end-start)).abs()>0.25{Err("Crop verification failed".into())}else{Ok(())}});
 if result.is_err(){let _=fs::remove_file(output);}result
}
pub fn render(p:&[Piece],output:&Path,mode:usize,rate:u32)->Result<(),String>{render_audio(p,output,mode,rate,false)}
pub fn render_audio(p:&[Piece],output:&Path,mode:usize,rate:u32,audio:bool)->Result<(),String>{
 let duration=total(p)?;unused(output)?;
 let (w,h)=crate::media_tools::export_dims(mode).ok_or("Invalid montage format")?;
 let side=output.with_extension("srt");unused(&side)?;
 let captions=subtitles(p)?;
 let ff=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;
 for v in p{range(&v.path,v.start,v.end)?;}
 let temp=std::env::temp_dir().join(format!("hb-montage-{}",stamp()));fs::create_dir(&temp).map_err(|e|e.to_string())?;
 let result=(||{
  let filter=format!("scale={w}:{h}:force_original_aspect_ratio=decrease:force_divisible_by=2,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps=30");
  let mut manifest=String::new();
  for (i,v) in p.iter().enumerate(){
   let name=format!("part{i:02}.mp4");
   let mut cmd=Command::new(&ff);cmd.args(["-hide_banner","-loglevel","error","-n","-ss",&format!("{:.3}",v.start),"-i"]).arg(&v.path);
   let has_audio=audio&&crate::media_tools::audio_track_count(&v.path)?>0;
   if audio&&!has_audio{cmd.args(["-f","lavfi","-i","anullsrc=r=48000:cl=stereo"]);}
   cmd.args(["-t",&format!("{:.3}",v.end-v.start),"-map","0:v:0"]);
   if audio{cmd.args(["-map",if has_audio{"0:a:0"}else{"1:a:0"},"-af","apad","-ar","48000","-ac","2","-c:a","aac","-b:a","160k"]);}else{cmd.arg("-an");}
   bounded(cmd.args(["-vf",&filter]).args(encoder_args(false,rate)).arg(temp.join(&name)),Duration::from_secs(120))?;
   manifest+=&format!("file '{name}'\n").replace("\n","\n");
  }
  fs::write(temp.join("manifest.txt"),manifest).map_err(|e|e.to_string())?;
  bounded(Command::new(&ff).args(["-hide_banner","-loglevel","error","-n","-f","concat","-safe","1","-i","manifest.txt","-c","copy","-movflags","+faststart"]).arg(output).current_dir(&temp),Duration::from_secs(60))?;
  let v=probe(output)?;if v.width!=w||v.height!=h||(v.duration-duration).abs()>0.35||crate::media_tools::audio_track_count(output)?!=usize::from(audio){return Err("Montage output verification failed".into());}
  if !captions.is_empty(){let mut file=fs::OpenOptions::new().create_new(true).write(true).open(&side).map_err(|e|e.to_string())?;file.write_all(captions.as_bytes()).map_err(|e|e.to_string())?;}
  Ok(())
 })();let _=fs::remove_dir_all(temp);
 if result.is_err(){let _=fs::remove_file(output);}result
}
#[cfg(test)]mod tests{use super::*;
 fn piece(a:f64,b:f64,c:&str)->Piece{Piece{path:"test.mp4".into(),start:a,end:b,caption:c.into()}}
 #[test]fn range_and_caption_boundaries(){let p=[piece(1.,3.,"Start"),piece(5.,6.5,"End")];assert_eq!(total(&p),Ok(3.5));let s=subtitles(&p).unwrap();assert!(s.contains("00:00:02,000 --> 00:00:03,500"));assert!(total(&[piece(0.,121.,"")]).is_err());assert!(total(&[piece(f64::NAN,1.,"")]).is_err());assert!(total(&[]).is_err());}
 #[test]fn real_montage_and_crop(){let dir=std::env::temp_dir().join(format!("hb-montage-test-{}",stamp()));fs::create_dir(&dir).unwrap();
  let result=(||->Result<(),String>{let ff=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;let a=dir.join("input.mp4");let b=dir.join("crop.mp4");let c=dir.join("montage.mp4");
   bounded(Command::new(ff).args(["-hide_banner","-loglevel","error","-f","lavfi","-i","testsrc2=size=320x180:rate=30","-t","2","-c:v","libx264","-pix_fmt","yuv420p"]).arg(&a),Duration::from_secs(20))?;
   let original=fs::read(&a).map_err(|e|e.to_string())?;
   crop(&a,&b,0.,0.5,2,4)?;assert_eq!((probe(&b)?.width,probe(&b)?.height),(720,1280));
   let p=[Piece{path:a.clone(),start:0.,end:0.5,caption:"First".into()},Piece{path:a.clone(),start:1.,end:1.5,caption:"Second".into()}];
   render(&p,&c,0,4)?;assert!((probe(&c)?.duration-1.).abs()<0.15);assert_eq!(crate::media_tools::audio_track_count(&c)?,0);assert!(c.with_extension("srt").is_file());let audio=dir.join("audio.mp4");render_audio(&p,&audio,0,4,true)?;assert_eq!(crate::media_tools::audio_track_count(&audio)?,1);assert_eq!(original,fs::read(&a).unwrap());Ok(())
  })();let _=fs::remove_dir_all(dir);result.unwrap();
 }
}

pub fn locate(p:&[Piece],t:f64)->Option<(usize,f64)>{
 let total=total(p).ok()?;if !t.is_finite()||t<0.||t>total{return None;}
 let mut elapsed=0.;for(i,v)in p.iter().enumerate(){let end=elapsed+v.end-v.start;if t<end||i+1==p.len(){return Some((i,(v.start+t-elapsed).clamp(v.start,(v.end-0.001).max(v.start))))}elapsed=end;}None
}
#[cfg(test)]mod locate_tests{use super::*;#[test]fn montage_scrub_respects_cut_boundaries(){let p=[Piece{path:"a".into(),start:5.,end:7.,caption:String::new()},Piece{path:"b".into(),start:9.,end:12.,caption:String::new()}];assert_eq!(locate(&p,0.),Some((0,5.)));assert_eq!(locate(&p,2.),Some((1,9.)));assert!(locate(&p,5.).is_some());assert_eq!(locate(&p,6.),None);assert_eq!(locate(&p,f64::NAN),None);}}
