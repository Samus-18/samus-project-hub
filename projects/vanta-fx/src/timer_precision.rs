//! Opt-in high precision Windows multimedia timer request for Horizon Boost's
//! OWN process. Never sets another game's clocks, device drivers or priorities.
//! Use only when user specifically enables it; high timer resolution may consume
//! more power. Every successful timeBeginPeriod has paired timeEndPeriod.
#[link(name="winmm")]
unsafe extern "system" {
    fn timeBeginPeriod(period_ms:u32)->u32;
    fn timeEndPeriod(period_ms:u32)->u32;
}
const REQUEST_MS:u32=1;
#[derive(Debug,Default)]
pub struct TimerPrecision{active:bool}
impl TimerPrecision{
    pub fn active(&self)->bool{self.active}
    pub fn enable(&mut self)->Result<String,String>{
        if self.active(){return Ok("Already active (1 ms request)".into());}
        let code=unsafe{timeBeginPeriod(REQUEST_MS)};
        if code!=0{
            return Err(format!("Windows rejected 1 ms timer request (MMRESULT {code}); unchanged"));
        }
        self.active=true;
        Ok("1 ms Windows timer requested for Horizon Boost own process only".into())
    }
    pub fn disable(&mut self)->Result<(),String>{
        if !self.active{return Ok(());}
        let code=unsafe{timeEndPeriod(REQUEST_MS)};
        if code!=0{
            return Err(format!("Windows could not release 1 ms timer (MMRESULT {code})"));
        }
        self.active=false;
        Ok(())
    }
}
impl Drop for TimerPrecision{
    fn drop(&mut self){let _=self.disable();}
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn opt_in_timer_is_paired_and_reversible(){
        let mut timer=TimerPrecision::default();
        assert!(!timer.active());
        assert!(timer.disable().is_ok());
        if timer.enable().is_ok(){
            assert!(timer.active());
            assert!(timer.enable().is_ok(),"duplicate opt-in must not double-count requests");
            assert!(timer.disable().is_ok());
            assert!(!timer.active());
            assert!(timer.disable().is_ok());
        }else{
            assert!(!timer.active(),"denied request must not claim enablement");
        }
    }
}
