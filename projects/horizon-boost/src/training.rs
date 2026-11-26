//! Purely manual, non-invasive gameplay practice tools.
//! No Roblox memory reads, user input injection or automation.
use serde::{Serialize,Deserialize};
use std::time::{Duration,Instant,SystemTime,UNIX_EPOCH};
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq,Default)]
#[serde(default)]
pub struct TrainingProgress{
    pub clears:u32,
    pub fails:u32,
    pub streak:u32,
    pub best_streak:u32,
    pub reactions_ms:Vec<u32>,
}
impl TrainingProgress{
    pub fn clear(&mut self){
        self.clears=self.clears.saturating_add(1);
        self.streak=self.streak.saturating_add(1);
        self.best_streak=self.best_streak.max(self.streak);
    }
    pub fn fail(&mut self){
        self.fails=self.fails.saturating_add(1);
        self.streak=0;
    }
    pub fn record_reaction(&mut self,ms:u32){
        if (50..=5000).contains(&ms){
            self.reactions_ms.push(ms);
            if self.reactions_ms.len()>30{
                self.reactions_ms.drain(..self.reactions_ms.len()-30);
            }
        }
    }
    pub fn best_reaction(&self)->Option<u32>{
        self.reactions_ms.iter().copied().min()
    }
    pub fn avg_reaction(&self)->Option<u32>{
        if self.reactions_ms.is_empty(){return None;}
        Some((self.reactions_ms.iter().map(|i|*i as u64).sum::<u64>()
            /self.reactions_ms.len() as u64) as u32)
    }
    pub fn sanitize(&mut self){
        if self.reactions_ms.len()>30{
            self.reactions_ms.drain(..self.reactions_ms.len()-30);
        }
        self.reactions_ms.retain(|i|(50..=5000).contains(i));
        self.streak=self.streak.min(self.clears);
        self.best_streak=self.best_streak.max(self.streak);
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Phase { Idle, Waiting, Ready, TooEarly, Result(u32) }
pub struct ReactionTrainer{
    phase:Phase,
    started:Option<Instant>,
    delay:Duration,
}
impl Default for ReactionTrainer{
    fn default()->Self{Self{phase:Phase::Idle,started:None,delay:Duration::ZERO}}
}
impl ReactionTrainer{
    pub fn phase(&self)->Phase{self.phase}
    pub fn start(&mut self){
        let entropy=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        self.delay=Duration::from_millis(850+(entropy%1750) as u64);
        self.started=Some(Instant::now());
        self.phase=Phase::Waiting;
    }
    pub fn tick(&mut self){
        if self.phase==Phase::Waiting&&self.started.is_some_and(|s|s.elapsed()>=self.delay){
            self.started=Some(Instant::now());
            self.phase=Phase::Ready;
        }
    }
    pub fn tap(&mut self)->Option<u32>{
        match self.phase{
            Phase::Waiting=>{
                self.phase=Phase::TooEarly;
                self.started=None;
                None
            },
            Phase::Ready=>{
                let ms=self.started.map(|s|s.elapsed().as_millis().min(5000) as u32)
                    .unwrap_or(0);
                self.phase=Phase::Result(ms);
                self.started=None;
                Some(ms)
            },
            _=>None
        }
    }
    pub fn reset(&mut self){*self=Self::default();}
    pub fn ongoing(&self)->bool{
        matches!(self.phase,Phase::Waiting|Phase::Ready)
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn manual_training_score_is_saturating(){
        let mut p=TrainingProgress::default();
        p.clear();p.clear();p.fail();p.clear();
        assert_eq!((p.clears,p.fails,p.streak,p.best_streak),(3,1,1,2));
        let s=serde_json::to_string(&p).unwrap();
        assert_eq!(p,serde_json::from_str(&s).unwrap());
    }
    #[test]fn recorded_reactions_are_bounded_and_validated(){
        let mut p=TrainingProgress::default();
        for i in 0..50{p.record_reaction(100+i);}
        assert_eq!(p.reactions_ms.len(),30);
        assert_eq!(p.best_reaction(),Some(120));
        p.record_reaction(4);p.record_reaction(20000);
        assert_eq!(p.reactions_ms.len(),30);
    }
    #[test]fn reaction_starts_manual_no_game_automation(){
        let mut watch=ReactionTrainer::default();
        assert_eq!(watch.phase(),Phase::Idle);
        watch.start();assert_eq!(watch.phase(),Phase::Waiting);
        assert_eq!(watch.tap(),None);
        assert_eq!(watch.phase(),Phase::TooEarly);
        watch.reset();assert!(!watch.ongoing());
    }
}
