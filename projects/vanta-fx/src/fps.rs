//! Non-injected real game frametime telemetry using Intel PresentMon 2.6 ETW.
 //! No synthetic metrics. Handles missing/blocked ETW sessions by leaving values unavailable.
use std::{
    collections::VecDeque,
    io::{BufRead,BufReader},
    path::PathBuf,
    process::{Child,Command,Stdio},
    time::{Duration,Instant},
};
use std::os::windows::process::CommandExt;
const CREATE_NO_WINDOW:u32=0x0800_0000;
pub fn find_presentmon()->Option<PathBuf>{
    let exe=std::env::current_exe().ok()?;
    for root in exe.ancestors().take(8){
        let path=root.join("engines").join("PresentMon").join("PresentMon-2.6.0-x64.exe");
        if path.is_file(){return Some(path);}
    }
    None
}
/// PresentMon ETW-reported presentation mode. Never inferred from HWND size.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum PresentPath{
    DwmComposed,
    HardwareComposed,
    IndependentFlip,
    LegacyExclusive,
    Unknown,
}
impl PresentPath{
    pub fn label(self)->&'static str{
        match self{
            Self::DwmComposed=>"DWM composited",
            Self::HardwareComposed=>"Hardware-composed flip",
            Self::IndependentFlip=>"Independent flip",
            Self::LegacyExclusive=>"Hardware legacy flip",
            Self::Unknown=>"Unrecognized ETW mode",
        }
    }
    pub fn glass_note(self)->&'static str{
        match self{
            Self::DwmComposed=>"Glass overlay is eligible for DWM composition (real visual result still needs checking).",
            Self::HardwareComposed=>"Hardware composition detected; glass visibility depends on driver/MPO plane availability.",
            Self::IndependentFlip=>"Independent flip detected; an overlay may trigger DWM composition or stay behind the game.",
            Self::LegacyExclusive=>"Legacy hardware flip detected. External windows are commonly hidden behind the game.",
            Self::Unknown=>"Present mode not recognized; fullscreen compatibility cannot be inferred.",
        }
    }
}
fn parse_present_path(raw:&str)->Option<PresentPath>{
    let s=raw.trim().trim_matches('"').to_ascii_lowercase();
    if s.is_empty()||s=="na"{return None;}
    if s.contains("legacy flip"){Some(PresentPath::LegacyExclusive)}
    else if s.contains("hardware composed"){Some(PresentPath::HardwareComposed)}
    else if s.contains("independent flip"){Some(PresentPath::IndependentFlip)}
    else if s.starts_with("composed:"){Some(PresentPath::DwmComposed)}
    else{Some(PresentPath::Unknown)}
}
#[derive(Clone,Copy,Default,Debug)]
pub struct Frame {
    pub ms:f32,
    pub gpu_ms:Option<f32>,
    pub latency_ms:Option<f32>,
    // Real PresentMon ETW optional timing, never calculated from game FPS.
    pub input_to_photon_ms:Option<f32>,
    pub click_to_photon_ms:Option<f32>,
    pub present_path:Option<PresentPath>,
}
#[derive(Clone,Copy,Debug)]
pub struct Stats {
    pub fps:f32,
    pub frametime_ms:f32,
    pub low_1_percent_fps:Option<f32>,
    pub gpu_ms:Option<f32>,
    pub latency_ms:Option<f32>,
    pub input_to_photon_ms:Option<f32>,
    pub click_to_photon_ms:Option<f32>,
    pub present_path:Option<PresentPath>,
}
// PresentMon is CSV; quoted Application names can contain commas and
// doubled quote escapes. A blind line.split(',') corrupts column positions.
fn csv_cells(line:&str)->Vec<String>{
    let mut fields=Vec::new();
    let mut cell=String::new();
    let mut chars=line.chars().peekable();
    let mut quoted=false;
    while let Some(ch)=chars.next(){
        match ch{
            '"' if quoted&&chars.peek()==Some(&'"')=>{
                chars.next();
                cell.push('"');
            },
            '"'=>quoted=!quoted,
            ',' if !quoted=>fields.push(std::mem::take(&mut cell)),
            _=>cell.push(ch)
        }
    }
    fields.push(cell);
    fields
}
fn field(headers:&[&str],row:&[&str],key:&str)->Option<f32>{
    let idx=headers.iter().position(|x|x.trim_matches('"').trim()==key)?;
    row.get(idx)?.trim().trim_matches('"').parse::<f32>().ok()
}
fn parse_row(headers:&[&str],line:&str)->Option<Frame>{
    let owned=csv_cells(line);
    let cells=owned.iter().map(String::as_str).collect::<Vec<_>>();
    let mode=headers.iter().position(|h|h.trim_matches('"')=="PresentMode")
        .and_then(|i|cells.get(i))
        .and_then(|raw|parse_present_path(raw));
    let ms=field(headers,&cells,"FrameTime")
        .or_else(||field(headers,&cells,"MsBetweenPresents"))
        .or_else(||field(headers,&cells,"MsBetweenDisplayChange"))?;
    // Filter missing/invalid/outlier presentation timestamps without making up FPS.
    if !ms.is_finite()||!(0.2..=1000.0).contains(&ms){return None;}
    Some(Frame {
        ms,
        gpu_ms:field(headers,&cells,"GPUTime")
            .or_else(||field(headers,&cells,"MsGPUTime"))
            .filter(|v|v.is_finite()&&*v>=0.0),
        latency_ms:field(headers,&cells,"DisplayLatency")
            .filter(|v|v.is_finite()&&(0.0..=2000.0).contains(v)),
        input_to_photon_ms:field(headers,&cells,"AllInputToPhotonLatency")
            .filter(|v|v.is_finite()&&(0.0..=2000.0).contains(v)),
        click_to_photon_ms:field(headers,&cells,"ClickToPhotonLatency")
            .filter(|v|v.is_finite()&&(0.0..=2000.0).contains(v)),
        present_path:mode,
    })
}
#[derive(Clone,Copy)]
struct TaggedFrame{frame:Frame,chain:u64,received:Instant}
fn parse_tagged(headers:&[&str],line:&str,pid:u32,received:Instant)->Option<TaggedFrame>{
    let row=csv_cells(line);
    let get=|name:&str|headers.iter().position(|h|h.trim_matches('"').trim()==name).and_then(|i|row.get(i)).map(|s|s.trim());
    if get("ProcessID")?.parse::<u32>().ok()?!=pid{return None;}
    let address=get("SwapChainAddress")?;
    let chain=if let Some(hex)=address.strip_prefix("0x").or_else(||address.strip_prefix("0X")){u64::from_str_radix(hex,16).ok()?}else{address.parse::<u64>().ok()?};
    if chain==0{return None;}
    Some(TaggedFrame{frame:parse_row(headers,line)?,chain,received})
}
#[derive(Default)]
pub struct PresentStats {
    pub child:Option<Child>,
    receiver:Option<crate::telemetry_ring::Rx<TaggedFrame,512>>,
    router:crate::swapchain_route::Router,
    pub stream_changed:bool,
    pending_stream_reset:bool,
    pub rejected_secondary:u64,
    pub rejected_stale:u64,
    queue_overflow:std::sync::Arc<std::sync::atomic::AtomicU64>,
    tracked_pid:Option<u32>,
    session_name:&'static str,
    pub frames:VecDeque<Frame>,
    pub new_frames:Vec<Frame>,
    last_frame:Option<Instant>,
    pub note:String,
    attempted:Option<(u32,Instant)>,
    cached_stats:std::cell::Cell<Option<(Instant,Stats)>>,
}
impl PresentStats{
    pub fn hud_meter()->Self{{let mut v=Self::default();v.session_name="HorizonBoostHudETW";v}}
    pub fn running(&mut self)->bool{
        match self.child.as_mut(){
            Some(p)=>match p.try_wait(){
                Ok(None)=>true,
                Ok(Some(code))=>{
                    self.child=None;self.note=format!("PresentMon stopped ({code}).");
                    false
                }
                Err(e)=>{
                    self.child=None;self.note=format!("PresentMon status unavailable: {e}");
                    false
                }
            },
            None=>false,
        }
    }
    pub fn track(&mut self,pid:Option<u32>,enabled:bool){
        let wanted=if enabled{pid}else{None};
        let running=self.running();
        if wanted==self.tracked_pid&&running{return;}
        if wanted==self.tracked_pid&&wanted.is_none(){return;}
        if wanted!=self.tracked_pid {
            self.stop();
            self.tracked_pid=wanted;
            self.frames.clear();self.last_frame=None;
        }
        let Some(new_pid)=wanted else{return;};
        if let Some((prior,last))=self.attempted {
            if new_pid==prior&&last.elapsed()<Duration::from_secs(35){return;}
        }
        self.attempted=Some((new_pid,Instant::now()));
        match self.start_process(new_pid){
            Ok(())=>self.note="ETW monitoring started · waiting for game presentations".into(),
            Err(e)=>self.note=format!("PresentMon unavailable: {e}"),
        }
    }
    fn start_process(&mut self,pid:u32)->Result<(),String>{
        let exe=find_presentmon().ok_or("PresentMon binary missing from engines/PresentMon")?;
        let mut child=Command::new(exe)
            .args(["--process_id",&pid.to_string(),
                "--output_stdout","--no_console_stats","--v2_metrics","--exclude_dropped",
                "--session_name",if self.session_name.is_empty(){"HorizonBoostETW"}else{self.session_name}])
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null())
            .spawn().map_err(|e|e.to_string())?;
        let stdout=child.stdout.take().ok_or("PresentMon stdout unavailable")?;
        let (tx,rx)=crate::telemetry_ring::channel::<TaggedFrame,512>();
        let overflow=std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));self.queue_overflow=overflow.clone();
        std::thread::spawn(move||{
            let mut headers:Vec<String>=Vec::new();
            for line in BufReader::new(stdout).lines(){
                let Ok(line)=line else{break};
                let s=line.trim_start_matches('\u{feff}').trim();
                if s.starts_with("Application,")||s.starts_with("\"Application\","){
                    headers=csv_cells(s);
                    continue;
                }
                if headers.is_empty(){continue;}
                let cols:Vec<&str>=headers.iter().map(String::as_str).collect();
                if let Some(frame)=parse_tagged(&cols,s,pid,Instant::now()){
                    match tx.push(frame){Ok(())=>{},Err(crate::telemetry_ring::PushError::Full)=>{overflow.fetch_add(1,std::sync::atomic::Ordering::Relaxed);},Err(crate::telemetry_ring::PushError::Closed)=>break}
                    // Queue-overflow samples are not game dropped frames.
                }
            }
        });
        // A restarted ETW session must not append to the old stream.
        self.pending_stream_reset|=self.router.selected.is_some();
        self.router=Default::default();self.frames.clear();self.new_frames.clear();
        self.last_frame=None;self.cached_stats.set(None);
        self.receiver=Some(rx);
        self.child=Some(child);
        Ok(())
    }
    pub fn poll(&mut self){
        self.new_frames.clear();
        self.stream_changed=std::mem::take(&mut self.pending_stream_reset);
        if let Some(rx)=&self.receiver {
            for _ in 0..500 {
                match rx.pop(){
                    Some(tagged)=>{
                        if tagged.received.elapsed()>Duration::from_secs(2){
                            self.rejected_stale=self.rejected_stale.saturating_add(1);continue;
                        }
                        let old=self.router.selected;
                        let accepted=self.router.observe(tagged.received,tagged.chain);
                        if old!=self.router.selected{
                            if old.is_some(){self.stream_changed=true;}
                            self.frames.clear();self.new_frames.clear();self.last_frame=None;self.cached_stats.set(None);
                        }
                        if !accepted{self.rejected_secondary=self.rejected_secondary.saturating_add(1);continue;}
                        self.new_frames.push(tagged.frame);
                        self.frames.push_back(tagged.frame);
                        if self.frames.len()>1200{self.frames.pop_front();}
                        self.last_frame=Some(tagged.received);
                    }
                    None=>break
                }
            }
        }
    }
    pub fn queue_overflow(&self)->u64{self.queue_overflow.load(std::sync::atomic::Ordering::Relaxed)}
    pub fn routed(&self)->bool{self.router.selected.is_some()}
    pub fn low_01_fps(&self)->Option<f32>{
        if self.last_frame?.elapsed()>Duration::from_secs(3)||self.frames.len()<1000{return None;}
        slowest_01(self.frames.iter().rev().take(1200).map(|f|f.ms))
    }
    pub fn snapshot_for(&self,pid:Option<u32>)->Option<Stats>{if pid.is_some()&&pid==self.tracked_pid{self.snapshot()}else{None}}
    pub fn snapshot(&self)->Option<Stats>{
        if self.last_frame?.elapsed()>Duration::from_secs(3) || self.frames.len()<8{
            return None;
        }
        if let Some((when,stats))=self.cached_stats.get(){
            if when.elapsed()<Duration::from_millis(250){return Some(stats);}
        }
        let recent=self.frames.iter().rev().take(180);
        let mut times=[0f32;180];
        let mut n=0;
        for frame in recent.clone(){times[n]=frame.ms;n+=1;}
        let ms=crate::metric_math::sum(&times[..n])/n as f32;
        if ms<=0.0{return None;}
        let low=if self.frames.len()>=100{
            let mut storage=[0f32;600];let mut count=0;
            for frame in self.frames.iter().rev().take(600){storage[count]=frame.ms;count+=1;}
            let frame_times=&mut storage[..count];
            frame_times.sort_unstable_by(|a,b|b.total_cmp(a));
            let tail=((frame_times.len() as f32)*0.01).ceil() as usize;
            let tail=tail.max(1);
            let average=crate::metric_math::sum(&frame_times[..tail])/tail as f32;
            if average>0.0{Some(1000.0/average)}else{None}
        }else{None};
        let mean=|extract:fn(&Frame)->Option<f32>|->Option<f32>{
            let (sum,n)=recent.clone().filter_map(extract).fold((0.0,0u32),|(s,n),v|(s+v,n+1));
            if n==0{None}else{Some(sum/n as f32)}
        };
        let stats=Stats {
            fps:1000.0/ms,
            frametime_ms:ms,
            low_1_percent_fps:low,
            gpu_ms:mean(|f|f.gpu_ms),
            latency_ms:mean(|f|f.latency_ms),
            input_to_photon_ms:mean(|f|f.input_to_photon_ms),
            click_to_photon_ms:mean(|f|f.click_to_photon_ms),
            present_path:recent.clone().find_map(|f|f.present_path),
        };
        self.cached_stats.set(Some((Instant::now(),stats)));
        Some(stats)
    }
    pub fn stop(&mut self){
        if let Some(mut process)=self.child.take(){
            let _=process.kill();
            let _=process.wait();
        }
        self.receiver=None;self.queue_overflow=Default::default();
        self.pending_stream_reset|=self.router.selected.is_some();
        self.router=Default::default();self.rejected_secondary=0;self.rejected_stale=0;
        self.tracked_pid=None;
        self.frames.clear();
        self.new_frames.clear();
        self.last_frame=None;
        self.cached_stats.set(None);
    }
}
impl Drop for PresentStats{fn drop(&mut self){self.stop();}}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn presentmon_quoted_csv_does_not_misalign_columns(){
        let h=csv_cells("Application,FrameTime,AllInputToPhotonLatency,PresentMode");
        let cols=h.iter().map(String::as_str).collect::<Vec<_>>();
        let valid=parse_row(&cols,
            "\"game, \"\"remastered\"\".exe\",16.66,42.0,\"Hardware: Independent Flip\"").unwrap();
        assert!((valid.ms-16.66).abs()<0.001);
        assert_eq!(valid.input_to_photon_ms,Some(42.0));
        assert_eq!(valid.present_path,Some(PresentPath::IndependentFlip));
        let names=csv_cells("\"A, B.exe\",\"escaped \"\"quote\"\"\"");
        assert_eq!(names[0],"A, B.exe");
        assert_eq!(names[1],"escaped \"quote\"");
    }
    #[test]fn parse_valid_metrics(){
        let h=["Application","ProcessID","MsBetweenPresents","MsGPUTime","DisplayLatency"];
        let r=parse_row(&h,"game.exe,4123,16.666,6.2,24.5").unwrap();
        assert!((r.ms-16.666).abs()<0.001);
        assert_eq!(r.gpu_ms,Some(6.2));
        assert_eq!(r.input_to_photon_ms,None);
        assert_eq!(r.click_to_photon_ms,None);
    }
    #[test]fn presentmon_v2_real_csv_schema(){
        // Header and column order from PresentMon 2.6.0 actual --v2_metrics stdout.
        let h="Application,ProcessID,SwapChainAddress,PresentRuntime,SyncInterval,PresentFlags,AllowsTearing,PresentMode,CPUStartTime,FrameTime,CPUBusy,CPUWait,GPULatency,GPUTime,GPUBusy,GPUWait,DisplayLatency,DisplayedTime,AnimationError,AnimationTime,MsFlipDelay,AllInputToPhotonLatency,ClickToPhotonLatency";
        let cols=h.split(',').collect::<Vec<_>>();
        let line="ExampleGame.exe,12740,0x123,DXGI,0,0,0,Composed: Flip,87.5294,31.7492,31.5552,0.1940,25.5794,11.9010,7.5460,4.3550,49.8437,26.6446,NA,87.5294,NA,NA,NA";
        let frame=parse_row(&cols,line).expect("PresentMon 2.6 row");
        assert!((frame.ms-31.7492).abs()<0.0001);
        assert_eq!(frame.gpu_ms,Some(11.9010));
        assert_eq!(frame.latency_ms,Some(49.8437));
        assert_eq!(frame.input_to_photon_ms,None);
        assert_eq!(frame.click_to_photon_ms,None);
    }
    #[test]fn classifies_etw_presentation_modes_not_hwnd_heuristics(){
        let modes=[
            ("Composed: Flip",PresentPath::DwmComposed),
            ("Hardware Composed: Independent Flip",PresentPath::HardwareComposed),
            ("Hardware: Independent Flip",PresentPath::IndependentFlip),
            ("Hardware: Legacy Flip",PresentPath::LegacyExclusive),
        ];
        for (text,expected) in modes{
            assert_eq!(parse_present_path(text),Some(expected));
            assert!(!expected.glass_note().is_empty());
        }
        assert_eq!(parse_present_path(""),None);
        let h=["Application","FrameTime","PresentMode"];
        let f=parse_row(&h,"TotK.exe,16.67,Hardware: Independent Flip").unwrap();
        assert_eq!(f.present_path,Some(PresentPath::IndependentFlip));
    }
    #[test]fn optional_real_input_to_photon_metrics_are_sparse_not_guessed(){
        let headers=["Application","FrameTime","DisplayLatency",
            "AllInputToPhotonLatency","ClickToPhotonLatency"];
        let frame=parse_row(&headers,"game.exe,16.7,12.5,42.3,51.1").unwrap();
        assert_eq!(frame.input_to_photon_ms,Some(42.3));
        assert_eq!(frame.click_to_photon_ms,Some(51.1));
        let missing=parse_row(&headers,"game.exe,16.7,12.5,NA,NA").unwrap();
        assert_eq!(missing.input_to_photon_ms,None);
        assert_eq!(missing.click_to_photon_ms,None);
        let bad=parse_row(&headers,"game.exe,16.7,12.5,-2.0,NaN").unwrap();
        assert_eq!(bad.input_to_photon_ms,None);
        assert_eq!(bad.click_to_photon_ms,None);
    }
    #[test]fn does_not_invent_metrics(){
        let h=["Application","MsBetweenPresents"];
        assert!(parse_row(&h,"test,NA").is_none());
        assert!(parse_row(&h,"test,-20").is_none());
        let s=PresentStats::default();
        assert!(s.snapshot().is_none());
    }
    #[test]fn strict_pid_and_swapchain_metadata(){
        let headers=["Application","ProcessID","SwapChainAddress","FrameTime"];
        let t=Instant::now();
        assert_eq!(parse_tagged(&headers,"game,77,0xABC,16.7",77,t).unwrap().chain,2748);
        assert!(parse_tagged(&headers,"game,88,0xABC,16.7",77,t).is_none());
        assert!(parse_tagged(&headers,"game,77,0x0,16.7",77,t).is_none());
        assert!(parse_tagged(&["Application","FrameTime"],"game,16.7",77,t).is_none());
    }
    #[test]fn stream_switch_resets_statistics_and_pending_benchmark_samples(){
        let mut s=PresentStats::default();let (tx,rx)=crate::telemetry_ring::channel::<TaggedFrame,512>();s.receiver=Some(rx);
        let n=Instant::now()-Duration::from_millis(1500);
        let frame=Frame{ms:16.666,..Default::default()};
        for _ in 0..60{tx.push(TaggedFrame{frame,chain:1,received:n}).unwrap();}
        let one=n+Duration::from_millis(1001);
        for _ in 0..10{tx.push(TaggedFrame{frame,chain:1,received:one}).unwrap();}
        for _ in 0..10{tx.push(TaggedFrame{frame:Frame{ms:2.,..Default::default()},chain:2,received:one}).unwrap();}
        s.poll();assert_eq!(s.frames.len(),10);assert!((s.snapshot().unwrap().fps-60.).abs()<0.1);
        s.stop();s.poll();assert!(s.stream_changed&&s.snapshot().is_none()&&s.new_frames.is_empty());
    }
    #[test]fn stats_from_real_samples(){
        let mut s=PresentStats::default();
        for _ in 0..300 {
            s.frames.push_back(Frame{ms:16.666,gpu_ms:Some(5.0),latency_ms:Some(23.0),
                input_to_photon_ms:Some(40.0),click_to_photon_ms:None,present_path:None});
        }
        s.last_frame=Some(Instant::now());
        let x=s.snapshot().unwrap();
        assert!((x.fps-60.0).abs()<0.1);
        assert!((x.low_1_percent_fps.unwrap()-60.0).abs()<0.1);
        assert_eq!(x.input_to_photon_ms,Some(40.0));
        assert_eq!(x.click_to_photon_ms,None);
    }
}
fn slowest_01(values:impl Iterator<Item=f32>)->Option<f32>{
 let mut count=0usize;let mut top=[0f32;2];
 for v in values.take(1200){
  if !v.is_finite()||v<=0.{continue;}
  count+=1;
  if v>=top[0]{top[1]=top[0];top[0]=v;}else if v>top[1]{top[1]=v;}
 }
 if count<1000{return None;}
 let n=count.div_ceil(1000);let mean=top[..n].iter().sum::<f32>()/n as f32;
 (mean>0.).then(||1000./mean)
}
#[cfg(test)]mod low01_tests{
 #[test]fn enough_frames_and_correct_worst_tail(){
  assert!(super::slowest_01(std::iter::repeat_n(10.,999)).is_none());
  let a=std::iter::repeat_n(10.,999).chain([50.]);assert_eq!(super::slowest_01(a),Some(20.));
  let b=std::iter::repeat_n(10.,1000).chain([20.,60.]);assert_eq!(super::slowest_01(b),Some(25.));
 }
}

#[cfg(test)]mod v90_pid_tests{use super::*;#[test]fn hud_snapshot_rejects_other_pid(){let mut s=PresentStats::default();s.tracked_pid=Some(123);s.last_frame=Some(Instant::now());for _ in 0..12{s.frames.push_back(Frame{ms:20.,gpu_ms:None,latency_ms:None,input_to_photon_ms:None,click_to_photon_ms:None,present_path:None});}assert!(s.snapshot_for(Some(123)).is_some());assert!(s.snapshot_for(Some(124)).is_none());assert!(s.snapshot_for(None).is_none());}}