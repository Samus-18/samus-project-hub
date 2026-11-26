//! Explicit synthetic GPU microbenchmark. No game/window capture.
use windows::{Win32::{Foundation::HMODULE,Graphics::{
 Direct3D::{D3D_DRIVER_TYPE_HARDWARE,D3D_FEATURE_LEVEL_11_0},
 Direct3D11::{D3D11CreateDevice,D3D11_SDK_VERSION,D3D11_CREATE_DEVICE_BGRA_SUPPORT,
 D3D11_TEXTURE2D_DESC,D3D11_SUBRESOURCE_DATA,D3D11_USAGE_DEFAULT,D3D11_BIND_SHADER_RESOURCE,
 ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11Query,D3D11_QUERY_DESC,
 D3D11_QUERY_TIMESTAMP,D3D11_QUERY_TIMESTAMP_DISJOINT,D3D11_QUERY_DATA_TIMESTAMP_DISJOINT},
 Dxgi::{IDXGIAdapter,Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_SAMPLE_DESC}}}}};
use serde::Serialize;
#[derive(Clone,Debug,Serialize)]
pub struct Row{pub label:String,pub width:u32,pub height:u32,pub gpu_ms:Option<f64>}
pub fn measure(device:&ID3D11Device,ctx:&ID3D11DeviceContext,mut run:impl FnMut()->Result<(),String>)->Result<Option<f64>,String>{
 let query=|kind|->Result<ID3D11Query,String>{
  let mut q=None;unsafe{device.CreateQuery(&D3D11_QUERY_DESC{Query:kind,MiscFlags:0},Some(&mut q))}.map_err(|e|e.to_string())?;
  q.ok_or("Timestamp query unavailable".into())
 };
 run()?;crate::native_filter::verify_gpu_completion(device,ctx)?;
 let disjoint=query(D3D11_QUERY_TIMESTAMP_DISJOINT)?;let a=query(D3D11_QUERY_TIMESTAMP)?;let b=query(D3D11_QUERY_TIMESTAMP)?;
 const N:u32=8;
 unsafe{ctx.Begin(&disjoint);ctx.End(&a);}
 let result=(||{for _ in 0..N{run()?;}Ok::<(),String>(())})();
 unsafe{ctx.End(&b);ctx.End(&disjoint);}
 result?;
 crate::native_filter::verify_gpu_completion(device,ctx)?;
 let read=|q:&ID3D11Query,p:*mut std::ffi::c_void,n:u32|unsafe{ctx.GetData(q,Some(p),n,0)}.map_err(|e|e.to_string());
 let mut d=D3D11_QUERY_DATA_TIMESTAMP_DISJOINT::default();let(mut t0,mut t1)=(0u64,0u64);
 read(&disjoint,&mut d as *mut _ as *mut _,std::mem::size_of_val(&d) as u32)?;
 read(&a,&mut t0 as *mut _ as *mut _,8)?;read(&b,&mut t1 as *mut _ as *mut _,8)?;
 if d.Disjoint.as_bool()||d.Frequency==0||t1<=t0{return Ok(None);}
 let ms=(t1-t0) as f64*1000./d.Frequency as f64/N as f64;
 Ok((ms.is_finite()&&ms>0.&&ms<500.).then_some(ms))
}
pub fn run(stack:&[usize])->Result<Vec<Row>,String>{
 crate::native_stack::validate(stack)?;
 let(mut device,mut ctx)=(None,None);
 unsafe{D3D11CreateDevice(None::<&IDXGIAdapter>,D3D_DRIVER_TYPE_HARDWARE,HMODULE::default(),
  D3D11_CREATE_DEVICE_BGRA_SUPPORT,Some(&[D3D_FEATURE_LEVEL_11_0]),D3D11_SDK_VERSION,
  Some(&mut device),None,Some(&mut ctx))}.map_err(|e|e.to_string())?;
 let device:ID3D11Device=device.ok_or("No GPU")?;let ctx:ID3D11DeviceContext=ctx.ok_or("No context")?;
 let mut rows=Vec::new();
 for (w,h) in [(1280u32,720u32),(1920,1080),(2560,1440)]{
  let mut data=vec![0u8;(w*h*4) as usize];
  for (i,p) in data.chunks_exact_mut(4).enumerate(){
   let x=i as u32%w;let y=i as u32/w;
   p.copy_from_slice(&[(x%256) as u8,(y%256) as u8,((x^y)%256) as u8,255]);
  }
  let desc=D3D11_TEXTURE2D_DESC{Width:w,Height:h,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R8G8B8A8_UNORM,
   SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,CPUAccessFlags:0,MiscFlags:0};
  let init=D3D11_SUBRESOURCE_DATA{pSysMem:data.as_ptr().cast(),SysMemPitch:w*4,SysMemSlicePitch:w*h*4};
  let mut source=None;unsafe{device.CreateTexture2D(&desc,Some(&init),Some(&mut source))}.map_err(|e|e.to_string())?;
  let source:ID3D11Texture2D=source.ok_or("Synthetic GPU input missing")?;
  let pipe=crate::native_pipeline::NativeGpuPipeline::new(&device,w,h)?;
  if w==1280{
   for (i,(name,_)) in crate::native_filter::PROGRAMS.iter().enumerate(){
    if !crate::native_stack::valid_stage(i){continue;}
    let value=measure(&device,&ctx,||pipe.dispatch(&device,&ctx,&source,&[i]).map(|_|()))?;
    rows.push(Row{label:(*name).into(),width:w,height:h,gpu_ms:value});
   }
  }
  let value=measure(&device,&ctx,||pipe.dispatch(&device,&ctx,&source,stack).map(|_|()))?;
  rows.push(Row{label:format!("Selected stack ({} passes)",stack.len()),width:w,height:h,gpu_ms:value});
 }
 Ok(rows)
}

pub fn run_lighting(stack:&[usize],tuning:&crate::lighting::LightTuning)->Result<Vec<Row>,String>{
 crate::native_stack::validate(stack)?;
 let g=crate::photo_lab::Gpu::new()?;let mut rows=Vec::new();
 let mut base=tuning.clone();base.enabled=false;base.grade.enabled=false;base.lut.name=None;base.cinema.enabled=false;base.recipe.enabled=false;base.precision.enabled=false;base.exposure.enabled=false;
 for(w,h)in[(1280,720),(1920,1080),(2560,1440)]{
  let im=image::RgbaImage::from_fn(w,h,|x,y|image::Rgba([(x%256)as u8,(y%256)as u8,((x^y)%256)as u8,255]));let tex=g.upload(&im)?;
  let mut srv=None;unsafe{g.device.CreateShaderResourceView(&tex,None,Some(&mut srv))}.map_err(|e|e.to_string())?;let srv=srv.ok_or("Lighting benchmark SRV")?;
  let p=crate::native_pipeline::NativeGpuPipeline::new(&g.device,w,h)?;
  for(label,t)in[("Base stack",&base),("Current full lighting",&tuning)]{
   let ms=measure(&g.device,&g.ctx,||p.dispatch_with_cached_srv_tuned(&g.ctx,&tex,&srv,stack,t).map(|_|()))?;
   let note=if p.optional_warning()!=0{" (fallback)"}else{""};rows.push(Row{label:format!("{label}{note}"),width:w,height:h,gpu_ms:ms});
  }
 }
 Ok(rows)
}

// Compare the exact ordered stack and tuning with only fusion toggled.
// Alternate execution order, report medians, never extrapolate to game FPS.
pub fn fusion_comparison(stack:&[usize],tuning:&crate::lighting::LightTuning)->Result<Vec<Row>,String>{
 crate::native_stack::validate(stack)?;
 let g=crate::photo_lab::Gpu::new()?;let mut rows=Vec::new();
 let mut off=tuning.clone();off.reconstruction.fused_color=false;
 let mut on=tuning.clone();on.reconstruction.fused_color=true;
 for(w,h)in[(1280u32,720u32),(1920,1080)]{
  let im=image::RgbaImage::from_fn(w,h,|x,y|image::Rgba([((x*17+y*11)%256)as u8,((x*5+y*31)%256)as u8,((x*31+y*7)%256)as u8,255]));
  let tex=g.upload(&im)?;let mut srv=None;unsafe{g.device.CreateShaderResourceView(&tex,None,Some(&mut srv))}.map_err(|e|e.to_string())?;let srv=srv.ok_or("Fusion bench SRV")?;
  let p=crate::native_pipeline::NativeGpuPipeline::new(&g.device,w,h)?;p.enable_fusion();
  let mut samples:[Vec<f64>;2]=[Vec::new(),Vec::new()];let mut saved=0;
  for order in [[0,1],[1,0],[0,1]]{
   for i in order{let t=if i==0{&off}else{&on};let value=measure(&g.device,&g.ctx,||p.dispatch_with_cached_srv_tuned(&g.ctx,&tex,&srv,stack,t).map(|_|()))?;if let Some(ms)=value{samples[i].push(ms);}if i==1{saved=p.fused_saved();}}
  }
  for (i,label)in ["Separate color passes".to_string(),format!("Fusion enabled ({saved} dispatches saved)")].into_iter().enumerate(){
   samples[i].sort_by(f64::total_cmp);let value=(samples[i].len()==3).then(||samples[i][1]);
   rows.push(Row{label,width:w,height:h,gpu_ms:value});
  }
 }Ok(rows)
}
#[cfg(test)]mod fusion_bench_tests{
 use super::*;
 #[test]fn real_gpu_fusion_timestamps_are_measured(){
  let rows=fusion_comparison(&[5,5,4,1,11,0,7,8],&crate::lighting::LightTuning::default()).unwrap();
  assert_eq!(rows.len(),4);
  for r in &rows{assert!(r.gpu_ms.is_some_and(|ms|ms>0.));eprintln!("FUSION_AB {}x{} {} {:?}",r.width,r.height,r.label,r.gpu_ms);}
  assert!(rows[1].label.contains("7 dispatches saved"));assert!(rows[3].label.contains("7 dispatches saved"));
 }
}
