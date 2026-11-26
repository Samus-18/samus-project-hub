//! Manual, reversible process affinity. Never auto-applies or changes clocks.
use std::ffi::c_void;
type Handle=*mut c_void;
#[link(name="kernel32")]
unsafe extern "system" {
    fn OpenProcess(access:u32,inherit:i32,pid:u32)->Handle;
    fn CloseHandle(handle:Handle)->i32;
    fn GetProcessAffinityMask(handle:Handle,process:*mut usize,system:*mut usize)->i32;
    fn SetProcessAffinityMask(handle:Handle,mask:usize)->i32;
}
fn masks(h:Handle)->Result<(usize,usize),String>{
    let(mut p,mut s)=(0,0);
    if unsafe{GetProcessAffinityMask(h,&mut p,&mut s)}==0||p==0||s==0{
        Err(format!("Affinity unavailable: {}",std::io::Error::last_os_error()))
    }else{Ok((p,s))}
}
fn open(pid:u32,write:bool)->Result<Handle,String>{
    if pid==0{return Err("Select a game first".into());}
    let h=unsafe{OpenProcess(0x0400|if write{0x0200}else{0},0,pid)};
    if h.is_null(){Err("Windows denied process affinity access".into())}else{Ok(h)}
}
pub fn read(pid:u32)->Result<usize,String>{
    let h=open(pid,false)?;let v=masks(h).map(|x|x.0);
    unsafe{CloseHandle(h);};v
}
pub fn validate(mask:usize,allowed:usize)->bool{mask!=0&&mask&!allowed==0}
pub struct Affinity{handle:Handle,pub pid:u32,original:usize,applied:usize}
impl Default for Affinity{fn default()->Self{Self{handle:std::ptr::null_mut(),pid:0,original:0,applied:0}}}
impl Affinity{
    pub fn active(&self)->bool{!self.handle.is_null()}
    pub fn apply(&mut self,pid:u32,mask:usize,expected:usize)->Result<String,String>{
        if pid==std::process::id(){return Err("Cannot tune Horizon itself".into());}
        if self.active(){return Err("Restore current affinity before applying another mask".into());}
        let h=open(pid,true)?;
        let result=(||{
            let (original,system)=masks(h)?;
            if original!=expected{return Err("Affinity changed; refresh the displayed CPU mask".into());}
            if !validate(mask,original&system){return Err("Select at least one originally allowed logical CPU".into());}
            if mask==original{return Ok("Already using selected logical CPUs".into());}
            if unsafe{SetProcessAffinityMask(h,mask)}==0{return Err("Windows rejected affinity".into());}
            if masks(h).map(|x|x.0)!=Ok(mask){
                unsafe{SetProcessAffinityMask(h,original);};
                return Err("Could not confirm affinity; attempted rollback".into());
            }
            self.handle=h;self.pid=pid;self.original=original;self.applied=mask;
            Ok(format!("Windows confirmed {} logical CPUs",mask.count_ones()))
        })();
        if self.handle!=h{unsafe{CloseHandle(h);}}
        result
    }
    pub fn restore(&mut self)->Result<String,String>{
        if !self.active(){return Ok("No affinity change to restore".into());}
        let h=self.handle;
        let actual=masks(h);
        let result=match actual{
            Ok((p,_)) if p==self.applied=>{
                if unsafe{SetProcessAffinityMask(h,self.original)}!=0 && masks(h).map(|x|x.0)==Ok(self.original){
                    Ok("Original CPU affinity restored".into())
                }else{return Err("Windows could not restore affinity; try again".into());}
            }
            Ok(_)=>Ok("Affinity changed externally; left untouched".into()),
            Err(_)=>Ok("Target exited or affinity unavailable".into()),
        };
        unsafe{CloseHandle(h);};
        self.handle=std::ptr::null_mut();self.pid=0;self.original=0;self.applied=0;
        result
    }
}
impl Drop for Affinity{fn drop(&mut self){let _=self.restore();if self.active(){unsafe{CloseHandle(self.handle);}}}}
pub fn proof()->Result<String,String>{
    use std::os::windows::process::CommandExt;
    // Disposable local child, never a user's game. No external network.
    let mut child=std::process::Command::new("cmd.exe").args(["/d","/c","ping -n 10 127.0.0.1 > nul"])
        .creation_flags(0x08000000).spawn().map_err(|e|e.to_string())?;
    let result=(||{
        let pid=child.id();let original=read(pid)?;
        if original.count_ones()<2{return Err("Need two logical processors for affinity test".into());}
        let single=original&original.wrapping_neg();
        let mut guard=Affinity::default();
        guard.apply(pid,single,original)?;
        if read(pid)?!=single{return Err("Child affinity did not change".into());}
        guard.restore()?;
        if read(pid)?!=original{return Err("Child affinity was not restored".into());}
        Ok(format!("AFFINITY_PROOF=PASS ORIGINAL=0x{original:X} TEMP=0x{single:X} RESTORED=true"))
    })();
    let _=child.kill();let _=child.wait();result
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn mask_is_subset_and_nonzero(){assert!(validate(3,7));assert!(!validate(0,7));assert!(!validate(8,7));}
    #[test]fn current_process_read_only(){assert!(read(std::process::id()).unwrap()!=0);}
    #[test]fn cannot_apply_to_horizon(){assert!(Affinity::default().apply(std::process::id(),1,1).is_err());}
}
