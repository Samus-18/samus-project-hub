//! Transient playback-only FFT; no microphone, disk, network or raw sample export.
use std::{sync::{Arc,atomic::{AtomicBool,Ordering},mpsc::{self,Receiver}},time::{Duration,Instant},thread};
pub const N:usize=2048;
pub const EDGES:[f32;13]=[20.,60.,120.,250.,500.,1000.,2000.,3000.,4500.,6000.,8000.,12000.,20000.];
#[derive(Clone,Debug)]pub struct Reading{pub bands:[Option<f32>;12],pub peak:f32,pub rms:f32,pub dominant_hz:Option<f32>,pub rate:u32,pub at:Instant}
pub struct Analyzer{samples:[f32;N],real:[f32;N],imag:[f32;N],window:[f32;N],cursor:usize,count:usize}
impl Default for Analyzer{fn default()->Self{Self{samples:[0.;N],real:[0.;N],imag:[0.;N],window:std::array::from_fn(|i|0.5-0.5*(std::f32::consts::TAU*i as f32/(N-1) as f32).cos()),cursor:0,count:0}}}
impl Analyzer{
 pub fn push(&mut self,v:f32){self.samples[self.cursor]=if v.is_finite(){v.clamp(-1.,1.)}else{0.};self.cursor=(self.cursor+1)%N;self.count=(self.count+1).min(N);}
 pub fn silence(&mut self,n:usize){for _ in 0..n.min(N){self.push(0.);}}
 pub fn clear(&mut self){self.samples.fill(0.);self.count=0;self.cursor=0;}
 pub fn measure(&mut self,rate:u32,at:Instant)->Option<Reading>{
  if self.count<N||!(8000..=192000).contains(&rate){return None;}
  let(mut peak,mut sum)=(0f32,0f32);
  for i in 0..N{
   let v=self.samples[(self.cursor+i)%N];peak=peak.max(v.abs());sum+=v*v;
   let j=i.reverse_bits()>>(usize::BITS-N.ilog2());self.real[j]=v*self.window[i];self.imag[j]=0.;
  }
  let mut len=2;
  while len<=N{
   let angle=-std::f32::consts::TAU/len as f32;let(co,si)=(angle.cos(),angle.sin());
   for start in (0..N).step_by(len){let(mut wr,mut wi)=(1.,0.);
    for k in 0..len/2{let(a,b)=(start+k,start+k+len/2);let r=self.real[b]*wr-self.imag[b]*wi;let q=self.real[b]*wi+self.imag[b]*wr;
     self.real[b]=self.real[a]-r;self.imag[b]=self.imag[a]-q;self.real[a]+=r;self.imag[a]+=q;
     (wr,wi)=(wr*co-wi*si,wr*si+wi*co);
    }
   }len*=2;
  }
  let mut powers=[0f32;12];let mut counts=[0u32;12];let(mut best,mut best_bin)=(0f32,0usize);
  for k in 1..=N/2{let freq=k as f32*rate as f32/N as f32;let power=self.real[k].powi(2)+self.imag[k].powi(2);
   if power>best{best=power;best_bin=k;}
   if let Some(i)=(0..12).find(|&i|freq>=EDGES[i]&&freq<EDGES[i+1]){powers[i]=powers[i].max(power);counts[i]+=1;}
  }
  let bands=std::array::from_fn(|i|if counts[i]==0{None}else{let amp=(powers[i].sqrt()*4./N as f32).max(0.000001);Some((20.*amp.log10()).clamp(-96.,0.))});
  Some(Reading{bands,peak,rms:(sum/N as f32).sqrt(),dominant_hz:(peak>0.001&&best_bin>0).then_some(best_bin as f32*rate as f32/N as f32),rate,at})
 }
}
#[derive(Default)]pub struct Meter{worker:Option<thread::JoinHandle<()>>,flag:Option<Arc<AtomicBool>>,rx:Option<Receiver<Result<Reading,String>>>,value:Option<Result<Reading,String>>,attempted:bool}
impl Meter{
 pub fn stop(&mut self){if let Some(f)=&self.flag{f.store(false,Ordering::Release);}self.value=None;self.attempted=false;}
 pub fn tick(&mut self,on:bool){
  if !on{self.stop();}
  let accept=on&&self.flag.as_ref().is_some_and(|f|f.load(Ordering::Acquire));
  if let Some(rx)=&self.rx{for _ in 0..4{match rx.try_recv(){Ok(v) if accept=>self.value=Some(v),Ok(_)=>{},_=>break}}}
  if self.worker.as_ref().is_some_and(|w|w.is_finished()){let _=self.worker.take().map(|w|w.join());self.flag=None;self.rx=None;}
  if on&&!self.attempted&&self.worker.is_none(){
   self.attempted=true;let flag=Arc::new(AtomicBool::new(true));let f=flag.clone();let(tx,rx)=mpsc::sync_channel(1);
   match thread::Builder::new().name("Horizon playback FFT".into()).spawn(move||{let result=crate::audio_capture::monitor_spectrum(f,|v|{let _=tx.try_send(Ok(v));});if let Err(e)=result{let _=tx.try_send(Err(e));}}){
    Ok(w)=>{self.worker=Some(w);self.flag=Some(flag);self.rx=Some(rx);},Err(e)=>self.value=Some(Err(e.to_string()))
   }
  }
 }
 pub fn reading(&self)->Option<&Reading>{if !self.flag.as_ref().is_some_and(|f|f.load(Ordering::Acquire)){return None;}self.value.as_ref()?.as_ref().ok().filter(|r|Instant::now().checked_duration_since(r.at).is_some_and(|age|age<=Duration::from_millis(750)))}
 pub fn error(&self)->Option<&str>{self.value.as_ref()?.as_ref().err().map(String::as_str)}
}
impl Drop for Meter{fn drop(&mut self){self.stop();}}
pub fn proof()->Result<String,String>{
 let mut a=Analyzer::default();let rate=48000;for i in 0..N{a.push((std::f32::consts::TAU*1500.*i as f32/rate as f32).sin()*0.5);}
 let r=a.measure(rate,Instant::now()).ok_or("FFT no result")?;
 if (r.dominant_hz.unwrap_or(0.)-1500.).abs()>1.||r.bands[5].unwrap_or(-96.) < -7.||r.bands[5].unwrap_or(0.) > -5.||r.bands[8].unwrap_or(0.) > -45.|| (r.rms-0.35355).abs()>0.002{return Err("FFT frequency or magnitude failed".into());}
 a.clear();a.silence(N);let s=a.measure(rate,Instant::now()).ok_or("FFT silence unavailable")?;if s.dominant_hz.is_some()||s.peak!=0.||s.rms!=0.{return Err("FFT silence incorrect".into());}
 Ok("AUDIO_FFT=PASS N=2048 TONE_HZ=1500 RMS_OK=true SILENCE_OK=true SYNTHETIC_ONLY=true NO_ENDPOINT_CAPTURE=true".into())
}
#[cfg(test)]mod tests{use super::*;#[test]fn pure_fft_proof(){assert!(proof().is_ok());let mut a=Analyzer::default();assert!(a.measure(48000,Instant::now()).is_none());a.silence(N);assert!(a.measure(100,Instant::now()).is_none());}}

#[cfg(test)]mod cancel_tests{use super::*;#[test]fn cancelled_worker_cannot_republish_old_values(){let mut m=Meter::default();m.flag=Some(Arc::new(AtomicBool::new(false)));m.attempted=true;let(tx,rx)=mpsc::sync_channel(1);m.rx=Some(rx);let reading=Reading{bands:[None;12],peak:1.,rms:1.,dominant_hz:Some(100.),rate:48000,at:Instant::now()};tx.send(Ok(reading)).unwrap();m.tick(true);assert!(m.reading().is_none());assert!(m.value.is_none());}}


#[cfg(test)]mod v95_fft_age_tests{use super::*;#[test]fn future_and_stale_fft_readings_not_accepted(){let n=Instant::now();let mut m=Meter::default();m.flag=Some(Arc::new(AtomicBool::new(true)));let mk=|at|Reading{bands:[None;12],peak:0.1,rms:0.1,dominant_hz:Some(100.),rate:48000,at};m.value=Some(Ok(mk(n+Duration::from_secs(4))));assert!(m.reading().is_none());m.value=Some(Ok(mk(n-Duration::from_secs(3))));assert!(m.reading().is_none());m.value=Some(Ok(mk(n)));assert!(m.reading().is_some());}}
