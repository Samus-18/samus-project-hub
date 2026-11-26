//! Non-invasive AMD ADLX driver integration (Windows x64).
//! Calls the GPU vendor's documented COM-style C interfaces. Does NOT hook a game,
//! modify a game EXE, force windowed mode, or ship the proprietary AMD DLL.
//! ABI indices match AMD ADLX SDK 2.x C declarations in ISystem.h and I3DSettings.h.
//! State-changing methods are NEVER called by auto-probe, unit tests or startup.
use std::ffi::c_void;
type Ptr=*mut c_void;
#[link(name="kernel32")]
unsafe extern "system"{
    fn LoadLibraryExW(name:*const u16,file:Ptr,flags:u32)->Ptr;
    fn GetProcAddress(module:Ptr,name:*const u8)->*const c_void;
    fn FreeLibrary(module:Ptr)->i32;
}
type VersionFn=unsafe extern "C" fn(*mut u64)->i32;
type InitFn=unsafe extern "C" fn(u64,*mut Ptr)->i32;
type EndFn=unsafe extern "C" fn()->i32;
const ADLX_OK:i32=0;
#[repr(C)]
#[derive(Clone,Copy,Debug)]
pub struct IntRange{pub min_value:i32,pub max_value:i32,pub step:i32}
impl Default for IntRange{
    fn default()->Self{Self{min_value:0,max_value:100,step:1}}
}
#[derive(Clone,Debug,Default)]
pub struct DriverFeature{
    pub supported:bool,
    pub enabled:bool,
}
#[derive(Clone,Debug)]
pub struct DriverStatus{
    pub gpu_count:u32,
    pub image_sharpen:DriverFeature,
    pub sharpness:i32,
    pub sharpness_range:IntRange,
    pub anti_lag:DriverFeature
}
impl DriverStatus{
    pub fn valid_sharpness(&self,requested:i32)->i32{
        let min=self.sharpness_range.min_value;
        let max=self.sharpness_range.max_value;
        let step=self.sharpness_range.step.max(1);
        min+((requested.clamp(min,max)-min)/step)*step
    }
}
fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(Some(0)).collect()}
fn result_ok(code:i32)->bool{code==ADLX_OK||code==1}
fn api_error(operation:&str,result:i32)->String{
    format!("{operation} returned ADLX error {result}; no driver settings changed")
}
struct Adlx{
    dll:Ptr,
    system:Ptr,
    end:EndFn
}
impl Drop for Adlx{
    fn drop(&mut self){
        unsafe{
            if !self.system.is_null(){(self.end)();}
            if !self.dll.is_null(){FreeLibrary(self.dll);}
        }
    }
}
impl Adlx{
    fn new()->Result<Self,String>{
        let dll=unsafe{LoadLibraryExW(wide("amdadlx64.dll").as_ptr(),std::ptr::null_mut(),0x800)};
        if dll.is_null(){return Err("AMD ADLX library is not installed".into());}
        // Only export lookups; no side effects yet.
        let query=unsafe{GetProcAddress(dll,b"ADLXQueryFullVersion\0".as_ptr())};
        let initialize=unsafe{GetProcAddress(dll,b"ADLXInitialize\0".as_ptr())};
        let terminate=unsafe{GetProcAddress(dll,b"ADLXTerminate\0".as_ptr())};
        if query.is_null()||initialize.is_null()||terminate.is_null(){
            unsafe{FreeLibrary(dll);}
            return Err("AMD driver does not expose required ADLX functions".into());
        }
        let query:VersionFn=unsafe{std::mem::transmute(query)};
        let init:InitFn=unsafe{std::mem::transmute(initialize)};
        let end:EndFn=unsafe{std::mem::transmute(terminate)};
        let mut version=0u64;
        let result=unsafe{query(&mut version)};
        if !result_ok(result)||version==0{
            unsafe{FreeLibrary(dll);}
            return Err(api_error("ADLXQueryFullVersion",result));
        }
        let mut system=std::ptr::null_mut();
        let result=unsafe{init(version,&mut system)};
        if !result_ok(result)||system.is_null(){
            unsafe{FreeLibrary(dll);}
            return Err(api_error("ADLXInitialize",result));
        }
        Ok(Self{dll,system,end})
    }
    unsafe fn vtbl_method(ptr:Ptr,index:usize)->Result<*const c_void,String>{
        if ptr.is_null(){return Err("ADLX interface pointer is null".into());}
        let table=unsafe{*(ptr as *const *const *const c_void)};
        if table.is_null(){return Err("ADLX vtable is null".into());}
        let method=unsafe{*table.add(index)};
        if method.is_null(){return Err(format!("ADLX vtable method {index} unavailable"));}
        Ok(method)
    }
    unsafe fn release(ptr:Ptr){
        if ptr.is_null(){return;}
        if let Ok(method)=unsafe{Self::vtbl_method(ptr,1)}{
            let fun:unsafe extern "system" fn(Ptr)->i32=unsafe{std::mem::transmute(method)};
            unsafe{fun(ptr); }
        }
    }
    fn get_gpus(&self)->Result<Ptr,String>{
        let p=unsafe{Self::vtbl_method(self.system,1)?};
        let f:unsafe extern "system" fn(Ptr,*mut Ptr)->i32=unsafe{std::mem::transmute(p)};
        let mut list=std::ptr::null_mut();
        let result=unsafe{f(self.system,&mut list)};
        if result!=ADLX_OK||list.is_null(){return Err(api_error("GetGPUs",result));}
        Ok(list)
    }
    fn get_3d_service(&self)->Result<Ptr,String>{
        let p=unsafe{Self::vtbl_method(self.system,7)?};
        let f:unsafe extern "system" fn(Ptr,*mut Ptr)->i32=unsafe{std::mem::transmute(p)};
        let mut service=std::ptr::null_mut();
        let result=unsafe{f(self.system,&mut service)};
        if result!=ADLX_OK||service.is_null(){return Err(api_error("Get3DSettingsServices",result));}
        Ok(service)
    }
    fn gpu_count(list:Ptr)->Result<u32,String>{
        let p=unsafe{Self::vtbl_method(list,3)?};
        let f:unsafe extern "system" fn(Ptr)->u32=unsafe{std::mem::transmute(p)};
        Ok(unsafe{f(list)})
    }
    fn gpu_at(list:Ptr,index:u32)->Result<Ptr,String>{
        let p=unsafe{Self::vtbl_method(list,11)?};
        let f:unsafe extern "system" fn(Ptr,u32,*mut Ptr)->i32=unsafe{std::mem::transmute(p)};
        let mut gpu=std::ptr::null_mut();
        let result=unsafe{f(list,index,&mut gpu)};
        if result!=ADLX_OK||gpu.is_null(){return Err(api_error("GPUList.At_GPUList",result));}
        Ok(gpu)
    }
    fn feature(settings:Ptr,gpu:Ptr,index:usize)->Result<Ptr,String>{
        let p=unsafe{Self::vtbl_method(settings,index)?};
        let f:unsafe extern "system" fn(Ptr,Ptr,*mut Ptr)->i32=unsafe{std::mem::transmute(p)};
        let mut obj=std::ptr::null_mut();
        let result=unsafe{f(settings,gpu,&mut obj)};
        if result!=ADLX_OK||obj.is_null(){return Err(api_error("Get3DFeature",result));}
        Ok(obj)
    }
    fn bool_read(obj:Ptr,index:usize)->Result<bool,String>{
        let p=unsafe{Self::vtbl_method(obj,index)?};
        let f:unsafe extern "system" fn(Ptr,*mut u8)->i32=unsafe{std::mem::transmute(p)};
        let mut value=0u8;
        let result=unsafe{f(obj,&mut value)};
        if result!=ADLX_OK{return Err(api_error("ADLX.IsSupported/IsEnabled",result));}
        Ok(value!=0)
    }
    fn int_read(obj:Ptr,index:usize)->Result<i32,String>{
        let p=unsafe{Self::vtbl_method(obj,index)?};
        let f:unsafe extern "system" fn(Ptr,*mut i32)->i32=unsafe{std::mem::transmute(p)};
        let mut value=0i32;
        let result=unsafe{f(obj,&mut value)};
        if result!=ADLX_OK{return Err(api_error("ADLX.GetSharpness",result));}
        Ok(value)
    }
    fn range_read(obj:Ptr,index:usize)->Result<IntRange,String>{
        let p=unsafe{Self::vtbl_method(obj,index)?};
        let f:unsafe extern "system" fn(Ptr,*mut IntRange)->i32=unsafe{std::mem::transmute(p)};
        let mut value=IntRange::default();
        let result=unsafe{f(obj,&mut value)};
        if result!=ADLX_OK{return Err(api_error("ADLX.GetSharpnessRange",result));}
        if value.max_value<=value.min_value||value.step<=0{
            return Err("AMD returned an invalid Image Sharpening range".into());
        }
        Ok(value)
    }
    fn set_bool(obj:Ptr,index:usize,enabled:bool)->Result<(),String>{
        let p=unsafe{Self::vtbl_method(obj,index)?};
        let f:unsafe extern "system" fn(Ptr,u8)->i32=unsafe{std::mem::transmute(p)};
        let result=unsafe{f(obj,if enabled{1u8}else{0u8})};
        if !result_ok(result){return Err(api_error("ADLX.SetEnabled",result));}
        Ok(())
    }
    fn set_int(obj:Ptr,index:usize,value:i32)->Result<(),String>{
        let p=unsafe{Self::vtbl_method(obj,index)?};
        let f:unsafe extern "system" fn(Ptr,i32)->i32=unsafe{std::mem::transmute(p)};
        let result=unsafe{f(obj,value)};
        if !result_ok(result){return Err(api_error("ADLX.SetSharpness",result));}
        Ok(())
    }
}
/// Everything in this function is READ-ONLY on AMD driver state.
pub fn probe()->Result<DriverStatus,String>{
    query(None)
}
pub enum DriverChange {
    RadeonSharpen{enabled:bool,level:i32},
    AntiLag(bool),
}
/// Writes AMD driver settings ONLY in response to an explicit UI action.
pub fn apply(request:DriverChange)->Result<DriverStatus,String>{
    query(Some(request))
}
fn query(change:Option<DriverChange>)->Result<DriverStatus,String>{
    let session=Adlx::new()?;
    let list=session.get_gpus()?;
    let service=match session.get_3d_service(){
        Ok(p)=>p,
        Err(e)=>{
            unsafe{Adlx::release(list);}
            return Err(e);
        }
    };
    // For the current desktop this is the only installed Radeon RX 580. Do not
    // expose controls for multi-GPU systems until adapter/display mapping is added.
    let result=(||{
        let count=Adlx::gpu_count(list)?;
        if count!=1{return Err(format!("Detected {count} AMD GPUs; select a single GPU configuration to avoid modifying the wrong adapter"));}
        let gpu=Adlx::gpu_at(list,0)?;
        let res=(||{
            let ris=Adlx::feature(service,gpu,6).ok();
            let anti=Adlx::feature(service,gpu,3).ok();
            let mut ris_status=DriverFeature::default();
            let mut sharp=0i32;
            let mut sharp_range=IntRange::default();
            let mut anti_status=DriverFeature::default();
            if let Some(ptr)=ris{
                let outcome=(||{
                    ris_status.supported=Adlx::bool_read(ptr,3)?;
                    if ris_status.supported{
                        ris_status.enabled=Adlx::bool_read(ptr,4)?;
                        sharp_range=Adlx::range_read(ptr,5)?;
                        sharp=Adlx::int_read(ptr,6)?;
                    }
                    Ok::<(),String>(())
                })();
                unsafe{Adlx::release(ptr);}
                outcome?;
            }
            if let Some(ptr)=anti{
                let outcome=(||{
                    anti_status.supported=Adlx::bool_read(ptr,3)?;
                    if anti_status.supported{anti_status.enabled=Adlx::bool_read(ptr,4)?;}
                    Ok::<(),String>(())
                })();
                unsafe{Adlx::release(ptr);}
                outcome?;
            }
            let snapshot=DriverStatus{
                gpu_count:count,image_sharpen:ris_status,
                sharpness:sharp,sharpness_range:sharp_range,anti_lag:anti_status
            };
            if let Some(change)=change{
                match change {
                    DriverChange::RadeonSharpen{enabled,level}=>{
                        if !snapshot.image_sharpen.supported{
                            return Err("Driver reports Radeon Image Sharpening unsupported".into());
                        }
                        let obj=Adlx::feature(service,gpu,6)?;
                        let outcome=(||{
                            if enabled{
                                let sharp=snapshot.valid_sharpness(level);
                                Adlx::set_int(obj,8,sharp)?;
                                Adlx::set_bool(obj,7,true)?;
                            }else{Adlx::set_bool(obj,7,false)?;}
                            Ok::<(),String>(())
                        })();
                        unsafe{Adlx::release(obj);}
                        outcome?;
                    }
                    DriverChange::AntiLag(enabled)=>{
                        if !snapshot.anti_lag.supported{
                            return Err("Driver reports Radeon Anti-Lag unsupported".into());
                        }
                        let obj=Adlx::feature(service,gpu,3)?;
                        let outcome=Adlx::set_bool(obj,5,enabled);
                        unsafe{Adlx::release(obj);}
                        outcome?;
                    }
                }
                // Re-read for genuine, current state (not a pretend toggle).
                let ris=Adlx::feature(service,gpu,6).ok();
                let anti=Adlx::feature(service,gpu,3).ok();
                let mut verified=snapshot;
                if let Some(ris)=ris{
                    if verified.image_sharpen.supported{
                        verified.image_sharpen.enabled=Adlx::bool_read(ris,4)?;
                        verified.sharpness=Adlx::int_read(ris,6)?;
                    }
                    unsafe{Adlx::release(ris);}
                }
                if let Some(anti)=anti{
                    if verified.anti_lag.supported{
                        verified.anti_lag.enabled=Adlx::bool_read(anti,4)?;
                    }
                    unsafe{Adlx::release(anti);}
                }
                Ok(verified)
            }else{Ok(snapshot)}
        })();
        unsafe{Adlx::release(gpu);}
        res
    })();
    unsafe{
        Adlx::release(service);
        Adlx::release(list);
    }
    result
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn range_alignment_and_clamp(){
        let mut status=DriverStatus{
            gpu_count:1,image_sharpen:DriverFeature::default(),
            sharpness:0,sharpness_range:IntRange{min_value:20,max_value:80,step:5},
            anti_lag:DriverFeature::default()
        };
        assert_eq!(status.valid_sharpness(-100),20);
        assert_eq!(status.valid_sharpness(46),45);
        assert_eq!(status.valid_sharpness(999),80);
        status.sharpness_range=IntRange{min_value:0,max_value:100,step:1};
        assert_eq!(status.valid_sharpness(53),53);
    }
    #[test]fn safe_api_is_not_simulated(){
        assert!(result_ok(0));
        assert!(!result_ok(12));
    }
}
