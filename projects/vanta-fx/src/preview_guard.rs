//! Opt-in feedback for Horizon's own 1x preview, not a game FPS limiter.
use std::time::{Duration,Instant};
#[derive(Default,Clone)]
pub struct Guard{
    last:Option<Instant>,bad:u8,good:u8,config:Option<(u32,u32)>,
    pub effective:Option<u32>,
}
impl Guard{
    pub fn reset(&mut self){*self=Self::default();}
    pub fn observe(&mut self,baseline:u32,target:u32,actual:Option<f32>,now:Instant)->Option<u32>{
        let baseline=crate::preview_pacer::valid_limit(baseline);
        let target=target.clamp(30,165).min(if baseline==0{120}else{baseline});
        let config=(baseline,target);
        if self.config!=Some(config){
            let changed=self.config.is_some();self.reset();self.config=Some(config);
            if changed{self.last=Some(now);return Some(baseline);}
        }
        if self.last.is_some_and(|t|now.saturating_duration_since(t)<Duration::from_secs(3)){return None;}
        self.last=Some(now);
        let current=self.effective.unwrap_or(baseline);
        let ceiling=if baseline==0{120}else{baseline};
        let Some(actual)=actual.filter(|x|x.is_finite()&&*x>0.0&&*x<=2000.) else{
            self.bad=0;self.good=0;return None;
        };
        let expected=target.min(if current==0{165}else{current}) as f32;
        if actual<expected*0.83{
            self.bad=self.bad.saturating_add(1);self.good=0;
        }else if actual>=expected*0.97{
            self.good=self.good.saturating_add(1);self.bad=0;
        }else{self.good=0;self.bad=0;}
        let levels=[30,45,60,75,90,100,120,144,165];
        let current_value=if current==0{120}else{current};
        let next=if self.bad>=2{
            self.bad=0;
            levels.iter().rev().copied().find(|&v|v<current_value).unwrap_or(30)
        }else if self.good>=4{
            self.good=0;
            let up=levels.iter().copied().find(|&v|v>current_value&&v<=ceiling).unwrap_or(current_value);
            if baseline==0&&up==120{0}else{up}
        }else{return None;};
        if next!=current{self.effective=Some(next);Some(next)}else{None}
    }
}
#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn requires_measured_low_fps_and_recovers_slowly(){
    let now=Instant::now();let mut g=Guard::default();
    assert_eq!(g.observe(60,60,None,now),None);
    assert_eq!(g.observe(60,60,Some(35.0),now+Duration::from_secs(3)),None);
    assert_eq!(g.observe(60,60,Some(35.0),now+Duration::from_secs(6)),Some(45));
    for i in [9,12,15]{assert_eq!(g.observe(60,60,Some(61.0),now+Duration::from_secs(i)),None);}
    assert_eq!(g.observe(60,60,Some(61.0),now+Duration::from_secs(18)),Some(60));
 }
}

#[cfg(test)]mod change_tests{use super::*;#[test]fn user_cap_changes_override_adaptive_state(){let n=Instant::now();let mut g=Guard::default();g.observe(90,120,Some(35.),n);assert_eq!(g.observe(90,120,Some(35.),n+Duration::from_secs(3)),Some(75));assert_eq!(g.observe(30,120,Some(60.),n+Duration::from_secs(4)),Some(30));assert!(g.effective.is_none());}}

#[cfg(test)]mod capped_recovery{use super::*;#[test]fn measured_cap_cannot_permanently_trap_recovery(){let n=Instant::now();let mut g=Guard::default();assert_eq!(g.observe(60,60,Some(35.),n),None);assert_eq!(g.observe(60,60,Some(35.),n+Duration::from_secs(3)),Some(45));for i in [6,9,12]{assert_eq!(g.observe(60,60,Some(45.),n+Duration::from_secs(i)),None);}assert_eq!(g.observe(60,60,Some(45.),n+Duration::from_secs(15)),Some(60));}}
