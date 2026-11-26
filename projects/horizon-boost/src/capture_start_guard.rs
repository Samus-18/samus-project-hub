//! Passive, hardware-safe validation before starting a new native capture.
 //! Does not acquire frames, initialize D3D, activate a window or alter a game.
use crate::{resource_guard,win};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum LaunchAdvice{Ready,LowRamWarning,BackgroundGame,MonitorFillingUnverified}
pub fn classify(mem:Option<resource_guard::PhysicalMemory>,p:&win::CapturePreflight)->Result<LaunchAdvice,String>{
 resource_guard::allowed_to_start(mem)?;
 if p.width<160||p.height<90{return Err("Selected game window is too small to capture".into());}
 if p.pid==0{return Err("No valid game PID".into());}
 if resource_guard::risk(mem)==resource_guard::Risk::LowWarning{return Ok(LaunchAdvice::LowRamWarning);}
 if p.monitor_filling{return Ok(LaunchAdvice::MonitorFillingUnverified);}
 if !p.foreground{return Ok(LaunchAdvice::BackgroundGame);}
 Ok(LaunchAdvice::Ready)
}
pub fn preflight(pid:u32)->Result<LaunchAdvice,String>{
 let mem=resource_guard::read();
 resource_guard::allowed_to_start(mem)?;
 let p=win::capture_preflight(pid)?;
 classify(mem,&p)
}
#[cfg(test)]mod tests{
 use super::*;
 fn window()->win::CapturePreflight{win::CapturePreflight{
  pid:123,title:"Minecraft".into(),class_name:"GLFW30".into(),width:1920,height:1080,
  foreground:true,monitor_filling:false,minecraft_java:true,dpi:96,monitor_width:1920,monitor_height:1080
 }}
 fn memory(free:u64)->Option<resource_guard::PhysicalMemory>{Some(resource_guard::PhysicalMemory{free_mib:free,total_mib:16000,system_load:42})}
 #[test]fn critical_memory_never_launches(){assert!(classify(memory(399),&window()).is_err());}
 #[test]fn low_memory_is_warning_above_hard_limit(){assert_eq!(classify(memory(900),&window()).unwrap(),LaunchAdvice::LowRamWarning);}
 #[test]fn monitor_filling_is_not_misreported_as_exclusive(){
  let mut w=window();w.monitor_filling=true;assert_eq!(classify(memory(6000),&w).unwrap(),LaunchAdvice::MonitorFillingUnverified);
 }
 #[test]fn background_game_can_start_but_is_not_certified(){
  let mut w=window();w.foreground=false;assert_eq!(classify(memory(6000),&w).unwrap(),LaunchAdvice::BackgroundGame);
 }
 #[test]fn invalid_dimensions_rejected(){let mut w=window();w.width=0;assert!(classify(None,&w).is_err());}
}
