//! Optional D3D11 hardware-fence completion for a single WGC frame copy.
//! Never releases source pixels before verified GPU completion.
use windows::{core::Interface,Win32::{Foundation::HANDLE,Graphics::Direct3D11::*}};
use std::ffi::c_void;
#[link(name="kernel32")]
unsafe extern "system"{
 fn CreateEventW(attributes:*const c_void,manual_reset:i32,initial_state:i32,name:*const u16)->*mut c_void;
 fn ResetEvent(event:*mut c_void)->i32;
 fn WaitForSingleObject(event:*mut c_void,millis:u32)->u32;
 fn CloseHandle(event:*mut c_void)->i32;
}
pub struct CopyFence{fence:ID3D11Fence,ctx4:ID3D11DeviceContext4,event:HANDLE,value:u64}
impl CopyFence{
 pub fn new(device:&ID3D11Device,context:&ID3D11DeviceContext)->Option<Self>{
  let dev:ID3D11Device5=device.cast().ok()?;
  let ctx4:ID3D11DeviceContext4=context.cast().ok()?;
  let mut fence=None;unsafe{dev.CreateFence(0,D3D11_FENCE_FLAG(0),&mut fence)}.ok()?;
  let fence=fence?;
  let event=HANDLE(unsafe{CreateEventW(std::ptr::null(),0,0,std::ptr::null())});
  if event.0.is_null(){return None;}
  Some(Self{fence,ctx4,event,value:0})
 }
 fn reached(&self)->bool{let n=unsafe{self.fence.GetCompletedValue()};n!=u64::MAX&&n>=self.value}
 pub fn complete(&mut self,context:&ID3D11DeviceContext)->bool{
  let Some(next)=self.value.checked_add(1)else{return false;};self.value=next;
  if unsafe{self.ctx4.Signal(&self.fence,next)}.is_err(){return false;}
  unsafe{context.Flush();}
  if self.reached(){return true;}
  if unsafe{ResetEvent(self.event.0)}==0{return false;}
  if unsafe{self.fence.SetEventOnCompletion(next,self.event)}.is_err(){return false;}
  let _=unsafe{WaitForSingleObject(self.event.0,100)};
  self.reached()
 }
}
impl Drop for CopyFence{fn drop(&mut self){unsafe{CloseHandle(self.event.0);}}}
#[cfg(test)]mod tests{use super::*;
 #[test]fn own_synthetic_copy_fence_or_explicit_fallback(){
  let g=crate::photo_lab::Gpu::new().unwrap();let im=image::RgbaImage::from_pixel(32,17,image::Rgba([35,66,122,255]));
  let src=g.upload(&im).unwrap();let dst=g.upload(&image::RgbaImage::new(32,17)).unwrap();
  let a:ID3D11Resource=src.cast().unwrap();let b:ID3D11Resource=dst.cast().unwrap();
  unsafe{g.ctx.CopyResource(&b,&a);}
  let mut f=CopyFence::new(&g.device,&g.ctx);
  let supported=f.as_mut().is_some_and(|f|f.complete(&g.ctx));
  if !supported{crate::native_filter::verify_gpu_completion(&g.device,&g.ctx).unwrap();}
  assert_eq!(g.read(&dst).unwrap(),im);
  eprintln!("COPY_FENCE_SUPPORTED={supported} PIXELS_EXACT=true");
 }
}
