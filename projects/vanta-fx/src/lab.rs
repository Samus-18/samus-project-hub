//! Performance lab: uses ONLY real frames delivered by PresentMon ETW.
//! No arbitrary FPS generation, GPU clock modification, anti-cheat hooks or game DLLs.
use crate::fps::Frame;
use serde::{Deserialize,Serialize};
use std::{fs,path::PathBuf,time::{Duration,Instant,SystemTime,UNIX_EPOCH}};
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct PerfReport{
    pub game:String,pub seconds:u32,pub frames:usize,
    #[serde(default)] pub pid:u32,
    #[serde(default)] pub run_id:u64,
    pub fps_avg:f32,pub fps_one_percent_low:f32,pub fps_point_one_percent_low:f32,
    pub frametime_avg_ms:f32,pub frametime_p95_ms:f32,pub frametime_p99_ms:f32,
    pub worst_frame_ms:f32,pub spikes:usize,
    pub gpu_mean_ms:Option<f32>,pub display_latency_mean_ms:Option<f32>,
    #[serde(default)] pub input_to_photon_mean_ms:Option<f32>,
    #[serde(default)] pub click_to_photon_mean_ms:Option<f32>
}
pub struct ActiveBenchmark{
    pub game:String,pub pid:u32,pub run_id:u64,
    pub length:Duration,pub started:Instant,pub frames:Vec<Frame>
}
#[derive(Default)]
pub struct PerformanceLab{
    pub active:Option<ActiveBenchmark>,
    pub latest:Option<PerfReport>,
    pub baseline:Option<PerfReport>,
    pub baseline_frames:Vec<Frame>,
    pub run_sequence:u64,
    pub last_frames:Vec<Frame>,
    pub last_error:Option<String>,
}
pub fn analyze(game:&str,seconds:u32,samples:&[Frame])->Result<PerfReport,String>{
    let mut clean=samples.iter().copied()
        .filter(|f|f.ms.is_finite()&&(0.2..=1000.0).contains(&f.ms))
        .collect::<Vec<_>>();
    if clean.len()<30{return Err(format!("Only {} valid ETW frames captured; at least 30 required",clean.len()));}
    let sum=clean.iter().map(|f|f.ms).sum::<f32>();
    let avg=sum/clean.len() as f32;
    clean.sort_by(|a,b|a.ms.total_cmp(&b.ms));
    let rank=|quantile:f32|->f32{
        let i=((clean.len()-1) as f32*quantile).ceil() as usize;
        clean[i.min(clean.len()-1)].ms
    };
    let worst=clean.last().unwrap().ms;
    let low=|fraction:f32|->f32 {
        let k=((clean.len() as f32*fraction).ceil() as usize).max(1);
        let avg_tail=clean.iter().rev().take(k).map(|f|f.ms).sum::<f32>()/k as f32;
        1000.0/avg_tail
    };
    let median=rank(0.5);
    let threshold=(median*2.0).max(33.0);
    let spikes=clean.iter().filter(|f|f.ms>threshold).count();
    let mean_optional=|get:fn(&Frame)->Option<f32>|->Option<f32>{
        let values=clean.iter().filter_map(get)
            .filter(|v|v.is_finite()&&*v>=0.0).collect::<Vec<_>>();
        if values.is_empty(){None}else{Some(values.iter().sum::<f32>()/values.len() as f32)}
    };
    Ok(PerfReport{
        game:game.into(),seconds,frames:clean.len(),pid:0,run_id:0,fps_avg:1000.0/avg,
        fps_one_percent_low:low(0.01),fps_point_one_percent_low:low(0.001),
        frametime_avg_ms:avg,frametime_p95_ms:rank(0.95),frametime_p99_ms:rank(0.99),
        worst_frame_ms:worst,spikes,gpu_mean_ms:mean_optional(|f|f.gpu_ms),
        display_latency_mean_ms:mean_optional(|f|f.latency_ms),
        input_to_photon_mean_ms:mean_optional(|f|f.input_to_photon_ms),
        click_to_photon_mean_ms:mean_optional(|f|f.click_to_photon_ms)
    })
}
#[derive(Clone,Debug,Serialize)]
pub struct AbComparison{
    pub before:PerfReport,
    pub after:PerfReport,
    pub fps_delta:f32,
    pub low_1_percent_delta:f32,
    pub p95_ms_delta:f32,
    pub p99_ms_delta:f32,
    pub gpu_ms_delta:Option<f32>,
    pub latency_ms_delta:Option<f32>,
    pub input_to_photon_ms_delta:Option<f32>,
    pub click_to_photon_ms_delta:Option<f32>,
}
impl PerformanceLab {
    pub fn start(&mut self,game:&str,pid:u32,seconds:u32)->Result<(),String>{
        if self.active.is_some(){return Err("A benchmark is already running".into());}
        if pid==0{return Err("Benchmark requires a selected real game PID".into());}
        if !(10..=120).contains(&seconds){return Err("Benchmark must be 10–120 seconds".into());}
        self.run_sequence=self.run_sequence.saturating_add(1);
        self.active=Some(ActiveBenchmark{game:game.into(),pid,run_id:self.run_sequence,
            length:Duration::from_secs(seconds as u64),
            started:Instant::now(),frames:Vec::with_capacity(seconds as usize*90)});
        self.last_error=None;
        Ok(())
    }
    pub fn cancel(&mut self){self.active=None;}
    pub fn remaining(&self)->Option<u64>{
        self.active.as_ref().map(|s|s.length.saturating_sub(s.started.elapsed()).as_secs())
    }
    pub fn ingest(&mut self,new_frames:&[Frame])->Option<Result<PerfReport,String>>{
        let Some(current)=self.active.as_mut() else{return None;};
        if current.started.elapsed()<current.length{
            current.frames.extend_from_slice(new_frames);
            return None;
        }
        let mut current=self.active.take().unwrap();
        current.frames.extend_from_slice(new_frames);
        let report=analyze(&current.game,current.length.as_secs() as u32,&current.frames)
            .map(|mut report|{
                report.pid=current.pid;
                report.run_id=current.run_id;
                report
            });
        match &report{
            Ok(r)=>{
                self.last_frames=current.frames;
                self.latest=Some(r.clone());
                self.last_error=None;
            }
            Err(e)=>self.last_error=Some(e.clone())
        }
        Some(report)
    }
    /// Baseline uses a finished actual ETW capture, never a simulated sample.
    pub fn set_baseline(&mut self)->Result<(),String>{
        if self.active.is_some(){return Err("Finish or cancel recording first".into());}
        let report=self.latest.as_ref().ok_or("No finished real ETW report")?;
        if report.pid==0||report.run_id==0||report.frames<180||self.last_frames.len()<180{
            return Err("A/B baseline requires 180 real ETW frames plus a valid game PID".into());
        }
        self.baseline=Some(report.clone());
        self.baseline_frames=self.last_frames.clone();
        Ok(())
    }
    pub fn comparison(&self)->Result<AbComparison,String>{
        let a=self.baseline.as_ref().ok_or("Capture baseline A first")?;
        let b=self.latest.as_ref().ok_or("Capture comparison B first")?;
        if a.run_id==0||b.run_id<=a.run_id{
            return Err("Record a NEW run B after setting baseline A".into());
        }
        if a.pid==0||a.pid!=b.pid||a.game!=b.game{
            return Err("A/B invalid: selected game PID or title differs".into());
        }
        if a.seconds!=b.seconds{
            return Err("A/B invalid: benchmark durations differ".into());
        }
        if a.frames<180||b.frames<180{
            return Err("A/B requires at least 180 genuine ETW frames in each run".into());
        }
        Ok(AbComparison{
            before:a.clone(),after:b.clone(),
            fps_delta:b.fps_avg-a.fps_avg,
            low_1_percent_delta:b.fps_one_percent_low-a.fps_one_percent_low,
            p95_ms_delta:b.frametime_p95_ms-a.frametime_p95_ms,
            p99_ms_delta:b.frametime_p99_ms-a.frametime_p99_ms,
            gpu_ms_delta:b.gpu_mean_ms.zip(a.gpu_mean_ms).map(|(new,old)|new-old),
            latency_ms_delta:b.display_latency_mean_ms.zip(a.display_latency_mean_ms)
                .map(|(new,old)|new-old),
            input_to_photon_ms_delta:b.input_to_photon_mean_ms
                .zip(a.input_to_photon_mean_ms).map(|(new,old)|new-old),
            click_to_photon_ms_delta:b.click_to_photon_mean_ms
                .zip(a.click_to_photon_mean_ms).map(|(new,old)|new-old)
        })
    }
    /// Export unmodified raw A and B sample rows plus source/limitations metadata.
    pub fn export_ab(&self)->Result<PathBuf,String>{
        let comparison=self.comparison()?;
        if self.baseline_frames.is_empty()||self.last_frames.is_empty(){
            return Err("Original baseline or comparison ETW samples not available".into());
        }
        let dir=crate::settings::folder().join("reports");
        fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
        let stamp=SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|e|e.to_string())?.as_millis();
        let prefix=format!("etw-ab-{stamp}");
        for (tag,frames) in [("A",&self.baseline_frames),("B",&self.last_frames)]{
            let mut csv=String::from("frame,frame_ms,gpu_ms,display_latency_ms,input_to_photon_ms,click_to_photon_ms\n");
            for (i,frame) in frames.iter().enumerate(){
                let gpu=frame.gpu_ms.map(|x|x.to_string()).unwrap_or_default();
                let latency=frame.latency_ms.map(|x|x.to_string()).unwrap_or_default();
                csv.push_str(&format!("{},{},{},{},{},{}\n",i+1,frame.ms,gpu,latency,
                    frame.input_to_photon_ms.map(|x|x.to_string()).unwrap_or_default(),
                    frame.click_to_photon_ms.map(|x|x.to_string()).unwrap_or_default()));
            }
            fs::write(dir.join(format!("{prefix}-{tag}.csv")),csv)
                .map_err(|e|e.to_string())?;
        }
        let summary=dir.join(format!("{prefix}-summary.json"));
        let document=serde_json::json!({
            "source":"Genuine PresentMon ETW game frame presentations, manual A/B",
            "result":comparison,
            "interpretation":"Observational only. Match scenes, graphics settings, cap and thermals. Difference does NOT prove Horizon Boost caused change.",
            "csv_a":format!("{prefix}-A.csv"),
            "csv_b":format!("{prefix}-B.csv"),
            "generated_frames_excluded":true
        });
        fs::write(&summary,serde_json::to_vec_pretty(&document)
            .map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        Ok(summary)
    }
    pub fn export(&self)->Result<PathBuf,String>{
        let summary=self.latest.as_ref().ok_or("No completed benchmark to export")?;
        if self.last_frames.is_empty(){return Err("Benchmark raw frame samples unavailable".into());}
        let path=crate::settings::folder().join("reports");
        fs::create_dir_all(&path).map_err(|e|e.to_string())?;
        let timestamp=SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|e|e.to_string())?.as_secs();
        let output=path.join(format!("benchmark-{timestamp}.csv"));
        let mut csv=String::from("frame,frame_ms,gpu_ms,display_latency_ms,input_to_photon_ms,click_to_photon_ms\n");
        for (index,frame) in self.last_frames.iter().enumerate(){
            let gpu=frame.gpu_ms.map(|n|n.to_string()).unwrap_or_default();
            let lat=frame.latency_ms.map(|n|n.to_string()).unwrap_or_default();
            csv.push_str(&format!("{},{},{},{},{},{}\n",index+1,frame.ms,gpu,lat,
                frame.input_to_photon_ms.map(|x|x.to_string()).unwrap_or_default(),
                frame.click_to_photon_ms.map(|x|x.to_string()).unwrap_or_default()));
        }
        fs::write(&output,csv).map_err(|e|e.to_string())?;
        let meta=path.join(format!("benchmark-{timestamp}.json"));
        fs::write(&meta,serde_json::to_vec_pretty(summary).map_err(|e|e.to_string())?)
            .map_err(|e|e.to_string())?;
        Ok(output)
    }
}
pub fn recommendation(report:&PerfReport,target:u32)->&'static str{
    let budget=1000.0/target.max(1) as f32;
    if report.gpu_mean_ms.is_some_and(|g|g>budget*0.9){
        "GPU limit likely: try FPS Priority, reduced render resolution or fewer GPU postprocessing passes."
    }else if report.frametime_p99_ms>budget*2.0{
        "Severe frame pacing spikes: investigate shader compilation, streaming and background programs."
    }else if report.frametime_avg_ms>budget{
        "Target is above measured average: lower game settings, or consider a lower FPS target."
    }else{
        "Target appears plausible in this sample. Re-test in a demanding scene before drawing conclusions."
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    fn example()->Vec<Frame>{
        (0..1000).map(|i|Frame{ms:if i%100==0{50.0}else{16.66},
            gpu_ms:Some(9.0),latency_ms:Some(23.0),
            input_to_photon_ms:Some(34.0),click_to_photon_ms:None,
            present_path:None}).collect()
    }
    #[test]fn measures_real_frame_distribution(){
        let r=analyze("game",30,&example()).unwrap();
        assert_eq!(r.frames,1000);
        assert_eq!(r.spikes,10);
        assert!(r.frametime_p99_ms>=16.66);
        assert!((r.fps_one_percent_low-20.0).abs()<0.1);
        assert_eq!(r.gpu_mean_ms,Some(9.0));
        assert_eq!(r.input_to_photon_mean_ms,Some(34.0));
        assert_eq!(r.click_to_photon_mean_ms,None);
        assert!(r.fps_avg>55.0&&r.fps_avg<60.0);
    }
    #[test]fn rejects_missing_or_bogus_frames(){
        assert!(analyze("none",30,&[]).is_err());
        let fake=vec![Frame{ms:f32::NAN,gpu_ms:None,latency_ms:None,
            input_to_photon_ms:None,click_to_photon_ms:None,present_path:None};50];
        assert!(analyze("broken",30,&fake).is_err());
    }
    #[test]fn advisor_uses_measured_values(){
        let mut r=analyze("game",30,&example()).unwrap();
        r.gpu_mean_ms=Some(19.0);
        assert!(recommendation(&r,60).contains("GPU limit"));
        r.gpu_mean_ms=Some(8.0);
        assert!(recommendation(&r,60).contains("spikes"));
    }
    #[test]fn paired_etw_never_compares_same_run_or_different_process(){
        let mut lab=PerformanceLab::default();
        assert!(lab.set_baseline().is_err());
        let mut before=analyze("game",30,&example()).unwrap();
        before.pid=445;before.run_id=1;
        lab.latest=Some(before.clone());lab.last_frames=example();
        lab.set_baseline().unwrap();
        assert!(lab.comparison().is_err());
        let mut after=before.clone();
        after.run_id=2;after.fps_avg+=5.0;after.frametime_p95_ms-=1.0;
        lab.latest=Some(after.clone());
        let diff=lab.comparison().unwrap();
        assert!((diff.fps_delta-5.0).abs()<0.001);
        assert!((diff.p95_ms_delta+1.0).abs()<0.001);
        after.pid=446;lab.latest=Some(after.clone());
        assert!(lab.comparison().unwrap_err().contains("PID"));
        after.pid=445;after.seconds=60;lab.latest=Some(after.clone());
        assert!(lab.comparison().unwrap_err().contains("durations"));
        after.seconds=30;after.frames=150;lab.latest=Some(after);
        assert!(lab.comparison().unwrap_err().contains("180"));
    }
    #[test]fn baseline_requires_180_actual_frames(){
        let mut lab=PerformanceLab::default();
        let mut report=analyze("game",30,&example()).unwrap();
        report.pid=445;report.run_id=1;
        lab.latest=Some(report);
        lab.last_frames=example()[..60].to_vec();
        assert!(lab.set_baseline().is_err());
        lab.last_frames=example();
        assert!(lab.set_baseline().is_ok());
    }
    #[test]fn benchmark_validation_and_cancel(){
        let mut lab=PerformanceLab::default();
        assert!(lab.start("game",140,1).is_err());
        assert!(lab.start("game",0,30).is_err());
        assert!(lab.start("game",140,30).is_ok());
        assert!(lab.start("other",141,30).is_err());
        lab.cancel();
        assert!(lab.active.is_none());
    }
}
