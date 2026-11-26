//! Honest capture-state diagnostics, never inferred game-engine FPS.
use crate::capture_health::Snapshot;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Status{Idle,LookRejected,Paused,NoFrames,OutputFrames}
pub fn diagnose(health:Option<&Snapshot>,presented:u64)->Status{
 match health{
  None=>Status::Idle,
  Some(v)if v.live_look_rejected=>Status::LookRejected,
  Some(v)if v.paused=>Status::Paused,
  Some(_)if presented==0=>Status::NoFrames,
  Some(_)=>Status::OutputFrames,
 }
}
pub fn message(s:Status,czech:bool)->&'static str{
 match(s,czech){
  (Status::Idle,true)=>"Vyber hru a zapni GPU výstup; samotný preset obrazovku nemění.",
  (Status::Idle,false)=>"Select a game and start GPU output; choosing a preset alone changes no screen.",
  (Status::LookRejected,true)=>"Shader byl odmítnut. Přepni na jednodušší efekt; předchozí platný efekt zůstal.",
  (Status::LookRejected,false)=>"GPU shader rejected. Try a simpler look; the last valid look was retained.",
  (Status::Paused,true)=>"Snímání je pozastavené. Vrať se do cílové hry pomocí Alt+Tab.",
  (Status::Paused,false)=>"Capture is paused. Return to the selected game with Alt+Tab.",
  (Status::NoFrames,true)=>"Zatím žádný odeslaný snímek. Zkontroluj okno hry a pak zkus samostatný WGC výstup.",
  (Status::NoFrames,false)=>"No frames submitted yet. Check game window, then try separate WGC output.",
  (Status::OutputFrames,true)=>"Horizon odesílá snímky do vlastního výstupu. Původní okno hry tím není ověřené.",
  (Status::OutputFrames,false)=>"Horizon is submitting frames to its own output. The original game window is not certified.",
 }
}
#[cfg(test)]mod tests{
 use super::*;
 use std::sync::atomic::Ordering;
 #[test]fn idle_is_not_success(){assert_eq!(diagnose(None,0),Status::Idle);}
 #[test]fn new_worker_without_frames_is_not_success(){
  let h=crate::capture_health::Health::new();
  assert_eq!(diagnose(Some(&h.snapshot()),0),Status::NoFrames);
 }
 #[test]fn rejected_shader_precedes_frame_counter(){
  let h=crate::capture_health::Health::new();
  h.live_look_rejected.store(true,Ordering::Relaxed);
  assert_eq!(diagnose(Some(&h.snapshot()),10),Status::LookRejected);
 }
 #[test]fn paused_worker_is_not_success(){
  let h=crate::capture_health::Health::new();
  h.paused.store(true,Ordering::Relaxed);
  assert_eq!(diagnose(Some(&h.snapshot()),30),Status::Paused);
 }
 #[test]fn reported_frames_are_output_only(){
  let h=crate::capture_health::Health::new();
  assert_eq!(diagnose(Some(&h.snapshot()),30),Status::OutputFrames);
  assert!(message(Status::OutputFrames,false).contains("not certified"));
 }
}
