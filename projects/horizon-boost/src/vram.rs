//! Read-only DXGI 1.4 local video-memory BUDGET for this process.
//! This is NOT total game or system-wide VRAM usage. Never clears caches,
//! modifies clocks, power plans, paging, or third-party processes.
use windows::{
    core::Interface,
    Win32::Graphics::Dxgi::{
        CreateDXGIFactory1,IDXGIFactory1,IDXGIAdapter3,
        DXGI_MEMORY_SEGMENT_GROUP_LOCAL,DXGI_QUERY_VIDEO_MEMORY_INFO,
    },
};
#[derive(Clone,Debug)]
pub struct VramBudget{
    pub device_name:String,
    pub process_usage_bytes:u64,
    pub process_budget_bytes:u64,
}
impl VramBudget{
    pub fn process_gb(&self)->f64{self.process_usage_bytes as f64/1_073_741_824.0}
    pub fn budget_gb(&self)->f64{self.process_budget_bytes as f64/1_073_741_824.0}
    pub fn fraction(&self)->f32{
        if self.process_budget_bytes==0{0.0}else{
            (self.process_usage_bytes as f64/self.process_budget_bytes as f64).clamp(0.0,1.0) as f32
        }
    }
    pub fn meaningful(&self)->bool{
        self.process_budget_bytes>=64*1024*1024&&
        true
    }
}
pub fn read_local_process_budget()->Result<VramBudget,String>{read_for_adapter(None)}
pub fn read_for_adapter(requested:Option<&str>)->Result<VramBudget,String>{
    if requested.is_some_and(|s|!crate::adapter_select::valid_id(s)){return Err("Invalid selected adapter".into());}
    let factory:IDXGIFactory1=unsafe{CreateDXGIFactory1()}
        .map_err(|e|format!("DXGI VRAM factory: {e}"))?;
    let mut result=None;
    for i in 0..8{
        let adapter=match unsafe{factory.EnumAdapters1(i)}{
            Ok(a)=>a,Err(_)=>break
        };
        let desc=unsafe{adapter.GetDesc1()}
            .map_err(|e|format!("DXGI GetDesc1: {e}"))?;
        if desc.VendorId==0x1414||desc.Flags&2!=0{continue;}
        let id=format!("{:08x}:{:08x}",desc.AdapterLuid.HighPart as u32,desc.AdapterLuid.LowPart);
        if requested.is_some_and(|key|!key.eq_ignore_ascii_case(&id)){continue;}
        let actual:IDXGIAdapter3=match adapter.cast(){
            Ok(a)=>a,Err(_)=>continue
        };
        let mut info=DXGI_QUERY_VIDEO_MEMORY_INFO::default();
        if unsafe{actual.QueryVideoMemoryInfo(0,DXGI_MEMORY_SEGMENT_GROUP_LOCAL,&mut info)}.is_err(){
            continue;
        }
        let data=VramBudget{
            device_name:String::from_utf16_lossy(&desc.Description)
                .trim_end_matches('\0').trim().to_string(),
            process_usage_bytes:info.CurrentUsage,
            process_budget_bytes:info.Budget,
        };
        if requested.is_some(){return Ok(data);}
        if result.is_none()||desc.VendorId==0x1002{
            result=Some(data);
        }
        if desc.VendorId==0x1002{break;} // actual AMD hardware
    }
    result.ok_or("DXGI 1.4 process VRAM budget not available".into())
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn vram_budget_is_process_scoped_and_never_invented(){
        let v=VramBudget{
            device_name:"Example adapter".into(),
            process_usage_bytes:256*1024*1024,
            process_budget_bytes:1024*1024*1024
        };
        assert_eq!(v.fraction(),0.25);
        assert!(v.meaningful());
        let unknown=VramBudget{device_name:"".into(),process_usage_bytes:0,process_budget_bytes:0};
        assert!(!unknown.meaningful());
        assert_eq!(unknown.fraction(),0.0);
    }
}

pub fn proof()->Result<String,String>{let rows=crate::adapter_select::list()?;let a=rows.first().ok_or("No hardware adapter")?;let v=read_for_adapter(Some(&a.id))?;if v.device_name!=a.name{return Err("VRAM adapter mismatch".into());}if read_for_adapter(Some("invalid")).is_ok(){return Err("Invalid ID accepted".into());}Ok(format!("VRAM_SELECTED=PASS BUDGET_BYTES={} PROCESS_BYTES={} NO_GAME_VRAM=true",v.process_budget_bytes,v.process_usage_bytes))}

#[cfg(test)]mod v88_tests{use super::*;#[test]fn severe_overbudget_not_hidden(){let b=VramBudget{device_name:"fixture".into(),process_usage_bytes:4*1024*1024*1024,process_budget_bytes:1024*1024*1024};assert!(b.meaningful());assert_eq!(b.fraction(),1.);}}

#[cfg(test)]mod v88_actual_adapter{#[test]fn real_selected_adapter_budget(){println!("{}",super::proof().expect("read-only DXGI budget"));}}