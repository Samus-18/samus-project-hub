//! Optional Windows game scheduler mode. This is an actual OS scheduling API,
//! not an FPS simulation. It does not touch GPU clocks, game memory or files.
//! No real-time/high priority, no system-wide power-plan or registry changes.
use std::ffi::c_void;
type Handle=*mut c_void;
const PROCESS_SET_INFORMATION:u32=0x0200;
const PROCESS_QUERY_LIMITED_INFORMATION:u32=0x1000;
const NORMAL_PRIORITY_CLASS:u32=0x20;
const ABOVE_NORMAL_PRIORITY_CLASS:u32=0x8000;
#[link(name="kernel32")]
unsafe extern "system"{
    fn OpenProcess(desired_access:u32,inherit:i32,pid:u32)->Handle;
    fn CloseHandle(process:Handle)->i32;
    fn GetPriorityClass(process:Handle)->u32;
    fn SetPriorityClass(process:Handle,priority:u32)->i32;
}
pub struct GameBoost{
    handle:Handle,
    pub pid:u32,
    original:u32,
}
impl Default for GameBoost{
    fn default()->Self{
        Self{handle:std::ptr::null_mut(),pid:0,original:0}
    }
}
impl GameBoost{
    pub fn active(&self)->bool{!self.handle.is_null()}
    pub fn start(&mut self,pid:u32)->Result<String,String>{
        if pid==0||pid==std::process::id(){
            return Err("Cannot boost an invalid process or Horizon Boost itself".into());
        }
        if self.active(){
            if self.pid==pid{return Ok(format!("Game Boost already active for PID {pid}"));}
            self.stop()?;
        }
        let access=PROCESS_QUERY_LIMITED_INFORMATION|PROCESS_SET_INFORMATION;
        let h=unsafe{OpenProcess(access,0,pid)};
        if h.is_null(){
            return Err("Windows refused game priority access; no changes were made".into());
        }
        let priority=unsafe{GetPriorityClass(h)};
        if priority==0{
            unsafe{CloseHandle(h);}
            return Err("Cannot read current game scheduling priority".into());
        }
        // NEVER raise an already-high or privileged priority, nor change
        // idle/below-normal applications unexpectedly.
        if priority!=NORMAL_PRIORITY_CLASS{
            unsafe{CloseHandle(h);}
            return Err(format!("Game is not in NORMAL priority (actual code 0x{priority:X}); unchanged"));
        }
        if unsafe{SetPriorityClass(h,ABOVE_NORMAL_PRIORITY_CLASS)}==0{
            unsafe{CloseHandle(h);}
            return Err("Windows denied ABOVE_NORMAL scheduler priority; unchanged".into());
        }
        let actual=unsafe{GetPriorityClass(h)};
        if actual!=ABOVE_NORMAL_PRIORITY_CLASS{
            unsafe{
                SetPriorityClass(h,priority);
                CloseHandle(h);
            }
            return Err("Windows did not confirm the new priority; reverted".into());
        }
        self.handle=h;
        self.pid=pid;
        self.original=priority;
        Ok(format!("PID {pid}: NORMAL → ABOVE NORMAL (verified by Windows)"))
    }
    pub fn stop(&mut self)->Result<(),String>{
        if self.handle.is_null(){return Ok(());}
        let h=std::mem::replace(&mut self.handle,std::ptr::null_mut());
        let actual=unsafe{GetPriorityClass(h)};
        let result=if actual==ABOVE_NORMAL_PRIORITY_CLASS{
            let ok=unsafe{SetPriorityClass(h,self.original)};
            if ok==0 {
                Err("Could not restore original priority (game may have exited)".into())
            }else{Ok(())}
        }else{Ok(())};
        unsafe{CloseHandle(h);}
        self.pid=0;
        self.original=0;
        result
    }
}
impl Drop for GameBoost{
    fn drop(&mut self){let _=self.stop();}
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn opt_in_never_targets_self(){
        let mut b=GameBoost::default();
        assert!(b.start(std::process::id()).is_err());
        assert!(!b.active());
    }
    #[test]fn stop_is_idempotent(){
        let mut b=GameBoost::default();
        assert!(b.stop().is_ok());
        assert!(b.stop().is_ok());
    }
}
