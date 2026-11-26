//! RX580 D3D12 independent COMPUTE queue + GPU copy proof. Not yet
//! asynchronous Horizon video shader dispatch (requires DXIL PSO + fences).
use std::{sync::{Arc,Mutex,OnceLock},thread,time::{Duration,Instant}};
use windows::{core::Interface,Win32::Graphics::{
 Direct3D::D3D_FEATURE_LEVEL_11_0,
 Direct3D12::*,
 Dxgi::Common::{DXGI_FORMAT_UNKNOWN,DXGI_SAMPLE_DESC},
}};
fn heap(kind:D3D12_HEAP_TYPE)->D3D12_HEAP_PROPERTIES{
 D3D12_HEAP_PROPERTIES{Type:kind,CPUPageProperty:Default::default(),
  MemoryPoolPreference:Default::default(),CreationNodeMask:1,VisibleNodeMask:1}
}
fn buffer_desc()->D3D12_RESOURCE_DESC{
 D3D12_RESOURCE_DESC{
  Dimension:D3D12_RESOURCE_DIMENSION_BUFFER,Alignment:0,Width:256,
  Height:1,DepthOrArraySize:1,MipLevels:1,Format:DXGI_FORMAT_UNKNOWN,
  SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
  Layout:D3D12_TEXTURE_LAYOUT_ROW_MAJOR,Flags:D3D12_RESOURCE_FLAG_NONE,
 }
}
pub fn prove()->Result<String,String>{
 let mut device:Option<ID3D12Device>=None;
 unsafe{D3D12CreateDevice(None::<&windows::core::IUnknown>,D3D_FEATURE_LEVEL_11_0,&mut device)}
  .map_err(|e|format!("Hardware DX12 absent: {e}"))?;
 let device=device.ok_or("D3D12 device missing")?;
 let desc=D3D12_COMMAND_QUEUE_DESC{
  Type:D3D12_COMMAND_LIST_TYPE_COMPUTE,
  Priority:D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0,
  Flags:D3D12_COMMAND_QUEUE_FLAG_NONE,NodeMask:0
 };
 let compute:ID3D12CommandQueue=unsafe{device.CreateCommandQueue(&desc)}
  .map_err(|e|format!("RX580 compute queue creation failed: {e}"))?;
 let direct:ID3D12CommandQueue=unsafe{device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC{
    Type:D3D12_COMMAND_LIST_TYPE_DIRECT,..desc
 })}.map_err(|e|format!("RX580 independent direct queue failed: {e}"))?;
 if compute.as_raw()==direct.as_raw(){return Err("DX12 command queues were not independent".into());}
 let resource=buffer_desc();
 let make=|kind,state|->Result<ID3D12Resource,String>{
  let mut result=None;
  unsafe{device.CreateCommittedResource(&heap(kind),D3D12_HEAP_FLAG_NONE,
    &resource,state,None,&mut result)}
    .map_err(|e|format!("D3D12 queue proof memory: {e}"))?;
  result.ok_or("D3D12 resource missing".into())
 };
 let upload=make(D3D12_HEAP_TYPE_UPLOAD,D3D12_RESOURCE_STATE_GENERIC_READ)?;
 let readback=make(D3D12_HEAP_TYPE_READBACK,D3D12_RESOURCE_STATE_COPY_DEST)?;
 let expected=(0..256).map(|x|((x*53+41)%251)as u8).collect::<Vec<_>>();
 let mut mapped=std::ptr::null_mut();
 unsafe{upload.Map(0,Some(&D3D12_RANGE{Begin:0,End:0}),Some(&mut mapped))}
  .map_err(|e|format!("D3D12 upload map failed: {e}"))?;
 if mapped.is_null(){return Err("D3D12 upload mapped null".into());}
 unsafe{std::ptr::copy_nonoverlapping(expected.as_ptr(),mapped.cast::<u8>(),256);
   upload.Unmap(0,Some(&D3D12_RANGE{Begin:0,End:256}));}
 // Both queues receive actual independent GPU copy workloads (not just
 // one empty DIRECT fence). We still do not claim live WGC shader overlap.
 let direct_readback=make(D3D12_HEAP_TYPE_READBACK,D3D12_RESOURCE_STATE_COPY_DEST)?;
 let direct_allocator:ID3D12CommandAllocator=unsafe{device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)}
  .map_err(|e|format!("D3D12 direct allocator: {e}"))?;
 let direct_list:ID3D12GraphicsCommandList=unsafe{device.CreateCommandList(
   0,D3D12_COMMAND_LIST_TYPE_DIRECT,&direct_allocator,None::<&ID3D12PipelineState>)}
  .map_err(|e|format!("D3D12 direct command list: {e}"))?;
 unsafe{direct_list.CopyBufferRegion(&direct_readback,0,&upload,0,256);
 direct_list.Close()}.map_err(|e|format!("D3D12 direct command list close: {e}"))?;
 let direct_native:ID3D12CommandList=direct_list.cast().map_err(|e|e.to_string())?;
 let allocator:ID3D12CommandAllocator=unsafe{device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_COMPUTE)}
  .map_err(|e|format!("D3D12 compute allocator: {e}"))?;
 let list:ID3D12GraphicsCommandList=unsafe{device.CreateCommandList(
  0,D3D12_COMMAND_LIST_TYPE_COMPUTE,&allocator,None::<&ID3D12PipelineState>)}
  .map_err(|e|format!("D3D12 compute command list: {e}"))?;
 unsafe{list.CopyBufferRegion(&readback,0,&upload,0,256);
 list.Close()}.map_err(|e|format!("D3D12 compute list close: {e}"))?;
 let raw:ID3D12CommandList=list.cast().map_err(|e|e.to_string())?;
 let cq_fence:ID3D12Fence=unsafe{device.CreateFence(0,D3D12_FENCE_FLAG_NONE)}
  .map_err(|e|format!("D3D12 compute fence: {e}"))?;
 let dq_fence:ID3D12Fence=unsafe{device.CreateFence(0,D3D12_FENCE_FLAG_NONE)}
  .map_err(|e|format!("D3D12 direct fence: {e}"))?;
 unsafe{
  // Independent workloads are enqueued without a CPU wait in between.
  compute.ExecuteCommandLists(&[Some(raw)]);
  direct.ExecuteCommandLists(&[Some(direct_native)]);
  compute.Signal(&cq_fence,1).map_err(|e|format!("Compute queue fence: {e}"))?;
  direct.Signal(&dq_fence,1).map_err(|e|format!("Direct queue fence: {e}"))?;
 }
 let start=Instant::now();
 while unsafe{cq_fence.GetCompletedValue()}<1||unsafe{dq_fence.GetCompletedValue()}<1{
  if start.elapsed()>Duration::from_secs(5){return Err("D3D12 compute/direct independent queues timed out".into());}
  thread::sleep(Duration::from_millis(1));
 }
 let mut gpu_data=std::ptr::null_mut();
 unsafe{readback.Map(0,Some(&D3D12_RANGE{Begin:0,End:256}),Some(&mut gpu_data))}
  .map_err(|e|format!("D3D12 compute queue readback: {e}"))?;
 if gpu_data.is_null(){return Err("D3D12 GPU readback returned null".into());}
 let observed=unsafe{std::slice::from_raw_parts(gpu_data.cast::<u8>(),256)};
 let valid=observed==expected.as_slice();
 unsafe{readback.Unmap(0,Some(&D3D12_RANGE{Begin:0,End:0}));}
 if !valid{return Err("D3D12 COMPUTE queue GPU copy disagrees with 256 byte reference".into());}
 let mut direct_gpu=std::ptr::null_mut();
 unsafe{direct_readback.Map(0,Some(&D3D12_RANGE{Begin:0,End:256}),Some(&mut direct_gpu))}
  .map_err(|e|format!("D3D12 direct queue readback: {e}"))?;
 if direct_gpu.is_null(){return Err("D3D12 direct queue readback is null".into());}
 let direct_valid=unsafe{std::slice::from_raw_parts(direct_gpu.cast::<u8>(),256)}==expected.as_slice();
 unsafe{direct_readback.Unmap(0,Some(&D3D12_RANGE{Begin:0,End:0}));}
 if !direct_valid{return Err("D3D12 DIRECT queue GPU copy disagrees with 256-byte reference".into());}
 Ok(format!("DX12 COMPUTE and DIRECT queues: two real independent 256-byte GPU buffer copies verified, both readbacks exact, fence 1/1, {:.2} ms. Queue concurrency is available; live WGC asynchronous compute SHADER scheduling is still pending.",start.elapsed().as_secs_f64()*1000.))
}
#[derive(Default)]struct State{busy:bool,message:Option<String>}
static STATE:OnceLock<Arc<Mutex<State>>>=OnceLock::new();
fn state()->&'static Arc<Mutex<State>>{STATE.get_or_init(||Arc::new(Mutex::new(State::default())))}
pub fn start()->Result<(),String>{
 let shared=state().clone();
 {
  let mut s=shared.lock().map_err(|_|"D3D12 worker guard unavailable")?;
  if s.busy{return Err("Async GPU proof is already running".into());}
  s.busy=true;s.message=None;
 }
 let worker=shared.clone();
 if let Err(e)=thread::Builder::new().name("Horizon independent compute queue proof".into()).spawn(move||{
  let outcome=prove().unwrap_or_else(|e|format!("Async compute queue not available: {e}"));
  if let Ok(mut v)=worker.lock(){v.busy=false;v.message=Some(outcome);}
 }){
  if let Ok(mut s)=shared.lock(){s.busy=false;}
  return Err(format!("DX12 async diagnostic worker failed to launch: {e}"));
 }
 Ok(())
}
pub fn snapshot()->(bool,Option<String>){
 state().lock().map(|s|(s.busy,s.message.clone())).unwrap_or((false,None))
}
#[cfg(test)]mod tests{
 #[test]fn actual_independent_dx12_compute_queue_executes_gpu_memory_op(){
  let s=super::prove().expect("A supported GPU cannot claim success without actual compute queue GPU command");
  assert!(s.contains("two real independent")&&s.contains("both readbacks exact")&&s.contains("pending"));
 }
}
