use std::{
    io::Write,
    path::PathBuf,
    process::{Child,Command,Stdio},
    sync::mpsc::{self,Receiver},
    time::{SystemTime,UNIX_EPOCH},
};
use std::os::windows::process::CommandExt;
const NO_WINDOW:u32=0x08000000;
pub fn ffmpeg_exe()->Option<PathBuf>{
    let path=std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
        .map(|p|p.join("Microsoft").join("WinGet").join("Links").join("ffmpeg.exe"));
    if let Some(p)=path{if p.exists(){return Some(p);}}
    for part in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()){
        let full=part.join("ffmpeg.exe");if full.exists(){return Some(full);}
    }
    None
}
fn stamp()->String{
    let dur=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{}_{}",dur.as_secs(),dur.subsec_millis())
}
pub fn pictures()->PathBuf{
    std::env::var_os("USERPROFILE").map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir).join("Pictures").join("HorizonBoost")
}
pub fn videos()->PathBuf{
    std::env::var_os("USERPROFILE").map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir).join("Videos").join("HorizonBoost")
}
pub fn capture_async()->Receiver<Result<PathBuf,String>>{
    let (tx,rx)=mpsc::channel();
    std::thread::spawn(move||{
        let result=(||{
            let exe=ffmpeg_exe().ok_or("FFmpeg is not installed")?;
            let folder=pictures();
            std::fs::create_dir_all(&folder).map_err(|e|e.to_string())?;
            let target=folder.join(format!("HBoost_{}.png",stamp()));
            let status=Command::new(exe).args([
                "-hide_banner","-loglevel","error","-y",
                "-f","gdigrab","-draw_mouse","1","-i","desktop",
                "-frames:v","1","-update","1"
            ]).arg(&target).creation_flags(NO_WINDOW)
                .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
                .status().map_err(|e|e.to_string())?;
            if status.success()&&target.metadata().map(|m|m.len()>500).unwrap_or(false)
                {Ok(target)}else{Err("Screenshot capture failed".into())}
        })();
        let _=tx.send(result);
    });rx
}
#[derive(Default)]
pub struct Recorder{
 child:Option<Child>,target:Option<PathBuf>,pub last_file:Option<PathBuf>,
 finishing:Option<Receiver<Result<PathBuf,String>>>,join:Option<std::thread::JoinHandle<()>>,
 started:Option<std::time::Instant>,encoder:Option<&'static str>,
 remux:bool,pending:Option<Result<PathBuf,String>>,
 audio:Vec<crate::audio_capture::Worker>,audio_requested:usize,
 progress:Option<std::sync::Arc<std::sync::Mutex<Progress>>>,
}
impl Recorder{
 pub fn active(&mut self)->bool{
  if let Some(child)=self.child.as_mut(){
   match child.try_wait(){Ok(None)=>return true,Ok(Some(s))=>{
    self.child=None;self.started=None;for w in &self.audio{w.cancel();}self.audio.clear();
    if let Some(path)=self.target.take(){self.pending=Some(Err(format!("Recorder exited ({s}); MKV retained if present: {}",path.display())));}
   },Err(e)=>{self.pending=Some(Err(e.to_string()));return true;}}
  }false
 }
 pub fn busy(&mut self)->bool{self.active()||self.finishing.is_some()}
 pub fn audio_requested(&self)->usize{if self.child.is_some()||self.finishing.is_some(){self.audio_requested}else{0}}
 pub fn finishing(&self)->bool{self.finishing.is_some()}
 pub fn elapsed(&self)->Option<f64>{self.started.map(|v|v.elapsed().as_secs_f64())}
 pub fn encoder(&self)->Option<&'static str>{self.encoder}
 pub fn progress(&self)->Option<Progress>{self.progress.as_ref()?.lock().ok().map(|v|*v)}
 pub fn start(&mut self,width:u32,fps:u32,hardware:bool,rate:u32,remux:bool,audio:crate::audio_capture::Options,gains:[f32;2])->Result<PathBuf,String>{
  if self.busy(){return Err("Recording or finalization already active".into());}
  if let Some(t)=self.join.take(){let _=t.join();}
  let exe=ffmpeg_exe().ok_or("FFmpeg not found")?;
  let folder=videos();std::fs::create_dir_all(&folder).map_err(|e|e.to_string())?;
  let output=folder.join(format!("HBoost_{}.mkv",crate::media_tools::stamp()));
  let mut child=Command::new(exe).args(["-hide_banner","-loglevel","error","-progress","pipe:1","-stats_period","1","-nostats","-n","-f","gdigrab","-framerate",&fps.clamp(15,60).to_string(),"-draw_mouse","1","-i","desktop","-vf",&format!("scale={}:-2:flags=fast_bilinear",if width>=1600{1920}else{1280})])
   .args(crate::media_tools::encoder_args(hardware,rate)).arg(&output).creation_flags(NO_WINDOW).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null())
   .spawn().map_err(|e|e.to_string())?;
  let progress=std::sync::Arc::new(std::sync::Mutex::new(Progress::default()));if let Some(out)=child.stdout.take(){let p=progress.clone();std::thread::spawn(move||{use std::io::BufRead;for line in std::io::BufReader::new(out).lines().map_while(Result::ok){if let Ok(mut v)=p.lock(){v.update(&line);}}});}self.progress=Some(progress);
  self.child=Some(child);self.target=Some(output.clone());let began=std::time::Instant::now();self.started=Some(began);
  self.audio.clear();self.audio_requested=usize::from(audio.system)+usize::from(audio.microphone);
  for (on,kind) in [(audio.system,crate::audio_capture::Kind::System),(audio.microphone,crate::audio_capture::Kind::Microphone)]{if on{if let Ok(w)=crate::audio_capture::Worker::start_gain(&output,kind,began,if kind==crate::audio_capture::Kind::System{gains[0]}else{gains[1]}){self.audio.push(w);}}}
  self.encoder=Some(if hardware{"AMD AMF requested"}else{"CPU x264 requested"});self.remux=remux;self.pending=None;Ok(output)
 }
 pub fn stop(&mut self)->Option<PathBuf>{
  let mut child=self.child.take()?;let path=self.target.take()?;self.started=None;
  if let Some(mut stdin)=child.stdin.take(){let _=stdin.write_all(b"q\n");}
  let result_path=path.clone();let remux=self.remux;let(tx,rx)=mpsc::channel();
  let audio=std::mem::take(&mut self.audio);for w in &audio{w.cancel();}let requested=self.audio_requested;
  self.join=Some(std::thread::spawn(move||{
   let mut done=false;for _ in 0..80{if matches!(child.try_wait(),Ok(Some(_))){done=true;break;}std::thread::sleep(std::time::Duration::from_millis(50));}
   if !done{let _=child.kill();let _=child.wait();}
   let mut tracks=Vec::new();let mut errors=Vec::new();for w in audio{match w.finish(){Ok(v)=>tracks.push(v),Err(e)=>errors.push(e)}}
   let result=crate::media_tools::probe(&path).and_then(|_|{
    let mut selected=path.clone();
    if requested>0{
     if tracks.len()!=requested{return Err(format!("Only {}/{} requested audio tracks completed; source MKV and any WAV retained: {}",tracks.len(),requested,errors.join("; ")));}
     let stem=path.file_stem().and_then(|s|s.to_str()).ok_or("Recording name invalid")?;
     let dest=path.with_file_name(format!("{stem}_audio.mkv"));
     crate::audio_capture::mux(&path,&tracks,&dest).map_err(|e|format!("{e}; original MKV and WAV tracks retained"))?;selected=dest;
    }
    if remux{let mp4=selected.with_extension("mp4");crate::media_tools::remux(&selected,&mp4).map(|_|mp4).map_err(|e|format!("{e}; source MKV retained"))}else{Ok(selected)}
   });
   let _=tx.send(result);
  }));
  self.finishing=Some(rx);Some(result_path)
 }
 pub fn poll(&mut self)->Option<Result<PathBuf,String>>{
  let _=self.active();
  if let Some(v)=self.pending.take(){return Some(v);}
  let v=match self.finishing.as_ref()?.try_recv(){Ok(v)=>v,Err(mpsc::TryRecvError::Empty)=>return None,Err(_)=>Err("Recorder finalization worker stopped".into())};
  self.finishing=None;if let Some(t)=self.join.take(){let _=t.join();}
  if let Ok(path)=&v{self.last_file=Some(path.clone());}Some(v)
 }
}
impl Drop for Recorder{fn drop(&mut self){self.stop();if let Some(t)=self.join.take(){let _=t.join();}}}
pub fn open_folder(which:&str){
    let folder=if which=="videos"{videos()}else{pictures()};
    if std::fs::create_dir_all(&folder).is_ok(){
        let _=Command::new("explorer.exe").arg(folder)
            .creation_flags(NO_WINDOW).spawn();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn paths_under_user(){assert!(pictures().to_string_lossy().contains("HorizonBoost"));}
}
#[derive(Clone,Copy,Default,Debug)]pub struct Progress{pub frames:Option<u64>,pub drops:Option<u64>,pub duplicates:Option<u64>,pub fps:Option<f32>}
impl Progress{fn update(&mut self,line:&str){if let Some((k,v))=line.split_once('='){match k{"frame"=>self.frames=v.trim().parse().ok(),"drop_frames"=>self.drops=v.trim().parse().ok(),"dup_frames"=>self.duplicates=v.trim().parse().ok(),"fps"=>self.fps=v.trim().parse::<f32>().ok().filter(|x|x.is_finite()&&*x>=0.),_=>{}}}}}
#[cfg(test)]mod progress_tests{#[test]fn ffmpeg_progress_unknown_not_zero(){let mut p=super::Progress::default();assert_eq!(p.drops,None);p.update("drop_frames=7");p.update("frame=95");p.update("fps=NaN");assert_eq!(p.drops,Some(7));assert_eq!(p.frames,Some(95));assert_eq!(p.fps,None);}}
