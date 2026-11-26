//! Explicit, foreground-bound DXGI Desktop Duplication fallback.
//! Native GPU-only capture from a selected monitor-filling game. Never uses
//! CPU readback, game DLL injection, hidden desktop capture or process hooks.
//! If selected game loses focus, caller must hide output and stop acquiring.
use windows::{
 core::Interface,
 Win32::{
  Foundation::HMODULE,
  Graphics::{
   Direct3D::{D3D_DRIVER_TYPE_UNKNOWN,D3D_FEATURE_LEVEL_11_0},
   Direct3D11::{
    D3D11CreateDevice,D3D11_CREATE_DEVICE_BGRA_SUPPORT,D3D11_SDK_VERSION,
    D3D11_BIND_SHADER_RESOURCE,D3D11_USAGE_DEFAULT,D3D11_TEXTURE2D_DESC,
    ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11ShaderResourceView,ID3D11Resource,
   },
   Dxgi::{
    CreateDXGIFactory1,IDXGIFactory1,IDXGIAdapter,IDXGIOutput1,IDXGIOutput6,
    IDXGIOutputDuplication,IDXGIResource,DXGI_OUTDUPL_FRAME_INFO,DXGI_ERROR_WAIT_TIMEOUT,
   },
  },
 },
};
pub fn covers(game:[i32;4],monitor:[i32;4])->bool{
 let (w,h)=(monitor[2]-monitor[0],monitor[3]-monitor[1]);
 if w<160||h<90||w>8192||h>8192||game[2]<=game[0]||game[3]<=game[1]{return false;}
 game.iter().zip(monitor).all(|(a,b)|i64::from(*a).saturating_sub(i64::from(b)).abs()<=16)
}
pub struct DupSource{
 pub device:ID3D11Device,
 pub context:ID3D11DeviceContext,
 duplication:IDXGIOutputDuplication,
 input:ID3D11Texture2D,
 input_srv:ID3D11ShaderResourceView,
 pipeline:crate::native_pipeline::NativeGpuPipeline,
 pub width:u32,pub height:u32,pub monitor:[i32;4],
}
impl DupSource{
 pub fn new(game_rect:[i32;4])->Result<Self,String>{
  let factory:IDXGIFactory1=unsafe{CreateDXGIFactory1()}.map_err(|e|format!("DXGI fallback factory: {e}"))?;
  let mut last="No suitable fullscreen desktop duplication monitor".to_string();
  for ai in 0..16{
   let Ok(adapter)=(unsafe{factory.EnumAdapters1(ai)})else{break;};
   if unsafe{adapter.GetDesc1()}.is_ok_and(|d|d.VendorId==0x1414){continue;}
   let raw:IDXGIAdapter=adapter.cast().map_err(|e|e.to_string())?;
   for oi in 0..16{
    let Ok(output)=(unsafe{adapter.EnumOutputs(oi)})else{break;};
    let Ok(advanced)=output.cast::<IDXGIOutput6>()else{continue;};
    let Ok(desc)=(unsafe{advanced.GetDesc1()})else{continue;};
    let mon=[desc.DesktopCoordinates.left,desc.DesktopCoordinates.top,
             desc.DesktopCoordinates.right,desc.DesktopCoordinates.bottom];
    if !covers(game_rect,mon){continue;}
    let mut dev=None;let mut ctx=None;
    unsafe{D3D11CreateDevice(&raw,D3D_DRIVER_TYPE_UNKNOWN,HMODULE::default(),
        D3D11_CREATE_DEVICE_BGRA_SUPPORT,Some(&[D3D_FEATURE_LEVEL_11_0]),
        D3D11_SDK_VERSION,Some(&mut dev),None,Some(&mut ctx))}
        .map_err(|e|format!("DXGI fallback hardware device: {e}"))?;
    let device=dev.ok_or("DXGI fallback D3D11 device missing")?;
    let context=ctx.ok_or("DXGI fallback D3D11 context missing")?;
    let out1:IDXGIOutput1=output.cast().map_err(|e|e.to_string())?;
    let duplication=match unsafe{out1.DuplicateOutput(&device)}{
     Ok(d)=>d,Err(e)=>{last=format!("DXGI DuplicateOutput unavailable: {e}");continue;}
    };
    let geometry=unsafe{duplication.GetDesc()};
    let w=geometry.ModeDesc.Width;let h=geometry.ModeDesc.Height;
    if w<160||h<90||w>8192||h>8192{return Err("Invalid duplicated desktop size".into());}
    let desc=D3D11_TEXTURE2D_DESC{
     Width:w,Height:h,MipLevels:1,ArraySize:1,Format:geometry.ModeDesc.Format,
     SampleDesc:windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC{Count:1,Quality:0},
     Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,
     CPUAccessFlags:0,MiscFlags:0,
    };
    let mut texture=None;
    unsafe{device.CreateTexture2D(&desc,None,Some(&mut texture))}
      .map_err(|e|format!("DXGI fallback GPU scratch texture: {e}"))?;
    let input=texture.ok_or("DXGI fallback scratch allocation missing")?;
    let mut view=None;
    unsafe{device.CreateShaderResourceView(&input,None,Some(&mut view))}
      .map_err(|e|format!("DXGI fallback scratch SRV: {e}"))?;
    let input_srv=view.ok_or("DXGI fallback scratch view missing")?;
    let pipeline=crate::native_pipeline::NativeGpuPipeline::new(&device,w,h)?;
    return Ok(Self{device,context,duplication,input,input_srv,pipeline,width:w,height:h,monitor:mon});
   }
  }
  Err(last)
 }
 pub fn next(&self,stages:&[usize],light:&crate::lighting::LightTuning)->Result<Option<ID3D11Texture2D>,String>{
  let mut info=DXGI_OUTDUPL_FRAME_INFO::default();
  let mut resource:Option<IDXGIResource>=None;
  match unsafe{self.duplication.AcquireNextFrame(8,&mut info,&mut resource)}{
   Err(e) if e.code()==DXGI_ERROR_WAIT_TIMEOUT=>return Ok(None),
   Err(e)=>return Err(format!("DXGI_DESKTOP_DUPLICATION_LOST: {e}")),
   Ok(())=>{},
  }
  // A successful AcquireNextFrame must always ReleaseFrame, even if copying or GPU
  // compilation fails. The duplicated resource is never mapped to host memory.
  let copied=(||->Result<(),String>{
   let texture:ID3D11Texture2D=resource.ok_or("Duplicated desktop returned no texture")?
     .cast().map_err(|e|e.to_string())?;
   let mut desc=D3D11_TEXTURE2D_DESC::default();unsafe{texture.GetDesc(&mut desc);}
   if (desc.Width,desc.Height)!=(self.width,self.height){return Err("DXGI output size changed, rebuild capture".into());}
   let dst:ID3D11Resource=self.input.cast().map_err(|e|e.to_string())?;
   let src:ID3D11Resource=texture.cast().map_err(|e|e.to_string())?;
   unsafe{self.context.CopyResource(&dst,&src);self.context.Flush();}
   crate::native_filter::verify_gpu_completion(&self.device,&self.context)
  })();
  let released=unsafe{self.duplication.ReleaseFrame()};
  if let Err(e)=released{return Err(format!("DXGI ReleaseFrame failed: {e}"));}
  copied?;
  self.pipeline.dispatch_live(&self.context,&self.input,&self.input_srv,stages,light).map(Some)
 }
 pub fn effective_stages(&self)->usize{self.pipeline.live_active_count()}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn only_exact_selected_monitor_filling_rectangles_are_allowed(){
  let screen=[-1920,0,0,1080];
  assert!(covers([-1928,-8,8,1088],screen));
  assert!(covers([-1920,0,0,1080],screen));
  assert!(!covers([-1500,300,-50,900],screen));
  assert!(!covers([-1920,0,300,1080],screen));
  assert!(!covers([0,0,0,0],screen));
 }
}
