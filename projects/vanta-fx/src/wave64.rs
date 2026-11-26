//! Real Shader Model 6 HLSL WaveOps kernel executed on the RX 580's native
//! D3D12 COMPUTE queue. Compiled with Microsoft DXC 1.9.2609 at development
//! time; only original 2.9KB DXIL + 92B root-signature are embedded at runtime.
//! NO game hooking, no fullscreen writes, no external compiler at runtime.
use std::{mem::ManuallyDrop,time::{Instant,Duration},thread};
use windows::{core::Interface,Win32::Graphics::{
 Direct3D::D3D_FEATURE_LEVEL_11_0,
 Direct3D12::*,
 Dxgi::Common::{DXGI_FORMAT_UNKNOWN,DXGI_SAMPLE_DESC},
}};
const SHADER:&[u8]=include_bytes!("../shaders/native/HB_WaveLaneCount.dxil");
const ROOTSIG:&[u8]=include_bytes!("../shaders/native/HB_WaveLaneCount.rootsig");
#[derive(Debug,Clone)]
pub struct Report{pub wave_size:u32,pub lanes_checked:usize,pub subgroup_sum:u32,pub hardware_gpu:bool}
fn heap(kind:D3D12_HEAP_TYPE)->D3D12_HEAP_PROPERTIES{
 D3D12_HEAP_PROPERTIES{Type:kind,CPUPageProperty:Default::default(),
  MemoryPoolPreference:Default::default(),CreationNodeMask:1,VisibleNodeMask:1}
}
fn resource_desc(flags:D3D12_RESOURCE_FLAGS)->D3D12_RESOURCE_DESC{
 D3D12_RESOURCE_DESC{Dimension:D3D12_RESOURCE_DIMENSION_BUFFER,Alignment:0,
  Width:256,Height:1,DepthOrArraySize:1,MipLevels:1,Format:DXGI_FORMAT_UNKNOWN,
  SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Layout:D3D12_TEXTURE_LAYOUT_ROW_MAJOR,Flags:flags}
}
pub fn execute()->Result<Report,String>{
 let mut device:Option<ID3D12Device>=None;
 unsafe{D3D12CreateDevice(None::<&windows::core::IUnknown>,D3D_FEATURE_LEVEL_11_0,&mut device)}
  .map_err(|e|format!("D3D12 hardware device: {e}"))?;
 let dev=device.ok_or("No D3D12 device")?;
 let mut options=D3D12_FEATURE_DATA_D3D12_OPTIONS1::default();
 unsafe{dev.CheckFeatureSupport(D3D12_FEATURE_D3D12_OPTIONS1,
   (&mut options as *mut D3D12_FEATURE_DATA_D3D12_OPTIONS1).cast(),
   std::mem::size_of::<D3D12_FEATURE_DATA_D3D12_OPTIONS1>() as u32)}
  .map_err(|e|format!("Driver does not expose WaveOps options: {e}"))?;
 if !options.WaveOps.as_bool(){return Err("RX580 driver reports no Shader Model 6 WaveOps".into());}
 let root:ID3D12RootSignature=unsafe{dev.CreateRootSignature(0,ROOTSIG)}
  .map_err(|e|format!("DXIL root signature unsupported: {e}"))?;
 let pso_desc=D3D12_COMPUTE_PIPELINE_STATE_DESC{
  pRootSignature:ManuallyDrop::new(Some(root.clone())),
  CS:D3D12_SHADER_BYTECODE{pShaderBytecode:SHADER.as_ptr().cast(),BytecodeLength:SHADER.len()},
  NodeMask:0,CachedPSO:Default::default(),Flags:D3D12_PIPELINE_STATE_FLAG_NONE,
 };
 let pso:ID3D12PipelineState=unsafe{dev.CreateComputePipelineState(&pso_desc)}
  .map_err(|e|format!("DXC SM6 WaveOps shader rejected by actual driver: {e}"))?;
 let mut desc=D3D12_COMMAND_QUEUE_DESC{
  Type:D3D12_COMMAND_LIST_TYPE_COMPUTE,
  Priority:D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0,
  Flags:D3D12_COMMAND_QUEUE_FLAG_NONE,NodeMask:0
 };
 let queue:ID3D12CommandQueue=unsafe{dev.CreateCommandQueue(&desc)}
  .map_err(|e|format!("D3D12 GPU compute queue: {e}"))?;
 let memory=resource_desc(D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS);
 let mut gpu_output=None;
 unsafe{dev.CreateCommittedResource(&heap(D3D12_HEAP_TYPE_DEFAULT),D3D12_HEAP_FLAG_NONE,
   &memory,D3D12_RESOURCE_STATE_UNORDERED_ACCESS,None,&mut gpu_output)}
  .map_err(|e|format!("Wave64 GPU UAV memory: {e}"))?;
 let gpu_output:ID3D12Resource=gpu_output.ok_or("GPU wave output unavailable")?;
 let mut host_readback=None;
 unsafe{dev.CreateCommittedResource(&heap(D3D12_HEAP_TYPE_READBACK),D3D12_HEAP_FLAG_NONE,
   &resource_desc(D3D12_RESOURCE_FLAG_NONE),D3D12_RESOURCE_STATE_COPY_DEST,None,&mut host_readback)}
  .map_err(|e|format!("Wave64 readback allocation: {e}"))?;
 let readback:ID3D12Resource=host_readback.ok_or("Wave64 readback missing")?;
 let allocator:ID3D12CommandAllocator=unsafe{dev.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_COMPUTE)}
  .map_err(|e|format!("Wave64 command allocator: {e}"))?;
 let list:ID3D12GraphicsCommandList=unsafe{dev.CreateCommandList(
   0,D3D12_COMMAND_LIST_TYPE_COMPUTE,&allocator,Some(&pso))}
  .map_err(|e|format!("Wave64 compute command list: {e}"))?;
 unsafe{
  list.SetComputeRootSignature(&root);
  list.SetPipelineState(&pso);
  list.SetComputeRootUnorderedAccessView(0,gpu_output.GetGPUVirtualAddress());
  list.Dispatch(1,1,1);
 }
 let mut transition=D3D12_RESOURCE_BARRIER{
  Type:D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
  Flags:D3D12_RESOURCE_BARRIER_FLAG_NONE,
  Anonymous:D3D12_RESOURCE_BARRIER_0{Transition:ManuallyDrop::new(
    D3D12_RESOURCE_TRANSITION_BARRIER{
      pResource:ManuallyDrop::new(Some(gpu_output.clone())),
      Subresource:D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
      StateBefore:D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
      StateAfter:D3D12_RESOURCE_STATE_COPY_SOURCE,
    }
  )},
 };
 unsafe{
  list.ResourceBarrier(std::slice::from_ref(&transition));
  ManuallyDrop::drop(&mut (*transition.Anonymous.Transition).pResource);
  list.CopyBufferRegion(&readback,0,&gpu_output,0,256);
  list.Close().map_err(|e|format!("Wave64 command close: {e}"))?;
 }
 let native:ID3D12CommandList=list.cast().map_err(|e|e.to_string())?;
 let fence:ID3D12Fence=unsafe{dev.CreateFence(0,D3D12_FENCE_FLAG_NONE)}
  .map_err(|e|format!("Wave64 dispatch fence: {e}"))?;
 unsafe{
  queue.ExecuteCommandLists(&[Some(native)]);
  queue.Signal(&fence,1).map_err(|e|format!("Wave64 fence signal: {e}"))?;
 }
 let start=Instant::now();
 while unsafe{fence.GetCompletedValue()}<1{
  if start.elapsed()>Duration::from_secs(5){return Err("WaveOps dispatch timeout".into());}
  thread::sleep(Duration::from_millis(1));
 }
 let mut host=std::ptr::null_mut();
 unsafe{readback.Map(0,Some(&D3D12_RANGE{Begin:0,End:256}),Some(&mut host))}
  .map_err(|e|format!("Wave64 readback mapping: {e}"))?;
 if host.is_null(){return Err("WaveOps GPU readback null".into());}
 let bytes=unsafe{std::slice::from_raw_parts(host.cast::<u8>(),256)};
 let lanes=bytes.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect::<Vec<_>>();
 unsafe{readback.Unmap(0,Some(&D3D12_RANGE{Begin:0,End:0}));}
 let size=lanes[0]&255;
 if !(size==32||size==64){
  return Err(format!("DXIL WaveGetLaneCount returned unexpected wave width {size}"));
 }
 let expected_sum=|i:usize|{
  if size==64{2080}
  else if i<32{528}else{1552}
 };
 for (i,&packed) in lanes.iter().enumerate(){
  if packed&255!=size||packed>>8!=expected_sum(i){
   return Err(format!("GPU WaveActiveSum mismatch at lane {i}: got {packed}, expected sum {}",expected_sum(i)));
  }
 }
 Ok(Report{wave_size:size,lanes_checked:lanes.len(),subgroup_sum:lanes[0]>>8,hardware_gpu:true})
}
#[cfg(test)]mod tests{
 #[test]fn real_dxc_dxil_wave_intrinsics_execute_on_actual_d3d12_compute_queue(){
  let r=super::execute().expect("WaveOps unavailable; do not claim shader-model-6 support");
  assert!(r.hardware_gpu&&r.lanes_checked==64&&(r.wave_size==32||r.wave_size==64));
  assert_eq!(r.subgroup_sum,if r.wave_size==64{2080}else{528});
  eprintln!("HORIZON_ACTUAL_GPU_WAVE_SIZE={} SUBGROUP_SUM={}; DXC_SM6_NATIVE_DISPATCH=true",r.wave_size,r.subgroup_sum);
 }
}


#[derive(Default)]struct State{running:bool,result:Option<String>}
static STATUS:std::sync::OnceLock<std::sync::Arc<std::sync::Mutex<State>>>=std::sync::OnceLock::new();
fn state()->&'static std::sync::Arc<std::sync::Mutex<State>>{
 STATUS.get_or_init(||std::sync::Arc::new(std::sync::Mutex::new(State::default())))
}
pub fn start()->Result<(),String>{
 let shared=state().clone();
 {
  let mut s=shared.lock().map_err(|_|"WaveOps worker state unavailable")?;
  if s.running{return Err("WaveOps diagnostic already running".into());}
  s.running=true;s.result=None;
 }
 let worker=shared.clone();
 if let Err(e)=thread::Builder::new().name("Horizon D3D12 WaveOps GPU proof".into()).spawn(move||{
  let message=match execute(){
   Ok(r)=>format!("Real D3D12 GPU Wave64 = {}, native WaveActiveSum={} across {} computed lanes. Not an FSR3/AMD optical-flow integration.",r.wave_size==64,r.subgroup_sum,r.lanes_checked),
   Err(e)=>format!("No validated Wave64 GPU result: {e}"),
  };
  if let Ok(mut s)=worker.lock(){s.running=false;s.result=Some(message);}
 }){
  if let Ok(mut s)=shared.lock(){s.running=false;}
  return Err(format!("Cannot launch WaveOps worker: {e}"));
 }
 Ok(())
}
pub fn snapshot()->(bool,Option<String>){
 state().lock().map(|s|(s.running,s.result.clone())).unwrap_or((false,None))
}
