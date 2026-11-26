//! Waitable object for Horizon's own swap chain only.
use windows::{core::Interface,Win32::{Foundation::HANDLE,Graphics::Dxgi::{IDXGISwapChain1,IDXGISwapChain2}}};
#[link(name="kernel32")]
unsafe extern "system"{
 fn WaitForSingleObject(h:*mut std::ffi::c_void,ms:u32)->u32;
 fn CloseHandle(h:*mut std::ffi::c_void)->i32;
}
pub struct FramePermit(HANDLE);
impl FramePermit{
 pub fn new(chain:&IDXGISwapChain1)->Result<Self,String>{
  let chain:IDXGISwapChain2=chain.cast().map_err(|e|e.to_string())?;
  unsafe{chain.SetMaximumFrameLatency(1)}.map_err(|e|e.to_string())?;
  let handle=unsafe{chain.GetFrameLatencyWaitableObject()};
  if handle.0.is_null(){return Err("DXGI returned no frame latency waitable object".into());}
  Ok(Self(handle))
 }
 pub fn wait(&self)->Result<bool,String>{
  match unsafe{WaitForSingleObject(self.0.0,50)}{
   0=>Ok(true),258=>Ok(false),n=>Err(format!("DXGI wait failed ({n})"))
  }
 }
}
impl Drop for FramePermit{fn drop(&mut self){unsafe{CloseHandle(self.0.0);}}}
