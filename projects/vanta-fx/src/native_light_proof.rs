//! Hardware verification for adjustable native Light Studio. Never reads a
//! private screen/game window: inputs are already bundled permissioned images.
use std::{ffi::c_void,path::Path};
use windows::{
 core::Interface,
 Win32::{
  Foundation::HMODULE,
  Graphics::{
   Direct3D::{D3D_DRIVER_TYPE_HARDWARE,D3D_FEATURE_LEVEL_11_0},
   Direct3D11::{
    D3D11CreateDevice,D3D11_SDK_VERSION,D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_TEXTURE2D_DESC,D3D11_SUBRESOURCE_DATA,D3D11_MAPPED_SUBRESOURCE,
    D3D11_USAGE_DEFAULT,D3D11_USAGE_STAGING,
    D3D11_CPU_ACCESS_READ,D3D11_BIND_SHADER_RESOURCE,
    D3D11_MAP_READ,ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11Resource,
   },
   Dxgi::IDXGIAdapter,
   Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_SAMPLE_DESC},
  },
 },
};
const W:u32=512;const H:u32=288;
fn image_input(plain:&[u8])->Result<image::RgbaImage,String>{
 Ok(image::load_from_memory(plain).map_err(|e|e.to_string())?
    .resize_to_fill(W,H,image::imageops::FilterType::Lanczos3).to_rgba8())
}
fn weights(t:[f32;7])->crate::lighting::LightTuning{
 crate::lighting::LightTuning{
  enabled:true,shadow:t[0],glow:t[1],bounce:t[2],detail:t[3],
  film:t[4],warmth:t[5],mix:t[6],grade:crate::color_grade::Grade::default(),layers:Default::default(),motion:Default::default(),lut:Default::default(),exposure:Default::default(),reconstruction:Default::default(),precision:Default::default(),recipe:Default::default(),cinema:Default::default(),bypass_look:false,
 }
}
pub fn export_lighting_proof(output:&Path)->Result<String,String>{
 std::fs::create_dir_all(output).map_err(|e|e.to_string())?;
 let shader_set=crate::native_presets::PRESETS.get(64)
    .ok_or("Natural Daylight native preset missing")?.stages;
 let variants=[
  ("baseline",crate::lighting::LightTuning::default()),
  ("zero",crate::lighting::LightTuning::zero()),
  ("natural",weights([0.24,0.25,0.21,0.16,0.24,0.10,0.74])),
  ("cinematic",weights([0.56,0.32,0.17,0.20,0.56,0.23,0.85])),
  ("shadow",weights([0.76,0.16,0.32,0.33,0.21,0.08,0.87])),
  ("radiance",weights([0.20,0.74,0.56,0.10,0.29,0.37,0.90])),
 ];
 let mut device:Option<ID3D11Device>=None;
 let mut context:Option<ID3D11DeviceContext>=None;
 unsafe{D3D11CreateDevice(None::<&IDXGIAdapter>,D3D_DRIVER_TYPE_HARDWARE,
   HMODULE::default(),D3D11_CREATE_DEVICE_BGRA_SUPPORT,
   Some(&[D3D_FEATURE_LEVEL_11_0]),D3D11_SDK_VERSION,
   Some(&mut device),None,Some(&mut context))}
   .map_err(|e|format!("Lighting proof GPU: {e}"))?;
 let device=device.ok_or("No lighting proof device")?;
 let ctx=context.ok_or("No lighting proof immediate context")?;
 let desc=D3D11_TEXTURE2D_DESC{
  Width:W,Height:H,MipLevels:1,ArraySize:1,
  Format:DXGI_FORMAT_R8G8B8A8_UNORM,
  SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
  Usage:D3D11_USAGE_DEFAULT,
  BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,
  CPUAccessFlags:0,MiscFlags:0,
 };
 let stage_desc=D3D11_TEXTURE2D_DESC{
  Usage:D3D11_USAGE_STAGING,BindFlags:0,
  CPUAccessFlags:D3D11_CPU_ACCESS_READ.0 as u32,..desc
 };
 let mut staging=None;
 unsafe{device.CreateTexture2D(&stage_desc,None,Some(&mut staging))}
   .map_err(|e|format!("Lighting staging create: {e}"))?;
 let staging:ID3D11Texture2D=staging.ok_or("No lighting stage texture")?;
 let staging_resource:ID3D11Resource=staging.cast().map_err(|e|e.to_string())?;
 let pipeline=crate::native_pipeline::NativeGpuPipeline::new(&device,W,H)?;
 let mut messages=Vec::new();
 for (scene,data) in [
  ("space",include_bytes!("../assets/preview_photo_cc0.jpg").as_slice()),
  ("gameplay",include_bytes!("../assets/hero_gameplay_v10.png").as_slice())
 ] {
  let input=image_input(data)?;
  let bytes=input.as_raw();
  let init=D3D11_SUBRESOURCE_DATA{
   pSysMem:bytes.as_ptr() as *const c_void,SysMemPitch:W*4,SysMemSlicePitch:W*H*4
  };
  let mut original=None;
  unsafe{device.CreateTexture2D(&desc,Some(&init),Some(&mut original))}
   .map_err(|e|format!("Lighting proof source GPU: {e}"))?;
  let original:ID3D11Texture2D=original.ok_or("Lighting source texture empty")?;
  let mut srv=None;
  unsafe{device.CreateShaderResourceView(&original,None,Some(&mut srv))}
   .map_err(|e|format!("Lighting source SRV: {e}"))?;
  let srv=srv.ok_or("Lighting source SRV missing")?;
  let mut hashes=Vec::new();
  for (label,params) in &variants{
   let result=pipeline.dispatch_with_cached_srv_tuned(&ctx,&original,&srv,
       shader_set,params)?;
   let resource:ID3D11Resource=result.cast().map_err(|e|e.to_string())?;
   unsafe{ctx.CopyResource(&staging_resource,&resource);}
   crate::native_filter::verify_gpu_completion(&device,&ctx)?;
   let mut mapped=D3D11_MAPPED_SUBRESOURCE::default();
   unsafe{ctx.Map(&staging_resource,0,D3D11_MAP_READ,0,Some(&mut mapped))}
     .map_err(|e|format!("Lighting stage map: {e}"))?;
   let copy=(||->Result<Vec<u8>,String>{
    if mapped.pData.is_null(){return Err("Empty map for light proof".into());}
    let mut out=vec![0u8;(W*H*4) as usize];
    for row in 0..H as usize{
     let line=unsafe{std::slice::from_raw_parts(
        (mapped.pData as *const u8).add(row*mapped.RowPitch as usize),W as usize*4)};
     out[row*W as usize*4..(row+1)*W as usize*4].copy_from_slice(line);
    }
    Ok(out)
   })();
   unsafe{ctx.Unmap(&staging_resource,0);}
   let rgba=copy?;
   let shot=image::RgbaImage::from_raw(W,H,rgba.clone())
      .ok_or("Bad D3D11 tuned GPU image")?;
   shot.save(output.join(format!("hb21-{scene}-{label}.png")))
      .map_err(|e|e.to_string())?;
   let mut h=0xcbf29ce484222325u64;
   for byte in rgba.iter().step_by(3){
    h=(h^*byte as u64).wrapping_mul(0x100000001b3);
   }
   messages.push(format!("LIGHT_GPU={scene} MODE={label} HASH=0x{h:016x}"));
   hashes.push(h);
  }
  if hashes[0]!=hashes[1]{
   return Err(format!("Lighting zero-intensity not exact for {scene}"));
  }
  if hashes[2..].iter().collect::<std::collections::HashSet<_>>().len()!=4||
      hashes[2..].contains(&hashes[0]){
   return Err(format!("Adjustable lighting shaders not visually distinct in {scene}"));
  }
 }
 Ok(format!("NATIVE_LIGHT_STUDIO_GPU=PASS REAL_GPU_PROFILES=12 NEUTRAL_EXACT=true\n{}",
    messages.join("\n")))
}
