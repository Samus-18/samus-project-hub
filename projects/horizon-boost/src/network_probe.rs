//! Explicitly opt-in Windows IPv4 ICMP monitor. Does not infer game RTT.
use std::{collections::VecDeque,net::Ipv4Addr,ffi::c_void,
    sync::{Arc,atomic::{AtomicBool,Ordering},mpsc::{self,Receiver}},
    thread,time::{Duration,Instant}};
type Handle=*mut c_void;
#[repr(C)]
struct IpOptions{ttl:u8,tos:u8,flags:u8,size:u8,data:*mut u8}
#[repr(C)]
struct EchoReply{address:u32,status:u32,rtt:u32,data_size:u16,reserved:u16,
    data:*mut c_void,options:IpOptions}
#[link(name="iphlpapi")]
unsafe extern "system"{
    fn IcmpCreateFile()->Handle;
    fn IcmpCloseHandle(handle:Handle)->i32;
    fn IcmpSendEcho(handle:Handle,address:u32,data:*const c_void,size:u16,
        options:*const IpOptions,reply:*mut c_void,reply_size:u32,timeout:u32)->u32;
}
#[derive(Clone,Copy,Debug)]
pub struct Sample{pub rtt:Option<u32>}
pub fn echo(ip:Ipv4Addr)->Result<Sample,String>{
    if ip.is_unspecified()||ip.is_multicast()||ip.is_broadcast(){
        return Err("Choose one unicast IPv4 address".into());
    }
    let handle=unsafe{IcmpCreateFile()};
    if handle.is_null()||handle as isize == -1{
        return Err(format!("IcmpCreateFile: {}",std::io::Error::last_os_error()));
    }
    let payload=[0u8;16];
    let mut reply=[0u64;64]; // aligned and ample for ICMP_ECHO_REPLY
    let count=unsafe{IcmpSendEcho(handle,u32::from_ne_bytes(ip.octets()),
        payload.as_ptr().cast(),16,std::ptr::null(),reply.as_mut_ptr().cast(),
        std::mem::size_of_val(&reply) as u32,750)};
    unsafe{IcmpCloseHandle(handle);}
    if count==0{return Ok(Sample{rtt:None});}
    let result=unsafe{&*(reply.as_ptr().cast::<EchoReply>())};
    Ok(Sample{rtt:if result.status==0{Some(result.rtt)}else{None}})
}
pub struct Probe{
    worker:Option<thread::JoinHandle<()>>,
    cancel:Arc<AtomicBool>,
    rx:Option<Receiver<(u64,Instant,Result<Sample,String>)>>,
    last_sequence:Option<u64>,
    pub discarded:u64,
    samples:VecDeque<Sample>,
    pub target:Option<Ipv4Addr>,
    pub error:Option<String>,
    last_reply:Option<Instant>,
}
impl Default for Probe{
    fn default()->Self{Self{worker:None,cancel:Arc::new(AtomicBool::new(false)),
        rx:None,samples:VecDeque::new(),target:None,error:None,last_reply:None,last_sequence:None,discarded:0}}
}
impl Probe{
    pub fn active(&self)->bool{self.cancel.load(Ordering::Acquire)&&self.worker.as_ref().is_some_and(|t|!t.is_finished())}
    pub fn stopping(&self)->bool{!self.cancel.load(Ordering::Acquire)&&self.worker.as_ref().is_some_and(|t|!t.is_finished())}
    pub fn start(&mut self,address:&str)->Result<(),String>{
        let ip:Ipv4Addr=address.trim().parse().map_err(|_|"Enter a valid IPv4 address")?;
        if ip.is_unspecified()||ip.is_multicast()||ip.is_broadcast(){return Err("Unicast IPv4 only".into());}
        if self.active()||self.stopping(){return Err("Stop the previous check and wait for it to exit".into());}
        if let Some(t)=self.worker.take(){let _=t.join();}
        self.samples.clear();self.last_reply=None;self.error=None;self.last_sequence=None;self.discarded=0;
        let (tx,rx)=mpsc::sync_channel(8);
        let run=Arc::new(AtomicBool::new(true));let worker_run=run.clone();
        let worker=thread::Builder::new().name("Horizon ICMP".into()).spawn(move||{
            let mut sequence=0u64;
            while worker_run.load(Ordering::Relaxed){
                let start=Instant::now();let result=echo(ip);
                let failed=result.is_err();sequence=sequence.saturating_add(1);let _=tx.try_send((sequence,Instant::now(),result));
                if failed{break;}
                while worker_run.load(Ordering::Relaxed)&&start.elapsed()<Duration::from_secs(2){
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }).map_err(|e|e.to_string())?;
        self.target=Some(ip);self.cancel=run;self.worker=Some(worker);self.rx=Some(rx);
        Ok(())
    }
    pub fn poll(&mut self){
        if self.worker.as_ref().is_some_and(|t|t.is_finished()){let _=self.worker.take().map(|t|t.join());}
        let Some(rx)=self.rx.as_ref()else{return;};
        while let Ok((sequence,at,result))=rx.try_recv(){
            // A GUI queue overflow is not ICMP packet loss. Reset the window so jitter never spans missing samples.
            let expected=self.last_sequence.unwrap_or(0).saturating_add(1);
            if sequence>expected{self.discarded=self.discarded.saturating_add(sequence-expected);self.samples.clear();}
            self.last_sequence=Some(sequence);
            match result{
                Ok(s)=>{self.last_reply=Some(at);self.samples.push_back(s);
                    if self.samples.len()>30{self.samples.pop_front();}},
                Err(e)=>self.error=Some(e),
            }
        }
    }
    pub fn stats(&self)->Option<(Option<u32>,f32,usize)>{
        if !self.active()||self.last_reply.is_none_or(|t|t.elapsed()>Duration::from_secs(5))
            ||self.samples.is_empty(){return None;}
        let count=self.samples.len();
        let misses=self.samples.iter().filter(|s|s.rtt.is_none()).count();
        Some((self.samples.back()?.rtt,misses as f32*100.0/count as f32,count))
    }
    pub fn stop(&mut self){
        self.cancel.store(false,Ordering::Relaxed);
        if self.worker.as_ref().is_some_and(|t|t.is_finished()){let _=self.worker.take().map(|t|t.join());}
        self.samples.clear();self.rx=None;self.target=None;self.last_reply=None;self.last_sequence=None;
    }
}
impl Drop for Probe{fn drop(&mut self){self.stop();if let Some(t)=self.worker.take(){let _=t.join();}}}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn local_icmp_returns_real_reply(){
        assert!(echo(Ipv4Addr::LOCALHOST).unwrap().rtt.is_some());
    }
    #[test]fn refuses_broadcast_and_non_unicast(){
        assert!(echo(Ipv4Addr::BROADCAST).is_err());
        assert!(echo(Ipv4Addr::UNSPECIFIED).is_err());
        assert!(echo(Ipv4Addr::new(224,0,0,1)).is_err());
    }
}

#[derive(Clone,Debug,PartialEq)]pub struct Distribution{pub replies:usize,pub median:Option<f64>,pub p95:Option<u32>,pub jitter:Option<f64>}
pub fn distribution(samples:&VecDeque<Sample>)->Distribution{
 let mut values=samples.iter().filter_map(|s|s.rtt).collect::<Vec<_>>();values.sort_unstable();let n=values.len();
 let median=if n==0{None}else{Some((values[(n-1)/2] as f64+values[n/2] as f64)*0.5)};
 let p95=if n==0{None}else{Some(values[((n*95).div_ceil(100)).saturating_sub(1)])};
 let(mut previous,mut count,mut sum)=(None::<u32>,0usize,0u64);
 for s in samples{if let(Some(a),Some(b))=(previous,s.rtt){sum+=u64::from(a.abs_diff(b));count+=1;}previous=s.rtt;}
 Distribution{replies:n,median,p95,jitter:(count>0).then(||sum as f64/count as f64)}
}
impl Probe{pub fn distribution(&self)->Option<Distribution>{self.stats()?;Some(distribution(&self.samples))}}
#[cfg(test)]mod distribution_tests{use super::*;#[test]fn actual_reply_quantiles_and_loss_break_jitter(){let a=[Some(10),Some(30),None,Some(90),Some(100)].into_iter().map(|rtt|Sample{rtt}).collect();let d=distribution(&a);assert_eq!(d.replies,4);assert_eq!(d.median,Some(60.));assert_eq!(d.p95,Some(100));assert_eq!(d.jitter,Some(15.));assert_eq!(distribution(&VecDeque::new()).jitter,None);}}

#[cfg(test)]mod stale_tests{use super::*;#[test]fn delayed_gui_does_not_refresh_sample_timestamp(){let mut p=Probe::default();let(tx,rx)=mpsc::sync_channel(8);p.rx=Some(rx);let at=Instant::now()-Duration::from_secs(20);tx.send((1,at,Ok(Sample{rtt:Some(5)}))).unwrap();p.poll();assert_eq!(p.last_reply,Some(at));assert!(p.stats().is_none());p.stop();assert!(p.samples.is_empty());}}

#[cfg(test)]mod v88_tests{use super::*;#[test]fn missing_gui_samples_reset_window_not_counted_as_network_loss(){let mut p=Probe::default();let(tx,rx)=mpsc::sync_channel(8);p.rx=Some(rx);let n=Instant::now();for (sequence,rtt) in [(1,Some(10)),(2,None),(5,Some(90)),(6,Some(100))]{tx.send((sequence,n,Ok(Sample{rtt}))).unwrap();}p.poll();assert_eq!(p.discarded,2);assert_eq!(p.samples.len(),2);assert_eq!(distribution(&p.samples).jitter,Some(10.));}}
