use serde::Serialize;
use std::{collections::VecDeque,time::{Instant,Duration}};
#[derive(Clone,Debug,Serialize)]pub struct Sample{pub elapsed_ms:u64,pub fps:f32,pub frame_ms:f32,pub low_1:Option<f32>,pub gpu_ms:Option<f32>,pub latency_ms:Option<f32>,pub output_real:Option<u64>,pub output_generated:Option<u64>}
#[derive(Clone,Debug,Serialize)]pub struct Spike{pub observed_ms:u64,pub frame_ms:f32}
#[derive(Clone,Debug,Serialize)]pub struct Marker{pub elapsed_ms:u64,pub label:String}
#[derive(Default)]pub struct Recorder{pid:Option<u32>,started:Option<Instant>,stopped:Option<Instant>,last:Option<Instant>,pub rows:VecDeque<Sample>,pub markers:Vec<Marker>,pub spikes:VecDeque<Spike>,raw:VecDeque<f32>,raw_total:u64}
impl Recorder{
 pub fn active(&self)->bool{self.pid.is_some()}
 pub fn start(&mut self,pid:u32,now:Instant){*self=Self{pid:Some(pid),started:Some(now),..Default::default()};}
 pub fn stop(&mut self){self.stop_at(Instant::now());}
 fn stop_at(&mut self,now:Instant){if self.active(){self.stopped=Some(now);self.pid=None;}}
 pub fn elapsed(&self,now:Instant)->u64{self.started.map_or(0,|t|self.stopped.unwrap_or(now).saturating_duration_since(t).as_millis().min(u64::MAX as u128) as u64)}
 pub fn update(&mut self,pid:Option<u32>,now:Instant,stats:Option<crate::fps::Stats>,output:Option<(u64,u64)>){
  if self.pid.is_none(){return;}if self.pid!=pid{self.stop_at(now);return;}
  if self.last.is_some_and(|t|now<t||now.saturating_duration_since(t)<Duration::from_secs(1)){return;}
  self.last=Some(now);
  if let Some(s)=stats.filter(|s|s.fps.is_finite()&&s.fps>0.&&s.frametime_ms.is_finite()&&s.frametime_ms>0.){
   let valid=|v:Option<f32>|v.filter(|x|x.is_finite()&&*x>=0.);
   if self.rows.len()>=3600{self.rows.pop_front();}
   self.rows.push_back(Sample{elapsed_ms:self.elapsed(now),fps:s.fps,frame_ms:s.frametime_ms,low_1:valid(s.low_1_percent_fps),
    gpu_ms:valid(s.gpu_ms),latency_ms:valid(s.latency_ms),output_real:output.map(|x|x.0),output_generated:output.map(|x|x.1)});
  }
 }
 pub fn capture_spikes(&mut self,pid:Option<u32>,frames:&[crate::fps::Frame],threshold:f32,now:Instant){
  if self.pid.is_none()||self.pid!=pid{return;}
  let threshold=if threshold.is_finite(){threshold.clamp(20.,200.)}else{45.};
  let elapsed=self.elapsed(now);
  for f in frames{
   if f.ms.is_finite()&&(0.2..=1000.).contains(&f.ms){
    self.raw_total=self.raw_total.saturating_add(1);if self.raw.len()>=32768{self.raw.pop_front();}self.raw.push_back(f.ms);
   }
   if f.ms.is_finite()&&(threshold..=1000.).contains(&f.ms){
    if self.spikes.len()>=512{self.spikes.pop_front();}
    self.spikes.push_back(Spike{observed_ms:elapsed,frame_ms:f.ms});
   }
  }
 }
 pub fn mark(&mut self,label:&str,now:Instant){if self.active()&&self.markers.len()<256{
  self.markers.push(Marker{elapsed_ms:self.elapsed(now),label:label.chars().filter(|c|!c.is_control()).take(48).collect()});
 }}
 pub fn json(&self)->Result<String,String>{serde_json::to_string_pretty(&serde_json::json!({"format":"HorizonBoost.Session.1","duration_ms":self.elapsed(Instant::now()),"samples":self.rows,"markers":self.markers,"stutters":self.spikes,"raw_retained_summary":self.raw_summary(),"source":"PresentMon ETW; 1s sampled averages; own output counters are not game FPS"})).map_err(|e|e.to_string())}
 pub fn csv(&self)->String{
  let mut out="elapsed_ms,fps,frame_ms,low_1,gpu_ms,latency_ms,own_real_count,own_generated_count\n".to_string();
  let f=|v:Option<f32>|v.map(|x|format!("{x:.3}")).unwrap_or_default();
  let n=|v:Option<u64>|v.map(|x|x.to_string()).unwrap_or_default();
  for s in &self.rows{out.push_str(&format!("{},{:.3},{:.3},{},{},{},{},{}\n",s.elapsed_ms,s.fps,s.frame_ms,f(s.low_1),f(s.gpu_ms),f(s.latency_ms),n(s.output_real),n(s.output_generated)));}
  out
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn no_fake_samples_pid_switch_and_bounded_markers(){
  let mut r=Recorder::default();let n=Instant::now();r.start(5,n);r.update(Some(5),n,None,None);assert!(r.rows.is_empty());
  for _ in 0..300{r.mark("manual",n);}assert_eq!(r.markers.len(),256);
  r.update(Some(6),n,None,None);assert!(!r.active());assert!(r.json().unwrap().contains("PresentMon ETW"));
 }
 #[test]fn real_shape_export_no_path_or_pid(){
  let mut r=Recorder::default();r.rows.push_back(Sample{elapsed_ms:10,fps:60.,frame_ms:16.67,low_1:None,gpu_ms:None,latency_ms:None,output_real:Some(2),output_generated:Some(1)});
  assert!(r.csv().contains("10,60.000,16.670,,,,2,1"));
  assert!(!r.json().unwrap().contains("game_key"));
 }
}

#[derive(Clone,Debug,Serialize,PartialEq)]pub struct Summary{pub duration_ms:u64,pub samples:usize,pub mean_fps:f32,pub median_ms:f32,pub p95_sample_ms:f32,pub max_sample_ms:f32,pub markers:usize,pub retained_spikes:usize}
impl Recorder{
 pub fn summary(&self,now:Instant)->Option<Summary>{
  if self.rows.len()<5{return None;}
  let mut values=self.rows.iter().map(|r|r.frame_ms).collect::<Vec<_>>();values.sort_by(f32::total_cmp);
  let n=values.len();let percentile=|p:usize|values[(n*p).div_ceil(100).saturating_sub(1).min(n-1)];
  Some(Summary{duration_ms:self.elapsed(now),samples:n,mean_fps:self.rows.iter().map(|r|r.fps).sum::<f32>()/n as f32,median_ms:(values[(n-1)/2]+values[n/2])*0.5,p95_sample_ms:percentile(95),max_sample_ms:values[n-1],markers:self.markers.len(),retained_spikes:self.spikes.len()})
 }
}
#[cfg(test)]mod compare_tests{use super::*;#[test]fn frozen_duration_and_sample_percentile(){let n=Instant::now();let mut r=Recorder::default();r.start(3,n);assert!(r.summary(n).is_none());for i in 0..5{r.rows.push_back(Sample{elapsed_ms:i*1000,fps:60.,frame_ms:10.+i as f32,low_1:None,gpu_ms:None,latency_ms:None,output_real:None,output_generated:None});}r.stop_at(n+Duration::from_secs(5));let s=r.summary(n+Duration::from_secs(90)).unwrap();assert_eq!(s.duration_ms,5000);assert_eq!(s.p95_sample_ms,14.);assert_eq!(s.median_ms,12.);}}

#[derive(Clone,Debug,Serialize)]pub struct RawSummary{pub retained:usize,pub total_valid:u64,pub mean_fps:f64,pub median_ms:f32,pub p95_ms:f32,pub p99_ms:f32,pub low_1:Option<f64>,pub low_01:Option<f64>,pub adjacent_jitter_ms:f64}
impl Recorder{
 pub fn raw_count(&self)->usize{self.raw.len()}
 pub fn raw_total(&self)->u64{self.raw_total}
 pub fn raw_summary(&self)->Option<RawSummary>{
  let n=self.raw.len();if n<45{return None;}
  let mut v=self.raw.iter().copied().collect::<Vec<_>>();v.sort_by(f32::total_cmp);
  let mean=v.iter().map(|x|f64::from(*x)).sum::<f64>()/n as f64;
  let pct=|p:usize|v[(n*p).div_ceil(100).saturating_sub(1).min(n-1)];
  let low=|div:usize|if n<div{None}else{let k=n.div_ceil(div);let tail=v[n-k..].iter().map(|v|f64::from(*v)).sum::<f64>()/k as f64;Some(1000./tail)};
  let jitter=self.raw.iter().zip(self.raw.iter().skip(1)).map(|(a,b)|f64::from((a-b).abs())).sum::<f64>()/(n-1)as f64;
  Some(RawSummary{retained:n,total_valid:self.raw_total,mean_fps:1000./mean,median_ms:(v[(n-1)/2]+v[n/2])*0.5,p95_ms:pct(95),p99_ms:pct(99),low_1:low(100),low_01:low(1000),adjacent_jitter_ms:jitter})
 }
}
#[cfg(test)]mod raw_tests{use super::*;#[test]fn retained_raw_distribution_and_exact_low_requirements(){let mut r=Recorder::default();assert!(r.raw_summary().is_none());for i in 0..1000{r.raw.push_back(if i==999{100.}else{10.});}r.raw_total=1000;let s=r.raw_summary().unwrap();assert_eq!(s.p95_ms,10.);assert_eq!(s.low_01,Some(10.));assert_eq!(s.retained,1000);assert!(s.low_1.unwrap()<100.);assert_eq!(s.adjacent_jitter_ms,90./999.);assert!(r.json().unwrap().contains("raw_retained_summary"));}}

#[cfg(test)]mod exact_distribution_tests{use super::*;#[test]fn even_median_and_invalid_spikes(){let now=Instant::now();let mut r=Recorder::default();r.start(7,now);let mut frames=vec![crate::fps::Frame{ms:10.,..Default::default()};50];frames.extend(vec![crate::fps::Frame{ms:20.,..Default::default()};50]);frames.push(crate::fps::Frame{ms:5000.,..Default::default()});frames.push(crate::fps::Frame{ms:60.,..Default::default()});r.capture_spikes(Some(7),&frames,f32::NAN,now);assert_eq!(r.spikes.len(),1);assert_eq!(r.raw_count(),101);r.raw.pop_back();assert_eq!(r.raw_summary().unwrap().median_ms,15.);}}


#[cfg(test)]mod v95_clock_tests{use super::*;#[test]fn regressing_timestamp_does_not_leak_new_samples(){let n=Instant::now();let mut r=Recorder::default();r.start(100,n);r.update(Some(100),n,None,None);r.update(Some(100),n-Duration::from_secs(1),None,None);assert_eq!(r.rows.len(),0);}}
