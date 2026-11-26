//! Experimental native composition swapchain for Horizon's own separate HWND.
//! Not a game overlay or a claim of exclusive fullscreen support.
use windows::{core::Interface,Win32::{Foundation::HWND,Graphics::{
 Direct3D11::ID3D11Device,
 DirectComposition::{DCompositionCreateDevice,IDCompositionDevice,IDCompositionTarget,IDCompositionVisual},
 Dxgi::{IDXGIFactory2,IDXGIOutput,IDXGIDevice,IDXGISwapChain1,IDXGISwapChain2,DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,DXGI_MATRIX_3X2_F},
 Dxgi::Common::DXGI_ALPHA_MODE_PREMULTIPLIED,
}}};
pub struct Output{
 pub chain:IDXGISwapChain1,
 device:IDCompositionDevice,target:IDCompositionTarget,_visual:IDCompositionVisual,
 size:(u32,u32),client:Option<(u32,u32)>,
}
impl Output{
 pub fn new(device:&ID3D11Device,factory:&IDXGIFactory2,hwnd:HWND,w:u32,h:u32)->Result<Self,String>{
  if hwnd.0.is_null()||w==0||h==0{return Err("Invalid own composition target".into());}
  let dxgi:IDXGIDevice=device.cast().map_err(|e|e.to_string())?;
  let dcomp:IDCompositionDevice=unsafe{DCompositionCreateDevice(&dxgi)}.map_err(|e|format!("DComposition device: {e}"))?;
  let mut desc=crate::native_present::swapchain_descriptor(w,h);
  desc.SwapEffect=DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL;desc.AlphaMode=DXGI_ALPHA_MODE_PREMULTIPLIED;
  let chain=unsafe{factory.CreateSwapChainForComposition(device,&desc,None::<&IDXGIOutput>)}.map_err(|e|format!("Composition swapchain: {e}"))?;
  let target=unsafe{dcomp.CreateTargetForHwnd(hwnd,true)}.map_err(|e|format!("Composition target: {e}"))?;
  let visual=unsafe{dcomp.CreateVisual()}.map_err(|e|format!("Composition visual: {e}"))?;
  unsafe{visual.SetContent(&chain)}.map_err(|e|format!("Composition content: {e}"))?;
  unsafe{target.SetRoot(&visual)}.map_err(|e|format!("Composition root: {e}"))?;
  unsafe{dcomp.Commit()}.map_err(|e|format!("Composition commit: {e}"))?;
  Ok(Self{chain,device:dcomp,target,_visual:visual,size:(w,h),client:None})
 }
 pub fn fit(&mut self,client:(u32,u32))->Result<(),String>{
  if client.0==0||client.1==0||self.client==Some(client){return Ok(());}
  let chain:IDXGISwapChain2=self.chain.cast().map_err(|e|e.to_string())?;
  let matrix=fit_matrix(self.size,client).ok_or("Invalid composition matrix")?;
  unsafe{chain.SetMatrixTransform(&matrix)}.map_err(|e|format!("Composition matrix: {e}"))?;
  self.client=Some(client);Ok(())
 }
}
impl Drop for Output{fn drop(&mut self){let _=unsafe{self.target.SetRoot(None::<&IDCompositionVisual>)};let _=unsafe{self.device.Commit()};}}

fn fit_matrix(size:(u32,u32),client:(u32,u32))->Option<DXGI_MATRIX_3X2_F>{
 if size.0==0||size.1==0||client.0==0||client.1==0{return None;}
 let (w,h)=(size.0 as f32,size.1 as f32);let(cw,ch)=(client.0 as f32,client.1 as f32);
 let scale=(cw/w).min(ch/h);Some(DXGI_MATRIX_3X2_F{_11:scale,_12:0.,_21:0.,_22:scale,_31:(cw-w*scale)*0.5,_32:(ch-h*scale)*0.5})
}
#[cfg(test)]mod tests{use super::*;#[test]fn composition_letterboxes_without_distortion(){let m=fit_matrix((1920,1080),(1000,1000)).unwrap();assert!((m._11-m._22).abs()<0.0001);assert!(m._31.abs()<0.001);assert!((m._32-218.75).abs()<0.001);assert!(fit_matrix((0,1080),(1000,1000)).is_none());}}
