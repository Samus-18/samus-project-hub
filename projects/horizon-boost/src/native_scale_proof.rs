//! Tests native GPU spatial modes using real user-provided gameplay image.
//! Reads bundled authentic scene, downsamples source to 640x360, upscales it
//! on GPU to 1280x720; no video game process touched.
use std::{ffi::c_void,path::Path,collections::HashSet};
use windows::{core::Interface,Win32::{
 Foundation::HMODULE,
 Graphics::{
  Direct3D::{D3D_DRIVER_TYPE_HARDWARE,D3D_FEATURE_LEVEL_11_0},
  Direct3D11::{
   D3D11CreateDevice,D3D11_SDK_VERSION,D3D11_CREATE_DEVICE_BGRA_SUPPORT,
   D3D11_TEXTURE2D_DESC,D3D11_SUBRESOURCE_DATA,D3D11_MAPPED_SUBRESOURCE,
   D3D11_USAGE_DEFAULT,D3D11_USAGE_STAGING,
   D3D11_CPU_ACCESS_READ,D3D11_BIND_SHADER_RESOURCE,D3D11_MAP_READ,
   ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11Resource,
   ID3D11Query,D3D11_QUERY_DESC,D3D11_QUERY_TIMESTAMP,D3D11_QUERY_TIMESTAMP_DISJOINT,
   D3D11_QUERY_DATA_TIMESTAMP_DISJOINT,
  },
  Dxgi::IDXGIAdapter,
  Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_SAMPLE_DESC}
 }
}};
const IW:u32=640; const IH:u32=360; const OW:u32=1280; const OH:u32=720;

/// Actual Radeon GPU timestamp query around repeated D3D11 compute dispatch.
/// Distinct from game-engine frametimes and CPU-wall stopwatch. The caller
/// already finished first-run shader JIT (first render before this function).
/// Returns None when driver marks GPU timestamp interval as disjoint.
fn benchmark_native_scaler_gpu(
 device:&ID3D11Device,ctx:&ID3D11DeviceContext,
 scaler:&mut crate::spatial::NativeSpatialScaler,
 input:&ID3D11Texture2D,mode:u8)->Result<Option<f64>,String>{
 let query=|kind|->Result<ID3D11Query,String>{
  let desc=D3D11_QUERY_DESC{Query:kind,MiscFlags:0};
  let mut q=None;
  unsafe{device.CreateQuery(&desc,Some(&mut q))}
    .map_err(|e|format!("GPU timestamp query creation: {e}"))?;
  q.ok_or("GPU timestamp query missing".into())
 };
 let disjoint=query(D3D11_QUERY_TIMESTAMP_DISJOINT)?;
 let begin=query(D3D11_QUERY_TIMESTAMP)?;
 let end=query(D3D11_QUERY_TIMESTAMP)?;
 // At 720p even a costly algorithm should finish quickly, and the test's
 // eight dispatches are bounded. No game HWND is involved.
 let samples=8u32;
 unsafe{ctx.Begin(&disjoint);ctx.End(&begin);}
 for _ in 0..samples{
  let _=scaler.render(device,ctx,input,mode)?;
 }
 unsafe{ctx.End(&end);ctx.End(&disjoint);}
 crate::native_filter::verify_gpu_completion(device,ctx)?;
 let read=|q:&ID3D11Query,dst:*mut std::ffi::c_void,len:u32|
     ->Result<(),String>{
  unsafe{ctx.GetData(q,Some(dst),len,0)}
    .map_err(|e|format!("GPU timestamp query GetData: {e}"))
 };
 let mut dis=D3D11_QUERY_DATA_TIMESTAMP_DISJOINT::default();
 let mut t0=0u64;let mut t1=0u64;
 read(&disjoint,&mut dis as *mut _ as *mut _,std::mem::size_of_val(&dis) as u32)?;
 read(&begin,&mut t0 as *mut _ as *mut _,8)?;
 read(&end,&mut t1 as *mut _ as *mut _,8)?;
 if dis.Disjoint.as_bool()||dis.Frequency==0||t1<t0{return Ok(None);}
 let ms=(t1-t0) as f64 * 1000.0 / dis.Frequency as f64/samples as f64;
 if !ms.is_finite()||ms<0.0||ms>250.0{return Ok(None);}
 Ok(Some(ms))
}

pub fn export_native_scaler_proof(destination:&Path)->Result<String,String>{
 std::fs::create_dir_all(destination).map_err(|e|e.to_string())?;
 let input=image::load_from_memory(include_bytes!("../assets/hero_gameplay_v10.png"))
  .map_err(|e|format!("Authentic real-game reference: {e}"))?
  .resize_to_fill(IW,IH,image::imageops::FilterType::Triangle)
  .to_rgba8();
 let mut device:Option<ID3D11Device>=None;
 let mut context:Option<ID3D11DeviceContext>=None;
 unsafe{D3D11CreateDevice(None::<&IDXGIAdapter>,D3D_DRIVER_TYPE_HARDWARE,
   HMODULE::default(),D3D11_CREATE_DEVICE_BGRA_SUPPORT,
   Some(&[D3D_FEATURE_LEVEL_11_0]),D3D11_SDK_VERSION,
   Some(&mut device),None,Some(&mut context))}
   .map_err(|e|format!("Scaler GPU: {e}"))?;
 let device=device.ok_or("No scaler proof GPU")?;
 let ctx=context.ok_or("No scaler proof GPU context")?;
 let desc=D3D11_TEXTURE2D_DESC{
  Width:IW,Height:IH,MipLevels:1,ArraySize:1,
  Format:DXGI_FORMAT_R8G8B8A8_UNORM,
  SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
  Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,
  CPUAccessFlags:0,MiscFlags:0
 };
 let init=D3D11_SUBRESOURCE_DATA{
  pSysMem:input.as_raw().as_ptr() as *const c_void,
  SysMemPitch:IW*4,SysMemSlicePitch:IW*IH*4
 };
 let mut source=None;
 unsafe{device.CreateTexture2D(&desc,Some(&init),Some(&mut source))}
   .map_err(|e|format!("Input GPU texture: {e}"))?;
 let source=source.ok_or("Scaler source missing")?;
 let staging_desc=D3D11_TEXTURE2D_DESC{
  Width:OW,Height:OH,
  Usage:D3D11_USAGE_STAGING,BindFlags:0,
  CPUAccessFlags:D3D11_CPU_ACCESS_READ.0 as u32,..desc
 };
 let mut staging=None;
 unsafe{device.CreateTexture2D(&staging_desc,None,Some(&mut staging))}
   .map_err(|e|format!("Scaler staging texture: {e}"))?;
 let staging:ID3D11Texture2D=staging.ok_or("No scaler staging")?;
 let staging_resource:ID3D11Resource=staging.cast().map_err(|e|e.to_string())?;
 let mut scaler=crate::spatial::NativeSpatialScaler::new(&device,OW,OH)?;
 let mut hashes=HashSet::new();
 let mut log=String::new();
 // Integer fit is verified separately at odd non-integer output; at 2x it is intentionally identical to nearest.
 for mode in 1..=11{
   let result=scaler.render(&device,&ctx,&source,mode)?;
   let resource:ID3D11Resource=result.cast().map_err(|e|e.to_string())?;
   unsafe{ctx.CopyResource(&staging_resource,&resource);}
   crate::native_filter::verify_gpu_completion(&device,&ctx)?;
   let mut mapped=D3D11_MAPPED_SUBRESOURCE::default();
   unsafe{ctx.Map(&staging_resource,0,D3D11_MAP_READ,0,Some(&mut mapped))}
     .map_err(|e|format!("Scaler GPU readback: {e}"))?;
   let bytes=(||->Result<Vec<u8>,String>{
       if mapped.pData.is_null(){return Err("Empty scaler GPU output".into());}
       let mut dest=vec![0u8;(OW*OH*4) as usize];
       for y in 0..OH as usize{
           let row=unsafe{std::slice::from_raw_parts(
             (mapped.pData as *const u8).add(y*mapped.RowPitch as usize),
             OW as usize*4)};
           dest[y*OW as usize*4..(y+1)*OW as usize*4].copy_from_slice(row);
       }
       Ok(dest)
   })();
   unsafe{ctx.Unmap(&staging_resource,0);}
   let bytes=bytes?;
   let mut hash=0xcbf29ce484222325u64;
   let mut green=0usize;
   for px in bytes.chunks_exact(4).step_by(5){
      // Do not accept output that turned an ordinary gray/blue reference
      // into the green-screen regression seen in the user's test video.
      if px[1]>230&&px[1]>px[0].saturating_add(80)&&
         px[1]>px[2].saturating_add(80){green+=1;}
      for c in &px[0..3]{hash=(hash^*c as u64).wrapping_mul(0x100000001b3);}
   }
   if green > bytes.len()/4/5/3{
       return Err(format!("Green-screen regression for native scaler mode {mode}"));
   }
   if !hashes.insert(hash){
       return Err(format!("Duplicate GPU-scaled image for mode {mode}"));
   }
   image::RgbaImage::from_raw(OW,OH,bytes).ok_or("Bad scaler RGBA buffer")?
       .save(destination.join(format!("v23-native-upscale-mode-{mode}.png")))
       .map_err(|e|format!("Save actual GPU scaler image: {e}"))?;
   let timing=benchmark_native_scaler_gpu(&device,&ctx,&mut scaler,&source,mode)?;
   let perf=timing.map(|ms|format!("{ms:.4}")).unwrap_or_else(||"unavailable".to_string());
   log.push_str(&format!(
       "NATIVE_UPSCALE_GPU_MODE={mode} HASH=0x{hash:016x} GREEN_PIXELS={green}         HORIZON_OWN_GPU_MS_PER_DISPATCH={perf} SAMPLES=8 GAME_FPS_NOT_MEASURED=true\n"));
 }
 Ok(format!("NATIVE_SPATIAL_GPU=PASS CAPTURE=640x360 OUTPUT=1280x720 UNIQUE_MODES={} NO_MAGPIE=true\n{}",
    hashes.len(),log))
}
