//! Optional local 30-second replay buffer. Uses a bounded 9x5s TS rotating
//! file ring on DISK (not 9 raw D3D textures). Strictly opt-in, no hooks.
//! The active/latest segment is never copied while FFmpeg is writing it.
//! Six completed segments are copied to a private snapshot on a background
//! thread, muxed into MP4 and kept in the user's Videos/HorizonBoost.
//! No audio: desktop-only video. No claim of game-engine frame generation.
use std::{
    fs,io::Write,path::{Path,PathBuf},process::{Child,Command,Stdio},
    sync::mpsc::{self,Receiver},thread,time::{SystemTime,UNIX_EPOCH},
};
use std::os::windows::{process::CommandExt,fs::MetadataExt};
const NO_WINDOW:u32=0x08000000;
const SEGMENT_SECONDS:u32=5;
const RING_SEGMENTS:usize=9;
const COMPLETE_REQUIRED:usize=6;
pub fn replay_seconds(v:u32)->u32{if (15..=120).contains(&v)&&v%5==0{v}else{30}}
fn needed(v:u32)->usize{(replay_seconds(v)/SEGMENT_SECONDS) as usize}
fn ring_count(v:u32)->usize{needed(v)+3}
fn timestamp()->u128{SystemTime::now().duration_since(UNIX_EPOCH)
    .unwrap_or_default().as_millis()}
fn root()->PathBuf{crate::settings::folder().join("instant_replay")}
fn segment_path(session:&Path)->PathBuf{session.join("part%02d.ts")}
fn valid_segment(path:&Path)->bool{
    path.extension().is_some_and(|ext|ext=="ts")&&
    path.file_name().and_then(|f|f.to_str())
        .is_some_and(|n|n.len()==9&&n.starts_with("part")&&
            n[4..6].chars().all(|c|c.is_ascii_digit())&&n.ends_with(".ts"))&&
    fs::symlink_metadata(path).is_ok_and(|meta|meta.is_file()&&meta.file_attributes()&0x400==0&&(1024..=512*1024*1024).contains(&meta.len()))
}
fn completed_segments(session:&Path,required:usize)->Result<Vec<PathBuf>,String>{
    if !fs::symlink_metadata(session).is_ok_and(|m|m.is_dir()&&m.file_attributes()&0x400==0){return Err("Replay session directory is unavailable or redirected".into());}
    let mut recorded=fs::read_dir(session).map_err(|e|e.to_string())?
        .take(64).filter_map(|entry|{
            let path=entry.ok()?.path();
            if !valid_segment(&path){return None;}
            let when=path.metadata().ok()?.modified().ok()?;
            Some((when,path))
        }).collect::<Vec<_>>();
    // File modified date determines the current circular position. In ties
    // prefer stable filename. Never concatenate the live partially-written TS.
    recorded.sort_by(|a,b|a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    if recorded.len()<=required{
        return Err(format!("Replay warming up: {}/{} completed 5s segments; keep it running",
            recorded.len().saturating_sub(1),required));
    }
    if recorded.windows(2).any(|w|w[0].0==w[1].0){return Err("Replay segment order ambiguous; retry after next segment".into());}
    recorded.pop(); // newest segment is actively receiving FFmpeg packets
    Ok(recorded.into_iter().rev().take(required)
        .map(|(_,p)|p).collect::<Vec<_>>()
        .into_iter().rev().collect())
}
fn encode_options(width:u32,fps:u32,hardware:bool,seconds:u32,rate:u32)->Vec<String>{
    let mut args=vec![
        "-hide_banner".into(),"-loglevel".into(),"error".into(),
        "-y".into(),"-f".into(),"gdigrab".into(),
        "-framerate".into(),fps.clamp(15,60).to_string(),
        "-draw_mouse".into(),"1".into(),"-i".into(),"desktop".into(),
        "-vf".into(),format!("scale={}:-2:flags=fast_bilinear",
            if width>=1600{1920}else{1280}),
    ];
    args.extend(crate::media_tools::encoder_args(hardware,rate));
    args.extend(["-g","30"].map(str::to_owned));
    args.extend([
        "-pix_fmt","yuv420p","-force_key_frames","expr:gte(t,n_forced*5)",
        "-f","segment","-segment_time","5",
    ].map(str::to_owned));
    args.extend(["-segment_wrap".into(),ring_count(seconds).to_string()]);
    args.extend(["-segment_format","mpegts","-reset_timestamps","1"].map(str::to_owned));
    args
}
fn snapshot(session:PathBuf,ffmpeg:PathBuf,required:usize)->Result<PathBuf,String>{
    let segments=completed_segments(&session,required)?;
    let temp=root().join(format!("export-{}",crate::media_tools::stamp()));
    fs::create_dir_all(&temp).map_err(|e|e.to_string())?;
    let result=(||{
        let mut playlist=String::new();let mut total_bytes=0u64;
        for (i,source) in segments.iter().enumerate(){
            let target=temp.join(format!("part{i:02}.ts"));
            if !valid_segment(source){return Err("Replay segment changed before snapshot".into());}
            let before=fs::symlink_metadata(source).map_err(|e|e.to_string())?;
            total_bytes=total_bytes.checked_add(before.len()).ok_or("Replay snapshot size overflow")?;
            if total_bytes>1024*1024*1024{return Err("Replay export exceeds 1 GiB safe snapshot budget".into());}
            fs::copy(source,&target).map_err(|e|format!("Copy replay segment: {e}"))?;
            let after=fs::symlink_metadata(source).map_err(|e|e.to_string())?;
            if !valid_segment(source)||before.len()!=after.len()||before.modified().ok()!=after.modified().ok()||fs::metadata(&target).map_err(|e|e.to_string())?.len()!=before.len(){
                return Err("Replay ring rotated during snapshot; retry".into());
            }
            playlist.push_str(&format!("file 'part{i:02}.ts'\n"));
        }
        fs::write(temp.join("manifest.txt"),playlist)
            .map_err(|e|format!("Create replay manifest: {e}"))?;
        let dest_dir=crate::record::videos();
        fs::create_dir_all(&dest_dir).map_err(|e|e.to_string())?;
        let target=dest_dir.join(format!("HBoost_InstantReplay_{}.mp4",crate::media_tools::stamp()));
        let status=crate::media_tools::bounded(Command::new(ffmpeg).args([
            "-hide_banner","-loglevel","error","-n","-f","concat","-safe","1","-i","manifest.txt","-c","copy","-movflags","+faststart"
        ]).arg(&target).current_dir(&temp),std::time::Duration::from_secs(90));
        if status.is_ok()&&crate::media_tools::probe(&target).is_ok(){Ok(target)}
        else{let _=fs::remove_file(&target);Err("Replay MP4 validation failed; no saved clip".into())}
    })();
    let _=fs::remove_dir_all(&temp);
    result
}
#[derive(Default)]
pub struct RollingReplay{
    child:Option<Child>,
    folder:Option<PathBuf>,
    saving:Option<Receiver<Result<PathBuf,String>>>,
    pub last_file:Option<PathBuf>,
    seconds:u32,
    stopping:Option<thread::JoinHandle<()>>,
    export_thread:Option<thread::JoinHandle<()>>,
    retired:Option<PathBuf>,
    limit_bytes:u64,
    last_budget:Option<std::time::Instant>,
    bytes:u64,
    failure:Option<String>,
}
impl RollingReplay{
    pub fn is_buffering(&self)->bool{self.child.is_some()}
    pub fn active(&mut self)->bool{
        match self.child.as_mut(){
            Some(child)=>match child.try_wait(){
                Ok(Some(s))=>{self.child=None;self.retired=self.folder.take();self.failure=Some(format!("Replay encoder exited ({s})"));false},
                Err(_)=>true,
                Ok(None)=>true,
            },
            None=>false
        }
    }
    pub fn seconds(&self)->u32{replay_seconds(self.seconds)}
    pub fn ready(&self)->bool{self.folder.as_ref().is_some_and(|p|completed_segments(p,needed(self.seconds())).is_ok())}
    pub fn saving(&self)->bool{self.saving.is_some()}
    pub fn bytes(&self)->u64{self.bytes}
    pub fn busy(&mut self)->bool{self.active()||self.saving.is_some()||self.stopping.as_ref().is_some_and(|t|!t.is_finished())}
    fn cleanup_retired(&mut self){
        if self.saving.is_none()&&self.stopping.as_ref().is_none_or(|t|t.is_finished()){
            if let Some(t)=self.stopping.take(){let _=t.join();}
            if let Some(p)=self.retired.take(){let _=fs::remove_dir_all(p);}
        }
    }
    pub fn start(&mut self,width:u32,fps:u32,hardware:bool,seconds:u32,rate:u32,limit:u32)->Result<(),String>{
        self.cleanup_retired();
        let estimated=(ring_count(seconds) as u64*5*crate::media_tools::bitrate(rate) as u64*1_000_000/8)*12/10;
        let limit_bytes=crate::media_tools::replay_limit(limit) as u64*1024*1024;
        if crate::suite_finish::free_bytes(&root()).is_some_and(|n|n<estimated.saturating_add(512*1024*1024)){return Err("Not enough free disk for requested replay ring and reserve".into());}
        if estimated>limit_bytes{return Err("Requested replay duration/bitrate may exceed disk budget; lower bitrate or raise budget".into());}
        if self.stopping.as_ref().is_some_and(|t|!t.is_finished()){return Err("Previous replay is still stopping".into());}
        if let Some(t)=self.stopping.take(){let _=t.join();}
        if self.saving.is_some(){return Err("Previous replay export still running".into());}
        if self.active(){return Err("Instant replay already buffering".into());}
        self.seconds=replay_seconds(seconds);self.limit_bytes=limit_bytes;self.bytes=0;self.last_budget=None;self.failure=None;
        if let Some(old)=self.folder.take(){let _=fs::remove_dir_all(old);}
        let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg is missing. Install FFmpeg first")?;
        let session=root().join(format!("session-{}",crate::media_tools::stamp()));
        fs::create_dir_all(&session).map_err(|e|e.to_string())?;
        let mut args=encode_options(width,fps,hardware,self.seconds(),rate);
        args.push(segment_path(&session).to_string_lossy().into_owned());
        let child=Command::new(exe).args(&args).creation_flags(NO_WINDOW)
            .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null())
            .spawn().map_err(|e|{let _=fs::remove_dir_all(&session);format!("Instant replay recorder did not launch: {e}")})?;
        self.folder=Some(session);
        self.child=Some(child);
        Ok(())
    }
    pub fn save_last_30s(&mut self)->Result<(),String>{self.save_window(self.seconds())}
    pub fn save_window(&mut self,seconds:u32)->Result<(),String>{
        if !self.active(){return Err("Enable replay buffering first".into());}
        if self.saving.is_some(){
            return Err("A replay is already being saved; check the result before saving again".into());
        }
        let session=self.folder.as_ref().ok_or("Replay ring is unavailable")?.clone();
        let exe=crate::record::ffmpeg_exe().ok_or("FFmpeg unavailable")?;
        // Refuse early instead of telling user an incomplete 12-second clip
        // is "the last 30 seconds".
        if seconds<5||seconds>self.seconds()||seconds%5!=0{return Err("Replay clip must use completed 5-second segments".into());}
        let required=(seconds/SEGMENT_SECONDS)as usize;
        completed_segments(&session,required)?;
        let (tx,rx)=mpsc::channel();
        self.export_thread=Some(thread::spawn(move||{let _=tx.send(snapshot(session,exe,required));}));
        self.saving=Some(rx);
        Ok(())
    }
    pub fn poll(&mut self)->Option<Result<PathBuf,String>>{
        let _=self.active();self.cleanup_retired();
        if let Some(e)=self.failure.take(){return Some(Err(e));}
        if self.child.is_some()&&self.last_budget.is_none_or(|t|t.elapsed()>=std::time::Duration::from_secs(1)){
            self.last_budget=Some(std::time::Instant::now());
            if let Some(dir)=&self.folder{
                self.bytes=fs::read_dir(dir).ok().into_iter().flatten().flatten().filter_map(|e|e.metadata().ok()).map(|m|m.len()).sum();
                if crate::suite_finish::free_bytes(dir).is_some_and(|n|n<512*1024*1024){self.stop();return Some(Err("Replay stopped: free disk below 512 MiB".into()));}
                if self.bytes>self.limit_bytes{self.stop();return Some(Err("Replay stopped: disk budget exceeded".into()));}
            }
        }
        let reply=match self.saving.as_ref()?.try_recv(){
            Ok(v)=>v,
            Err(mpsc::TryRecvError::Empty)=>return None,
            Err(mpsc::TryRecvError::Disconnected)=>Err("Replay export worker stopped".into()),
        };
        self.saving=None;if let Some(t)=self.export_thread.take(){let _=t.join();}self.cleanup_retired();
        if let Ok(file)=&reply{self.last_file=Some(file.clone());}
        Some(reply)
    }
    pub fn stop(&mut self){
        if let Some(mut child)=self.child.take(){
            if let Some(mut stdin)=child.stdin.take(){let _=stdin.write_all(b"q\n");}
            self.retired=self.folder.take();
            self.stopping=Some(thread::spawn(move||{
                let mut done=false;
                for _ in 0..50{
                    if matches!(child.try_wait(),Ok(Some(_))){done=true;break;}
                    thread::sleep(std::time::Duration::from_millis(100));
                }
                if !done{let _=child.kill();let _=child.wait();}
                // Owner removes ring after both stop and export have finished.
            }));
        }
    }
}
impl Drop for RollingReplay{fn drop(&mut self){self.stop();if let Some(t)=self.stopping.take(){let _=t.join();}if let Some(t)=self.export_thread.take(){let _=t.join();}if let Some(p)=self.retired.take(){let _=fs::remove_dir_all(p);}if let Some(p)=self.folder.take(){let _=fs::remove_dir_all(p);}}}
pub fn synthetic_proof()->Result<String,String>{
    let dir=std::env::temp_dir().join(format!("hb-replay-proof-{}",crate::media_tools::stamp()));
    fs::create_dir(&dir).map_err(|e|e.to_string())?;
    let result=(||{
        let ff=crate::record::ffmpeg_exe().ok_or("FFmpeg missing")?;
        let options=encode_options(1280,30,true,15,4);
        let offset=options.iter().position(|a|a=="-c:v").ok_or("Encoder arguments missing")?;
        crate::media_tools::bounded(Command::new(&ff).args(["-hide_banner","-loglevel","error","-f","lavfi","-i","testsrc2=size=320x180:rate=30","-t","21"]).args(&options[offset..]).arg(segment_path(&dir)),std::time::Duration::from_secs(60))?;
        let segments=completed_segments(&dir,needed(15))?;
        let mut manifest=String::new();
        for segment in segments{manifest.push_str(&format!("file '{}'\n",segment.file_name().and_then(|s|s.to_str()).ok_or("Invalid synthetic segment")?));}
        fs::write(dir.join("manifest.txt"),manifest).map_err(|e|e.to_string())?;
        let output=dir.join("clip.mp4");
        crate::media_tools::bounded(Command::new(&ff).args(["-hide_banner","-loglevel","error","-f","concat","-safe","1","-i","manifest.txt","-c","copy"]).arg(&output).current_dir(&dir),std::time::Duration::from_secs(30))?;
        let info=crate::media_tools::probe(&output)?;
        if (info.duration-15.).abs()>0.25{return Err(format!("Replay duration {:.3}",info.duration));}
        crate::media_tools::bounded(Command::new(&ff).args(["-v","error","-i"]).arg(&output).args(["-f","null","NUL"]),std::time::Duration::from_secs(30))?;
        Ok(format!("REPLAY_NEW_OPTIONS=PASS DURATION={:.3} AMF=true DECODE=true SYNTHETIC=true",info.duration))
    })();let _=fs::remove_dir_all(dir);result
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn replay_segments_are_bounded_and_optional(){
        assert_eq!(SEGMENT_SECONDS,5);
        assert_eq!(RING_SEGMENTS,9);assert_eq!(ring_count(15),6);assert_eq!(ring_count(60),15);assert_eq!(needed(99),6);assert_eq!(needed(120),24);assert_eq!(ring_count(120),27);assert_eq!(needed(45),9);
        assert_eq!(COMPLETE_REQUIRED*SEGMENT_SECONDS as usize,30);
        let options=encode_options(1280,30,true,30,4);
        assert!(options.windows(2).any(|x|x==["-segment_wrap","9"]));
        assert!(options.windows(2).any(|x|x==["-c:v","h264_amf"]));
        assert!(options.windows(2).any(|x|x==["-forced_idr","1"]));
        assert!(!options.iter().any(|x|x=="-ftrace"));
    }
    #[test]fn early_incomplete_replay_must_not_be_saved(){
        let dir=std::env::temp_dir().join(format!("hboost-replay-unit-{}",timestamp()));
        fs::create_dir_all(&dir).unwrap();
        for i in 0..4{
            fs::write(dir.join(format!("part{i:02}.ts")),vec![42u8;2048]).unwrap();
        }
        assert!(completed_segments(&dir,6).unwrap_err().contains("warming up"));
        fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(test)]mod ordering_tests{use super::*;#[test]fn rejects_short_files_and_directories(){let d=std::env::temp_dir().join(format!("hb-ring-{}",crate::media_tools::stamp()));fs::create_dir(&d).unwrap();let p=d.join("part00.ts");fs::write(&p,[1u8;32]).unwrap();assert!(!valid_segment(&p));fs::write(&p,[1u8;2048]).unwrap();assert!(valid_segment(&p));fs::create_dir(d.join("part01.ts")).unwrap();assert!(!valid_segment(&d.join("part01.ts")));assert!(completed_segments(&d,3).is_err());fs::remove_dir_all(d).unwrap();}}


#[cfg(test)]mod v95_snapshot_budget_tests{use super::*;#[test]fn replay_ring_remains_bounded(){assert_eq!(needed(15),3);assert_eq!(needed(30),6);assert_eq!(needed(60),12);assert_eq!(ring_count(60),15);}}
