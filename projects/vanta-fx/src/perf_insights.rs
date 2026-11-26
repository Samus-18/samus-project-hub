//! Real PresentMon ETW-based frame stability and cap advisor.
//! Read-only: does not fabricate game FPS or enforce an FPS cap.
use serde::Serialize;
use std::collections::VecDeque;
#[derive(Clone,Debug,Serialize,PartialEq)]
pub struct FrameHealth{
    pub count:usize,
    pub mean_ms:f32,
    pub median_ms:f32,
    pub p95_ms:f32,
    pub p99_ms:f32,
    pub max_ms:f32,
    pub jitter_ms:f32,
    pub spike_count:usize,
    pub misses_60:usize,
    pub misses_120:usize,
    pub bands:[usize;5],
    pub suggested_cap:Option<u32>,
    pub sample_duration_ms:f64,
    pub longest_spike_run:usize,
}
pub fn latency_points(frames:&VecDeque<crate::fps::Frame>)->Vec<Option<f32>>{let count=frames.len().min(120);frames.iter().skip(frames.len()-count).map(|f|f.latency_ms.filter(|v|v.is_finite()&&(0.0..=1000.).contains(v))).collect()}
impl FrameHealth{
    pub fn from_frames(frames:&VecDeque<crate::fps::Frame>)->Option<Self>{
        if frames.len()<45{return None;}
        let count=frames.len().min(600);let times=frames.iter().skip(frames.len()-count).map(|x|x.ms).collect::<Vec<_>>();
        Self::from_ms(&times)
    }
    pub fn from_ms(raw:&[f32])->Option<Self>{
        let samples=raw[raw.len().saturating_sub(600)..].iter().copied().filter(|x|x.is_finite()&&(0.2..=1000.0).contains(x)).collect::<Vec<_>>();
        let n=samples.len();
        if n<45{return None;}
        let mut ordered=samples.clone();ordered.sort_by(f32::total_cmp);
        let percentile=|p:usize|->f32{ordered[(n*p).div_ceil(100).saturating_sub(1).min(n-1)]};
        let median=(ordered[(n-1)/2]+ordered[n/2])*0.5;
        let p95=percentile(95);
        let p99=percentile(99);
        let mean=samples.iter().sum::<f32>()/n as f32;
        let jitter=samples.windows(2)
            .map(|pair|(pair[1]-pair[0]).abs()).sum::<f32>()/(n-1) as f32;
        let threshold=33.33_f32.max(median*1.8);
        let spikes=samples.iter().filter(|x|**x>threshold).count();
        let mut run=0;let mut longest_spike_run=0;for &ms in &samples{if ms>threshold{run+=1;longest_spike_run=longest_spike_run.max(run);}else{run=0;}}
        let mut bands=[0usize;5];
        for &time in &samples{
            let bucket=if time<=8.33{0}else if time<=16.67{1}
                else if time<=33.33{2}else if time<=50.0{3}else{4};
            bands[bucket]+=1;
        }
        // Recommendation is explicitly advisory, never a clock/driver change.
        let cap=if n>=180{
            let tail_fps=1000.0/p95;
            [30,40,45,50,60,75,90,100,120,144,165,240]
                .into_iter().filter(|x|(*x as f32)<=tail_fps+1.6)
                .last()
        }else{None};
        Some(Self{
            count:n,mean_ms:mean,median_ms:median,p95_ms:p95,p99_ms:p99,
            max_ms:*ordered.last().unwrap(),jitter_ms:jitter,spike_count:spikes,
            misses_60:samples.iter().filter(|x|**x>16.67).count(),
            misses_120:samples.iter().filter(|x|**x>8.33).count(),
            bands,suggested_cap:cap,sample_duration_ms:samples.iter().map(|&v|v as f64).sum(),longest_spike_run,
        })
    }
    pub fn target_misses(frames:&VecDeque<crate::fps::Frame>,target:u32)->Option<(usize,usize)>{
        if !(20..=360).contains(&target){return None;}let budget=1000.0/target as f32;let mut count=0;let mut misses=0;
        for f in frames.iter().rev().take(600){if f.ms.is_finite()&&(0.2..=1000.).contains(&f.ms){count+=1;if f.ms>budget{misses+=1;}}}
        (count>=45).then_some((misses,count))
    }
    pub fn export(&self,game:&str)->Result<std::path::PathBuf,String>{
        let dir=crate::settings::folder().join("diagnostics");
        std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
        let stamp=std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).map_err(|e|e.to_string())?
            .as_secs();
        let dest=dir.join(format!("frametime-{stamp}-{}.json",crate::media_tools::stamp()));
        let data=serde_json::json!({
            "source":"PresentMon ETW real captured frames",
            "game":game,
            "data":self,
            "no_automatic_fps_cap":true,
            "capture_based_framegen_is_not_included":true,
            "sample_window_max_frames":600
        });
        crate::settings::atomic_json_replace(&dest,&serde_json::to_vec_pretty(&data).map_err(|e|e.to_string())?)?;
        Ok(dest)
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn insufficient_samples_produce_no_stats(){
        assert!(FrameHealth::from_ms(&vec![16.6;20]).is_none());
    }
    #[test]fn exactly_60fps_has_stable_advisory(){
        let a=FrameHealth::from_ms(&vec![16.666;600]).unwrap();
        assert_eq!(a.count,600);
        assert_eq!(a.spike_count,0);
        assert_eq!(a.suggested_cap,Some(60));
        assert!(a.jitter_ms<0.001);
        assert_eq!(a.bands,[0,600,0,0,0]);
    }
    #[test]fn heavy_frame_spikes_are_measured_not_invented(){
        let mut input=vec![16.6;295];
        input.extend([50.,80.,90.,100.,120.]);
        let a=FrameHealth::from_ms(&input).unwrap();
        assert_eq!(a.spike_count,5);
        assert!(a.p99_ms>16.6);
        assert!(a.max_ms>=120.);
        assert_eq!(a.bands.iter().sum::<usize>(),300);
    }
    #[test]fn invalid_values_are_ignored(){
        let mut input=vec![16.6;100];
        input.extend([f32::NAN,-2.0,0.0,f32::INFINITY,50000.0]);
        assert_eq!(FrameHealth::from_ms(&input).unwrap().count,100);
    }
}

#[cfg(test)]mod quantile_tests{use super::*;#[test]fn even_median_and_nearest_rank(){let v=(1..=100).map(|v|v as f32).collect::<Vec<_>>();let s=FrameHealth::from_ms(&v).unwrap();assert_eq!(s.median_ms,50.5);assert_eq!(s.p95_ms,95.);assert_eq!(s.p99_ms,99.);}}

#[cfg(test)]mod latest_tests{use super::*;#[test]fn latest_window_not_old_history(){let mut a=vec![100.;600];a.extend(vec![10.;600]);let h=FrameHealth::from_ms(&a).unwrap();assert_eq!(h.mean_ms,10.);assert_eq!(h.spike_count,0);}}

#[cfg(test)]mod target_tests{use super::*;
 #[test]fn budget_counts_only_valid_recent_frames(){let mut f=VecDeque::new();for ms in [10.;50].into_iter().chain([25.;10]).chain([f32::NAN]){f.push_back(crate::fps::Frame{ms,..Default::default()});}assert_eq!(FrameHealth::target_misses(&f,60),Some((10,60)));assert_eq!(FrameHealth::target_misses(&f,0),None);}
 #[test]fn spike_streak_counts_consecutive_intervals(){let mut v=vec![10.;100];v.extend([70.,80.,10.,60.]);let h=FrameHealth::from_ms(&v).unwrap();assert_eq!(h.longest_spike_run,2);assert_eq!(h.spike_count,3);assert_eq!(h.sample_duration_ms,1220.);}
}

#[cfg(test)]mod v88_latency_tests{use super::*;#[test]fn latency_gaps_are_not_filled_or_fps_derived(){let f=[Some(0.),None,Some(f32::NAN),Some(30.)].into_iter().map(|latency_ms|crate::fps::Frame{ms:16.,latency_ms,..Default::default()}).collect::<VecDeque<_>>();assert_eq!(latency_points(&f),vec![Some(0.),None,None,Some(30.)]);}}

#[derive(Debug,PartialEq)]
pub struct GpuCoverage{pub paired:usize,pub valid:usize,pub near_budget:usize,pub median_ratio:f32}
pub fn gpu_coverage(frames:&VecDeque<crate::fps::Frame>)->Option<GpuCoverage>{
 let mut valid=0;let mut ratios=Vec::new();
 for f in frames.iter().skip(frames.len().saturating_sub(600)){
  if !f.ms.is_finite()||f.ms<0.2||f.ms>1000.{continue;}valid+=1;
  if let Some(g)=f.gpu_ms.filter(|v|v.is_finite()&&*v>=0.&&*v<=1000.&&*v<=f.ms*1.35){ratios.push(g/f.ms);}
 }
 if valid<45||ratios.len()*10<valid*7{return None;}
 ratios.sort_by(f32::total_cmp);let n=ratios.len();
 Some(GpuCoverage{paired:n,valid,near_budget:ratios.iter().filter(|v|**v>=0.85).count(),median_ratio:(ratios[(n-1)/2]+ratios[n/2])*0.5})
}
#[cfg(test)]mod v88_coverage_tests{use super::*;
 #[test]fn paired_only_coverage_and_headroom(){let mut q=VecDeque::new();for _ in 0..60{q.push_back(crate::fps::Frame{ms:20.,gpu_ms:Some(18.),latency_ms:None,input_to_photon_ms:None,click_to_photon_ms:None,present_path:None});}
 let s=gpu_coverage(&q).unwrap();assert_eq!(s.near_budget,60);assert_eq!(s.paired,60);assert!((s.median_ratio-0.9).abs()<0.001);
 for f in q.iter_mut().take(20){f.gpu_ms=None;}assert!(gpu_coverage(&q).is_none());}
}

#[cfg(test)]mod v95_paired_gpu_tests{use super::*;#[test]fn unreasonable_gpu_time_does_not_establish_bottleneck(){let mut q=VecDeque::new();for _ in 0..60{q.push_back(crate::fps::Frame{ms:16.,gpu_ms:Some(300.),..Default::default()});}assert!(gpu_coverage(&q).is_none());for f in &mut q{f.gpu_ms=Some(14.);}assert!(gpu_coverage(&q).is_some());}}
