use std::{path::{Path,PathBuf},process::{Command,Stdio},io::Read,time::{Instant,Duration,SystemTime,UNIX_EPOCH},thread,fs};
use std::os::windows::process::CommandExt;
const NO_WINDOW:u32=0x08000000;
pub fn stamp()->String{
 static SEQ:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
 format!("{}-{}-{}",SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),std::process::id(),SEQ.fetch_add(1,std::sync::atomic::Ordering::Relaxed))
}
pub fn bounded(cmd:&mut Command,timeout:Duration)->Result<String,String>{
 let mut child=cmd.creation_flags(NO_WINDOW).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e|e.to_string())?;
 // Drain even after the cap so an unexpectedly verbose process cannot deadlock.
 fn drain(mut input:impl Read+Send+'static)->thread::JoinHandle<Vec<u8>>{thread::spawn(move||{
  let mut out=Vec::new();let mut buf=[0u8;8192];
  while let Ok(n)=input.read(&mut buf){if n==0{break;}let take=n.min(262144usize.saturating_sub(out.len()));out.extend_from_slice(&buf[..take]);}
  out
 })}
 let a=drain(child.stdout.take().ok_or("stdout unavailable")?);let b=drain(child.stderr.take().ok_or("stderr unavailable")?);
 let now=Instant::now();
 let status=loop{
  match child.try_wait(){Ok(Some(s))=>break Some(s),Ok(None) if now.elapsed()<timeout=>thread::sleep(Duration::from_millis(25)),
   _=>{let _=child.kill();let _=child.wait();break None;}
  }
 };
 let out=String::from_utf8_lossy(&a.join().unwrap_or_default()).into_owned();
 let error=String::from_utf8_lossy(&b.join().unwrap_or_default()).into_owned();
 match status{Some(s) if s.success()=>Ok(out),Some(s)=>Err(format!("Media command failed ({s}): {}",error.chars().take(350).collect::<String>())),None=>Err("Media command timed out".into())}
}
pub fn bitrate(v:u32)->u32{v.clamp(2,16)}
pub fn replay_limit(v:u32)->u32{v.clamp(64,1024)}
pub fn encoder_args(hardware:bool,mbps:u32)->Vec<String>{
 let rate=format!("{}M",bitrate(mbps));let buffer=format!("{}M",bitrate(mbps)*2);
 let mut a=vec!["-c:v".into(),if hardware{"h264_amf".into()}else{"libx264".into()}];
 if hardware{a.extend(["-quality","speed","-rc","cbr","-forced_idr","1","-bf","0"].map(str::to_owned));}
 else{a.extend(["-preset","veryfast","-bf","0"].map(str::to_owned));}
 a.extend(["-b:v".into(),rate.clone(),"-maxrate".into(),rate,"-bufsize".into(),buffer,"-pix_fmt".into(),"yuv420p".into()]);
 a
}
#[derive(Clone,Debug)]pub struct Encoders{pub amf:bool,pub cpu:bool}
pub fn encoder_probe()->Result<Encoders,String>{
 let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;
 let check=|hardware|bounded(Command::new(&exe).args(["-hide_banner","-loglevel","error","-f","lavfi","-i","testsrc2=size=128x128:rate=30","-frames:v","8"]).args(encoder_args(hardware,4)).args(["-f","null","NUL"]),Duration::from_secs(20)).is_ok();
 Ok(Encoders{amf:check(true),cpu:check(false)})
}
fn ffprobe()->Option<PathBuf>{
 let path=crate::record::ffmpeg_exe()?.with_file_name("ffprobe.exe");if path.is_file(){Some(path)}else{
 std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).map(|p|p.join("ffprobe.exe")).find(|p|p.is_file())}
}
#[derive(Clone,Debug,serde::Deserialize)]pub struct Stream{pub codec_name:Option<String>,pub width:Option<u32>,pub height:Option<u32>}
#[derive(serde::Deserialize)]struct Format{duration:Option<String>}
#[derive(serde::Deserialize)]struct Info{streams:Vec<Stream>,format:Format}
#[derive(Clone,Debug)]pub struct ClipInfo{pub duration:f64,pub width:u32,pub height:u32,pub codec:String}
pub fn probe(path:&Path)->Result<ClipInfo,String>{
 let exe=ffprobe().ok_or("ffprobe missing")?;
 let raw=bounded(Command::new(exe).args(["-v","error","-select_streams","v:0","-show_entries","stream=codec_name,width,height:format=duration","-of","json","-i"]).arg(path),Duration::from_secs(15))?;
 let info:Info=serde_json::from_str(&raw).map_err(|e|e.to_string())?;
 let s=info.streams.first().ok_or("No video stream")?;
 let duration=info.format.duration.and_then(|v|v.parse::<f64>().ok()).filter(|v|v.is_finite()&&*v>0.).ok_or("Video duration unavailable")?;
 Ok(ClipInfo{duration,width:s.width.unwrap_or(0),height:s.height.unwrap_or(0),codec:s.codec_name.clone().unwrap_or("Unknown".into())})
}
pub fn audio_track_count(path:&Path)->Result<usize,String>{
 let exe=ffprobe().ok_or("ffprobe missing")?;let out=bounded(Command::new(exe).args(["-v","error","-select_streams","a","-show_entries","stream=codec_type","-of","csv=p=0","-i"]).arg(path),Duration::from_secs(15))?;
 Ok(out.lines().filter(|s|s.trim()=="audio").count())
}
fn checked_output(path:&Path)->Result<(),String>{
 if !path.metadata().is_ok_and(|v|v.len()>512){return Err("Output missing or empty".into());}
 let _=probe(path)?;Ok(())
}
pub fn remux(input:&Path,output:&Path)->Result<(),String>{
 if input==output||output.exists(){return Err("Destination already exists".into());}
 let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;
 let result=bounded(Command::new(exe).args(["-hide_banner","-loglevel","error","-n","-i"]).arg(input).args(["-map","0:v:0","-map","0:a?","-c","copy","-movflags","+faststart"]).arg(output),Duration::from_secs(120)).and_then(|_|checked_output(output));
 if result.is_err(){let _=fs::remove_file(output);}result
}
pub fn trim(input:&Path,output:&Path,start:f64,end:f64,hardware:bool,rate:u32)->Result<(),String>{
 let info=probe(input)?;
 if !start.is_finite()||!end.is_finite()||start<0.||end>info.duration+0.03||end-start<0.25||end-start>120.{
  return Err("Choose a valid clip range of 0.25 to 120 seconds".into());
 }
 if input==output||output.exists(){return Err("Destination already exists".into());}
 let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;
 let result=bounded(Command::new(exe).args(["-hide_banner","-loglevel","error","-n","-ss",&format!("{start:.3}"),"-i"]).arg(input)
  .args(["-t",&format!("{:.3}",end-start),"-map","0:v:0","-map","0:a?"]).args(encoder_args(hardware,rate)).args(["-c:a","aac","-b:a","160k","-movflags","+faststart"]).arg(output),
  Duration::from_secs(180)).and_then(|_|checked_output(output));
 if result.is_err(){let _=fs::remove_file(output);}result
}
#[derive(Clone,Debug)]pub struct Item{pub path:PathBuf,pub bytes:u64,pub video:bool}
pub fn gallery()->Vec<Item>{
 let mut out=Vec::new();
 for (dir,video) in [(crate::record::pictures(),false),(crate::record::videos(),true)]{
  if let Ok(entries)=fs::read_dir(dir){
   for e in entries.flatten().take(2000){
    if !e.file_type().is_ok_and(|t|t.is_file()){continue;}
    let p=e.path();let ext=p.extension().and_then(|v|v.to_str()).unwrap_or("").to_ascii_lowercase();
    let name=p.file_name().and_then(|v|v.to_str()).unwrap_or("");
    if !name.starts_with("HBoost_")||!(if video{matches!(ext.as_str(),"mkv"|"mp4")}else{ext=="png"}){continue;}
    if let Ok(m)=e.metadata(){out.push((m.modified().ok(),Item{path:p,bytes:m.len(),video}));}
   }
  }
 }
 out.sort_by(|a,b|b.0.cmp(&a.0));out.into_iter().take(200).map(|(_,v)|v).collect()
}
pub fn proof()->Result<String,String>{
 let dir=std::env::temp_dir().join(format!("hb-media-proof-{}",stamp()));fs::create_dir(&dir).map_err(|e|e.to_string())?;
 let result=(||{
  let enc=encoder_probe()?;if !enc.amf||!enc.cpu{return Err("Synthetic encoder probe unavailable".into());}
  let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;
  let mkv=dir.join("source.mkv");let mp4=dir.join("remux.mp4");let short=dir.join("trim.mp4");
  bounded(Command::new(&exe).args(["-hide_banner","-loglevel","error","-f","lavfi","-i","testsrc2=size=320x180:rate=30","-t","3"]).args(encoder_args(true,4)).arg(&mkv),Duration::from_secs(30))?;
  remux(&mkv,&mp4)?;trim(&mp4,&short,0.5,2.0,false,4)?;let a=still(&mp4,0.4)?;let b=still(&mp4,1.4)?;if a.dimensions()!=b.dimensions()||a==b{return Err("Clip still preview failed".into());}
  let i=probe(&short)?;if (i.duration-1.5).abs()>0.15||i.width!=320||i.height!=180{return Err("Trim dimensions or duration incorrect".into());}
  bounded(Command::new(exe).args(["-v","error","-i"]).arg(&short).args(["-f","null","NUL"]),Duration::from_secs(20))?;
  Ok(format!("MEDIA_TOOLS=PASS AMF=true CPU=true MKV_REMUX=true TRIM_DURATION={:.3} DECODE=true STILL_PREVIEW=true SOURCE_SYNTHETIC=true",i.duration))
 })();let _=fs::remove_dir_all(dir);result
}
#[cfg(test)]mod tests{
 #[test]fn encoder_args_and_limits(){
  assert_eq!(super::bitrate(99),16);assert_eq!(super::replay_limit(1),64);
  let a=super::encoder_args(true,6);assert!(a.windows(2).any(|x|x==["-b:v","6M"]));assert!(a.windows(2).any(|x|x==["-forced_idr","1"]));
 }
}

pub fn still(path:&Path,seconds:f64)->Result<image::RgbaImage,String>{
 if !seconds.is_finite()||seconds<0.{return Err("Invalid preview time".into());}
 let meta=fs::symlink_metadata(path).map_err(|e|e.to_string())?;if !meta.is_file()||meta.file_type().is_symlink(){return Err("Choose a regular local media file".into());}
 let read=|p:&Path|->Result<image::RgbaImage,String>{let mut reader=image::ImageReader::open(p).map_err(|e|e.to_string())?;let mut lim=image::Limits::default();lim.max_image_width=Some(8192);lim.max_image_height=Some(8192);lim.max_alloc=Some(128*1024*1024);reader.limits(lim);Ok(reader.decode().map_err(|e|e.to_string())?.thumbnail(640,360).to_rgba8())};
 if path.extension().is_some_and(|e|e.eq_ignore_ascii_case("png")){if meta.len()>64*1024*1024{return Err("Image exceeds 64 MiB".into());}return read(path);}
 let info=probe(path)?;if seconds>info.duration{return Err("Preview time exceeds duration".into());}
 let dir=std::env::temp_dir().join(format!("hb-still-{}",stamp()));fs::create_dir(&dir).map_err(|e|e.to_string())?;let temp=dir.join("still.png");
 let result=(||{let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;bounded(Command::new(exe).args(["-hide_banner","-loglevel","error","-n","-ss",&format!("{seconds:.3}"),"-i"]).arg(path).args(["-frames:v","1","-vf","scale=640:360:force_original_aspect_ratio=decrease","-update","1"]).arg(&temp),Duration::from_secs(20))?;read(&temp)})();
 let _=fs::remove_dir_all(dir);result
}

pub const EXPORT_NAMES:[&str;3]=["Landscape 720p","Landscape 1080p","Vertical 720x1280"];
pub fn export_dims(mode:usize)->Option<(u32,u32)>{[(1280,720),(1920,1080),(720,1280)].get(mode).copied()}
pub fn export_format(input:&Path,output:&Path,start:f64,end:f64,mode:usize,rate:u32)->Result<(),String>{
 let (w,h)=export_dims(mode).ok_or("Unknown export format")?;
 let info=probe(input)?;
 if !start.is_finite()||!end.is_finite()||start<0.||end>info.duration+0.03||end-start<0.25||end-start>120.{return Err("Choose a valid range of 0.25 to 120 seconds".into());}
 if input==output||output.exists(){return Err("Destination already exists".into());}
 let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;
 let filter=format!("scale={w}:{h}:force_original_aspect_ratio=decrease:force_divisible_by=2,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2,setsar=1");
 let result=bounded(Command::new(exe).args(["-hide_banner","-loglevel","error","-n","-ss",&format!("{start:.3}"),"-i"]).arg(input)
 .args(["-t",&format!("{:.3}",end-start),"-map","0:v:0","-map","0:a?","-vf",&filter]).args(encoder_args(false,rate)).args(["-c:a","aac","-b:a","160k","-movflags","+faststart"]).arg(output),Duration::from_secs(180))
 .and_then(|_|{let actual=probe(output)?;if actual.width!=w||actual.height!=h||(actual.duration-(end-start)).abs()>0.25{Err("Export verification failed".into())}else{Ok(())}});
 if result.is_err(){let _=fs::remove_file(output);}result
}
#[cfg(test)]mod export_tests{use super::*;
 #[test]fn fixed_export_dimensions(){assert_eq!(export_dims(2),Some((720,1280)));assert_eq!(export_dims(3),None);}
 #[test]fn real_synthetic_vertical_export(){
  let dir=std::env::temp_dir().join(format!("hb-format-{}",stamp()));fs::create_dir(&dir).unwrap();
  let result=(||->Result<(),String>{let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;let a=dir.join("input.mp4");let b=dir.join("output.mp4");
   bounded(Command::new(exe).args(["-hide_banner","-loglevel","error","-f","lavfi","-i","testsrc2=size=320x180:rate=15","-t","1","-c:v","libx264","-pix_fmt","yuv420p"]).arg(&a),Duration::from_secs(20))?;
   export_format(&a,&b,0.,1.,2,4)?;assert!(a.is_file());assert_eq!(probe(&b)?.height,1280);Ok(())
  })();let _=fs::remove_dir_all(dir);result.unwrap();
 }
}

pub fn split(input:&Path,left:&Path,right:&Path,at:f64,rate:u32)->Result<(),String>{
 let info=probe(input)?;
 if !at.is_finite()||at<0.25||info.duration-at<0.25||at>120.||info.duration-at>120.{return Err("Split requires two 0.25–120 second pieces".into());}
 if left==right||left==input||right==input||left.exists()||right.exists(){return Err("Choose two unused destinations".into());}
 trim(input,left,0.,at,false,rate)?;
 if let Err(e)=trim(input,right,at,info.duration,false,rate){return Err(format!("{e}; first part retained at {}",left.display()));}
 Ok(())
}
#[cfg(test)]mod split_tests{use super::*;
 #[test]fn synthetic_split_preserves_source(){
  let dir=std::env::temp_dir().join(format!("hb-split-{}",stamp()));fs::create_dir(&dir).unwrap();
  let result=(||->Result<(),String>{let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;let a=dir.join("source.mp4");let b=dir.join("part-a.mp4");let c=dir.join("part-b.mp4");
   bounded(Command::new(exe).args(["-hide_banner","-loglevel","error","-f","lavfi","-i","testsrc2=size=320x180:rate=30","-t","2","-c:v","libx264","-pix_fmt","yuv420p"]).arg(&a),Duration::from_secs(20))?;
   let source=fs::read(&a).map_err(|e|e.to_string())?;split(&a,&b,&c,1.,4)?;
   assert_eq!(source,fs::read(&a).unwrap());assert!((probe(&b)?.duration-1.).abs()<0.15);assert!((probe(&c)?.duration-1.).abs()<0.15);Ok(())
  })();let _=fs::remove_dir_all(dir);result.unwrap();
 }
}

pub fn adjust_audio(input:&Path,output:&Path,gain:[f32;2],mute_second:bool)->Result<(),String>{
 if input==output||output.exists(){return Err("Destination already exists".into());}
 let n=audio_track_count(input)?;if n==0||n>2{return Err("Select a video with one or two audio tracks".into());}
 if gain.iter().any(|v|!v.is_finite()||*v<0.||*v>2.){return Err("Gain must be 0 to 200%".into());}
 let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;
 let mut cmd=Command::new(exe);cmd.args(["-hide_banner","-loglevel","error","-n","-i"]).arg(input).args(["-map","0:v:0","-map","0:a","-c:v","copy","-c:a","aac","-b:a","160k"]);
 for i in 0..n{cmd.args([format!("-filter:a:{i}"),format!("volume={:.3}",if i==1&&mute_second{0.}else{gain[i]})]);}
 let result=bounded(cmd.arg(output),Duration::from_secs(180)).and_then(|_|{checked_output(output)?;if audio_track_count(output)?!=n{return Err("Audio track count changed".into());}Ok(())});
 if result.is_err(){let _=fs::remove_file(output);}result
}
#[cfg(test)]mod gain_tests{use super::*;
 #[test]fn two_track_gain_keeps_video_and_source(){
 let root=std::env::temp_dir().join(format!("hb-gain-{}",stamp()));fs::create_dir(&root).unwrap();
 let result=(||->Result<(),String>{let ff=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;let a=root.join("a.mkv");let b=root.join("b.mkv");
 bounded(Command::new(ff).args(["-hide_banner","-loglevel","error","-f","lavfi","-i","testsrc2=size=160x90:rate=15","-f","lavfi","-i","sine=frequency=440:sample_rate=48000","-f","lavfi","-i","sine=frequency=880:sample_rate=48000","-t","1","-map","0:v","-map","1:a","-map","2:a","-c:v","libx264","-c:a","aac"]).arg(&a),Duration::from_secs(30))?;
 let original=fs::read(&a).unwrap();adjust_audio(&a,&b,[0.5,1.],true)?;assert_eq!(audio_track_count(&b)?,2);assert_eq!(probe(&a)?.codec,probe(&b)?.codec);assert_eq!(original,fs::read(&a).unwrap());Ok(())})();let _=fs::remove_dir_all(root);result.unwrap();
 }
}
