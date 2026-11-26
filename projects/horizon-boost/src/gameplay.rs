//! Practical optional gameplay tools with no game memory reads or input automation.
//! User controls all timers/checklists manually; they do not claim to track
//! actual abilities, quest state or enemy locations.
use serde::{Deserialize,Serialize};
use std::time::{Duration,Instant};
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
#[serde(default)]
pub struct QuestTask {pub label:String,pub done:bool}
impl Default for QuestTask{
    fn default()->Self{Self{label:String::new(),done:false}}
}
pub fn sanitize_tasks(tasks:&mut Vec<QuestTask>){
    tasks.truncate(20);
    for task in tasks.iter_mut(){
        task.label=task.label.trim().chars().take(90).collect();
    }
    tasks.retain(|t|!t.label.is_empty());
}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(default)]
pub struct CooldownSpec{pub label:String,pub duration_seconds:u32}
impl Default for CooldownSpec{
    fn default()->Self{Self{label:"Skill / potion".into(),duration_seconds:30}}
}
impl CooldownSpec{
    pub fn sanitize(&mut self){
        self.label=self.label.chars().take(40).collect();
        if self.label.trim().is_empty(){self.label="Cooldown".into();}
        self.duration_seconds=self.duration_seconds.clamp(3,600);
    }
}
#[derive(Clone,Debug,Default,Serialize,Deserialize,PartialEq,Eq)]
#[serde(default)]
pub struct GameMoment{
    pub timestamp_unix:u64,
    pub timer_elapsed_secs:u64,
    pub label:String,
}
pub fn sanitize_moments(moments:&mut Vec<GameMoment>){
    if moments.len()>60{moments.drain(..moments.len()-60);}
    for m in moments.iter_mut(){
        m.label=m.label.trim().chars().take(48).collect();
        if m.label.is_empty(){m.label="Moment".into();}
    }
}
#[derive(Default)]
pub struct CooldownClock{started:Option<Instant>}
impl CooldownClock{
    pub fn start(&mut self){self.started=Some(Instant::now());}
    pub fn reset(&mut self){self.started=None;}
    pub fn remaining(&self,duration_seconds:u32)->Option<Duration>{
        let start=self.started?;
        let full=Duration::from_secs(duration_seconds.clamp(3,600) as u64);
        Some(full.saturating_sub(start.elapsed()))
    }
    pub fn active(&self,duration_seconds:u32)->bool{
        self.remaining(duration_seconds).is_some_and(|d|d>Duration::ZERO)
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn tasks_sanitize_without_touching_game(){
        let mut rows=(0..30).map(|i|QuestTask{label:format!("  quest {i}  "),done:false}).collect::<Vec<_>>();
        sanitize_tasks(&mut rows);
        assert_eq!(rows.len(),20);
        assert_eq!(rows[0].label,"quest 0");
    }
    #[test]fn countdown_is_manual_and_never_negative(){
        let mut watch=CooldownClock::default();
        assert!(watch.remaining(15).is_none());
        watch.start();
        assert!(watch.active(15));
        assert!(watch.remaining(15).unwrap()<=Duration::from_secs(15));
        watch.reset();
        assert!(watch.remaining(15).is_none());
    }
    #[test]fn local_moments_are_bounded_and_serializable(){
        let mut v=(0..120).map(|i|GameMoment{
            timestamp_unix:i,
            timer_elapsed_secs:i,
            label:"  A long moment  ".into(),
        }).collect::<Vec<_>>();
        sanitize_moments(&mut v);
        assert_eq!(v.len(),60);
        assert_eq!(v[0].timestamp_unix,60);
        assert_eq!(v[0].label,"A long moment");
        assert_eq!(v,serde_json::from_str::<Vec<GameMoment>>(
            &serde_json::to_string(&v).unwrap()).unwrap());
    }
    #[test]fn cooldown_clamps_user_input(){
        let mut spec=CooldownSpec{label:String::from("x").repeat(200),duration_seconds:3000};
        spec.sanitize();
        assert_eq!(spec.label.len(),40);
        assert_eq!(spec.duration_seconds,600);
    }
}
