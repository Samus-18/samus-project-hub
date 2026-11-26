//! Explicit own-renderer adapter selection; no driver or game changes.
use windows::{core::{Interface,BOOL},Win32::{Foundation::HMODULE,Graphics::{Direct3D::{D3D_DRIVER_TYPE_HARDWARE,D3D_DRIVER_TYPE_UNKNOWN,D3D_FEATURE_LEVEL_11_0},Direct3D11::*,Dxgi::*}}};
#[derive(Clone,Debug)]pub struct Adapter{pub id:String,pub name:String,pub dedicated_mib:u64,pub vendor:u32}
fn id(d:&DXGI_ADAPTER_DESC1)->String{format!("{:08x}:{:08x}",d.AdapterLuid.HighPart as u32,d.AdapterLuid.LowPart)}
pub fn valid_id(s:&str)->bool{s.len()==17&&s.as_bytes()[8]==b':'&&s.as_bytes().iter().enumerate().all(|(i,c)|i==8||c.is_ascii_hexdigit())}
fn enumerate()->Result<Vec<(Adapter,IDXGIAdapter1)>,String>{
 let f:IDXGIFactory1=unsafe{CreateDXGIFactory1()}.map_err(|e|e.to_string())?;let mut out=Vec::new();
 for i in 0..32{let a=match unsafe{f.EnumAdapters1(i)}{Ok(a)=>a,Err(e) if e.code()==DXGI_ERROR_NOT_FOUND=>break,Err(e)=>return Err(e.to_string())};
  let d=unsafe{a.GetDesc1()}.map_err(|e|e.to_string())?;if d.Flags&(DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32)!=0{continue;}
  let end=d.Description.iter().position(|&x|x==0).unwrap_or(d.Description.len());
  out.push((Adapter{id:id(&d),name:String::from_utf16_lossy(&d.Description[..end]),dedicated_mib:d.DedicatedVideoMemory as u64/1048576,vendor:d.VendorId},a));
 }Ok(out)
}
pub fn list()->Result<Vec<Adapter>,String>{Ok(enumerate()?.into_iter().map(|x|x.0).collect())}
pub fn create(requested:Option<&str>)->Result<(ID3D11Device,ID3D11DeviceContext),String>{
 let adapter:Option<IDXGIAdapter>=if let Some(key)=requested{
  if !valid_id(key){return Err("Invalid renderer adapter ID".into());}
  Some(enumerate()?.into_iter().find(|x|x.0.id.eq_ignore_ascii_case(key)).ok_or("Selected renderer adapter unavailable; select Automatic")?.1.cast().map_err(|e|e.to_string())?)
 }else{None};
 let mut dev=None;let mut ctx=None;
 unsafe{D3D11CreateDevice(adapter.as_ref(),if adapter.is_some(){D3D_DRIVER_TYPE_UNKNOWN}else{D3D_DRIVER_TYPE_HARDWARE},HMODULE::default(),D3D11_CREATE_DEVICE_BGRA_SUPPORT,Some(&[D3D_FEATURE_LEVEL_11_0]),D3D11_SDK_VERSION,Some(&mut dev),None,Some(&mut ctx))}.map_err(|e|format!("Selected own renderer D3D11 device: {e}"))?;
 Ok((dev.ok_or("D3D11 device missing")?,ctx.ok_or("D3D11 context missing")?))
}
pub fn actual_id(d:&ID3D11Device)->Result<String,String>{
 let dx:IDXGIDevice=d.cast().map_err(|e|e.to_string())?;let a:IDXGIAdapter1=unsafe{dx.GetAdapter()}.map_err(|e|e.to_string())?.cast().map_err(|e|e.to_string())?;
 Ok(id(&unsafe{a.GetDesc1()}.map_err(|e|e.to_string())?))
}
pub fn tearing(f:&IDXGIFactory2)->Option<bool>{
 let f:IDXGIFactory5=f.cast().ok()?;let mut supported=BOOL(0);
 unsafe{f.CheckFeatureSupport(DXGI_FEATURE_PRESENT_ALLOW_TEARING,(&mut supported as *mut BOOL).cast(),std::mem::size_of::<BOOL>() as u32)}.ok()?;
 Some(supported.as_bool())
}
pub fn tearing_flag(interval:u32,created:bool)->DXGI_PRESENT{if interval==0&&created{DXGI_PRESENT_ALLOW_TEARING}else{DXGI_PRESENT(0)}}
pub fn proof()->Result<String,String>{
 let rows=list()?;let first=rows.first().ok_or("No hardware adapter")?;
 let(d,_)=create(Some(&first.id))?;if actual_id(&d)?!=first.id{return Err("Device adapter differs from selection".into());}
 let auto=create(None)?.0;let actual=actual_id(&auto)?;
 if !rows.iter().any(|v|v.id==actual){return Err("Default adapter missing from hardware list".into());}
 let f:IDXGIFactory2=unsafe{CreateDXGIFactory1()}.map_err(|e|e.to_string())?;
 Ok(format!("ADAPTER_SELECTION=PASS HARDWARE_COUNT={} SELECTED={:?} AUTO_MATCH=true TEARING_CAPABILITY={:?} NO_CAPTURE=true",rows.len(),first.name,tearing(&f)))
}
#[cfg(test)]mod tests{use super::*;#[test]fn validates_and_tearing_flag(){assert!(valid_id("00000000:deadbeef"));assert!(!valid_id("bad"));assert_eq!(tearing_flag(1,true).0,0);assert_eq!(tearing_flag(0,false).0,0);assert_eq!(tearing_flag(0,true).0,512);}}
