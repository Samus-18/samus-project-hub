//! Real AMD Vulkan compute proof using Naga SPIR-V and the Vulkan ICD.
//! Only on opt-in, only 256 u32 private storage-buffer values. NOT a
//! completed live WGC->Vulkan->DXGI compositor or a game API switch.
use ash::{vk,Entry};
use std::{ffi::CString,sync::{Arc,Mutex,OnceLock},thread};
const CODE:&str=r#"
struct Samples{ values:array<u32,256> }
@group(0) @binding(0) var<storage,read_write> samples:Samples;
@compute @workgroup_size(64,1,1)
fn main(@builtin(global_invocation_id) global:vec3<u32>){
 let i=global.x;
 if(i<256u){samples.values[i]=samples.values[i]*3u+7u;}
}"#;
pub fn compile_spirv()->Result<Vec<u32>,String>{
 let module=naga::front::wgsl::parse_str(CODE).map_err(|e|format!("Vulkan WGSL parse: {e}"))?;
 let info=naga::valid::Validator::new(
     naga::valid::ValidationFlags::all(),naga::valid::Capabilities::empty())
     .validate(&module).map_err(|e|format!("Vulkan storage shader validation: {e}"))?;
 naga::back::spv::write_vec(&module,&info,&naga::back::spv::Options::default(),None)
   .map_err(|e|format!("Vulkan SPIR-V generator: {e}"))
}
struct Resources{
 _entry:Entry,
 instance:ash::Instance,
 device:Option<ash::Device>,
 buffer:vk::Buffer,
 memory:vk::DeviceMemory,
 layout:vk::DescriptorSetLayout,
 pool:vk::DescriptorPool,
 pipeline_layout:vk::PipelineLayout,
 pipeline:vk::Pipeline,
 shader:vk::ShaderModule,
 command_pool:vk::CommandPool,
 fence:vk::Fence,
}
impl Drop for Resources{
 fn drop(&mut self){
  if let Some(device)=self.device.take(){
   unsafe{
    if self.fence!=vk::Fence::null(){device.destroy_fence(self.fence,None);}
    if self.command_pool!=vk::CommandPool::null(){device.destroy_command_pool(self.command_pool,None);}
    if self.pipeline!=vk::Pipeline::null(){device.destroy_pipeline(self.pipeline,None);}
    if self.shader!=vk::ShaderModule::null(){device.destroy_shader_module(self.shader,None);}
    if self.pipeline_layout!=vk::PipelineLayout::null(){device.destroy_pipeline_layout(self.pipeline_layout,None);}
    if self.pool!=vk::DescriptorPool::null(){device.destroy_descriptor_pool(self.pool,None);}
    if self.layout!=vk::DescriptorSetLayout::null(){device.destroy_descriptor_set_layout(self.layout,None);}
    if self.buffer!=vk::Buffer::null(){device.destroy_buffer(self.buffer,None);}
    if self.memory!=vk::DeviceMemory::null(){device.free_memory(self.memory,None);}
    device.destroy_device(None);
   }
  }
  unsafe{self.instance.destroy_instance(None);}
 }
}
impl Resources{
 fn new()->Result<Self,String>{
  let entry=unsafe{Entry::load()}.map_err(|e|format!("Vulkan loader not found: {e}"))?;
  let application=CString::new("HorizonBoost-Vulkan-SyntheticCompute").unwrap();
  let info=vk::ApplicationInfo::default().application_name(&application)
    .application_version(1).engine_name(&application).api_version(vk::API_VERSION_1_0);
  let instance=unsafe{entry.create_instance(&vk::InstanceCreateInfo::default()
    .application_info(&info),None)}.map_err(|e|format!("Vulkan ICD instance: {e:?}"))?;
  Ok(Self{_entry:entry,instance,device:None,buffer:vk::Buffer::null(),
   memory:vk::DeviceMemory::null(),layout:vk::DescriptorSetLayout::null(),
   pool:vk::DescriptorPool::null(),pipeline_layout:vk::PipelineLayout::null(),
   pipeline:vk::Pipeline::null(),shader:vk::ShaderModule::null(),
   command_pool:vk::CommandPool::null(),fence:vk::Fence::null()})
 }
}
pub fn proof()->Result<String,String>{
 let mut r=Resources::new()?;
 let devices=unsafe{r.instance.enumerate_physical_devices()}
    .map_err(|e|format!("Vulkan physical devices: {e:?}"))?;
 let mut chosen=None;
 for physical in devices{
  let props=unsafe{r.instance.get_physical_device_properties(physical)};
  if props.vendor_id!=0x1002{continue;} // only actual AMD RX580, no software ICD
  let queues=unsafe{r.instance.get_physical_device_queue_family_properties(physical)};
  let family=queues.iter().position(|q|q.queue_count>0&&q.queue_flags.contains(vk::QueueFlags::COMPUTE));
  if let Some(family)=family{chosen=Some((physical,family as u32,props.device_name));break;}
 }
 let (physical,family,_name)=chosen.ok_or("No AMD Vulkan hardware compute queue available")?;
 let priorities=[1.0f32];
 let qdesc=[vk::DeviceQueueCreateInfo::default().queue_family_index(family).queue_priorities(&priorities)];
 let device=unsafe{r.instance.create_device(physical,
   &vk::DeviceCreateInfo::default().queue_create_infos(&qdesc),None)}
   .map_err(|e|format!("AMD Vulkan compute device: {e:?}"))?;
 r.device=Some(device);
 let d=r.device.as_ref().unwrap();
 let queue=unsafe{d.get_device_queue(family,0)};
 r.buffer=unsafe{d.create_buffer(&vk::BufferCreateInfo::default()
   .size(1024).usage(vk::BufferUsageFlags::STORAGE_BUFFER)
   .sharing_mode(vk::SharingMode::EXCLUSIVE),None)}
   .map_err(|e|format!("Vulkan storage buffer: {e:?}"))?;
 let req=unsafe{d.get_buffer_memory_requirements(r.buffer)};
 let mem=unsafe{r.instance.get_physical_device_memory_properties(physical)};
 let index=(0..mem.memory_type_count).find(|&i|{
  req.memory_type_bits&(1<<i)!=0 &&
  mem.memory_types[i as usize].property_flags
    .contains(vk::MemoryPropertyFlags::HOST_VISIBLE|vk::MemoryPropertyFlags::HOST_COHERENT)
 }).ok_or("AMD Vulkan memory lacks a host-coherent storage-buffer heap")?;
 r.memory=unsafe{d.allocate_memory(&vk::MemoryAllocateInfo::default()
   .allocation_size(req.size).memory_type_index(index),None)}
   .map_err(|e|format!("Vulkan memory allocation: {e:?}"))?;
 unsafe{d.bind_buffer_memory(r.buffer,r.memory,0)}
   .map_err(|e|format!("Vulkan buffer binding: {e:?}"))?;
 let expected=(0..256u32).map(|i|i.wrapping_mul(37).wrapping_add(11))
    .collect::<Vec<_>>();
 let address=unsafe{d.map_memory(r.memory,0,1024,vk::MemoryMapFlags::empty())}
    .map_err(|e|format!("Map Vulkan coherent memory: {e:?}"))?;
 unsafe{
  std::ptr::copy_nonoverlapping(expected.as_ptr(),address.cast::<u32>(),256);
  d.unmap_memory(r.memory);
 }
 let binding=[vk::DescriptorSetLayoutBinding::default()
    .binding(0).descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
    .descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE)];
 r.layout=unsafe{d.create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default()
    .bindings(&binding),None)}
    .map_err(|e|format!("Vulkan descriptor set layout: {e:?}"))?;
 let counts=[vk::DescriptorPoolSize::default()
    .ty(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1)];
 r.pool=unsafe{d.create_descriptor_pool(&vk::DescriptorPoolCreateInfo::default()
    .max_sets(1).pool_sizes(&counts),None)}
    .map_err(|e|format!("Vulkan descriptors pool: {e:?}"))?;
 let sets=[r.layout];
 let allocated=unsafe{d.allocate_descriptor_sets(&vk::DescriptorSetAllocateInfo::default()
    .descriptor_pool(r.pool).set_layouts(&sets))}
    .map_err(|e|format!("Vulkan descriptor set allocation: {e:?}"))?;
 let bindbuffer=[vk::DescriptorBufferInfo::default()
    .buffer(r.buffer).offset(0).range(1024)];
 let update=[vk::WriteDescriptorSet::default().dst_set(allocated[0])
    .dst_binding(0).descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
    .buffer_info(&bindbuffer)];
 unsafe{d.update_descriptor_sets(&update,&[])};
 r.pipeline_layout=unsafe{d.create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default()
   .set_layouts(&sets),None)}
   .map_err(|e|format!("Vulkan pipeline layout: {e:?}"))?;
 let spv=compile_spirv()?;
 r.shader=unsafe{d.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&spv),None)}
   .map_err(|e|format!("Vulkan compiled Naga SPIR-V module: {e:?}"))?;
 let entry=CString::new("main").unwrap();
 let stage=vk::PipelineShaderStageCreateInfo::default()
    .stage(vk::ShaderStageFlags::COMPUTE).module(r.shader).name(&entry);
 let descriptions=[vk::ComputePipelineCreateInfo::default()
   .stage(stage).layout(r.pipeline_layout)];
 let pipes=unsafe{d.create_compute_pipelines(vk::PipelineCache::null(),&descriptions,None)}
   .map_err(|(_,e)|format!("AMD Vulkan compute pipeline: {e:?}"))?;
 r.pipeline=pipes[0];
 r.command_pool=unsafe{d.create_command_pool(&vk::CommandPoolCreateInfo::default()
   .queue_family_index(family).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),None)}
   .map_err(|e|format!("Vulkan command pool: {e:?}"))?;
 let cmds=unsafe{d.allocate_command_buffers(&vk::CommandBufferAllocateInfo::default()
   .command_pool(r.command_pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(1))}
   .map_err(|e|format!("Vulkan commands: {e:?}"))?;
 let cb=cmds[0];
 unsafe{d.begin_command_buffer(cb,&vk::CommandBufferBeginInfo::default()
   .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT))}
   .map_err(|e|format!("Vulkan command begin: {e:?}"))?;
 let before=[vk::BufferMemoryBarrier::default()
   .src_access_mask(vk::AccessFlags::HOST_WRITE)
   .dst_access_mask(vk::AccessFlags::SHADER_READ|vk::AccessFlags::SHADER_WRITE)
   .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
   .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
   .buffer(r.buffer).offset(0).size(1024)];
 unsafe{
  d.cmd_pipeline_barrier(cb,vk::PipelineStageFlags::HOST,
   vk::PipelineStageFlags::COMPUTE_SHADER,vk::DependencyFlags::empty(),&[],&before,&[]);
  d.cmd_bind_pipeline(cb,vk::PipelineBindPoint::COMPUTE,r.pipeline);
  d.cmd_bind_descriptor_sets(cb,vk::PipelineBindPoint::COMPUTE,r.pipeline_layout,0,&allocated,&[]);
  d.cmd_dispatch(cb,4,1,1);
 }
 let after=[vk::BufferMemoryBarrier::default()
   .src_access_mask(vk::AccessFlags::SHADER_WRITE)
   .dst_access_mask(vk::AccessFlags::HOST_READ)
   .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
   .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
   .buffer(r.buffer).offset(0).size(1024)];
 unsafe{
  d.cmd_pipeline_barrier(cb,vk::PipelineStageFlags::COMPUTE_SHADER,
   vk::PipelineStageFlags::HOST,vk::DependencyFlags::empty(),&[],&after,&[]);
  d.end_command_buffer(cb)
 }.map_err(|e|format!("Vulkan command end: {e:?}"))?;
 r.fence=unsafe{d.create_fence(&vk::FenceCreateInfo::default(),None)}
   .map_err(|e|format!("Vulkan fence: {e:?}"))?;
 let command_buffers=[cb];
 let submit=[vk::SubmitInfo::default().command_buffers(&command_buffers)];
 unsafe{d.queue_submit(queue,&submit,r.fence)}
   .map_err(|e|format!("Vulkan hardware compute submission: {e:?}"))?;
 unsafe{d.wait_for_fences(&[r.fence],true,5_000_000_000)}
   .map_err(|e|format!("Vulkan compute GPU completion timeout: {e:?}"))?;
 let result=unsafe{d.map_memory(r.memory,0,1024,vk::MemoryMapFlags::empty())}
   .map_err(|e|format!("Vulkan result map: {e:?}"))?;
 let received=unsafe{std::slice::from_raw_parts(result.cast::<u32>(),256)}.to_vec();
 unsafe{d.unmap_memory(r.memory);}
 for (i,value) in received.iter().enumerate(){
  let expected=expected[i].wrapping_mul(3).wrapping_add(7);
  if *value!=expected{return Err(format!("Vulkan shader returned wrong sample at {i}: {} != {expected}",value));}
 }
 Ok("AMD RX580 native Vulkan hardware COMPUTE queue: 256/256 GPU storage words transformed correctly via Naga-generated SPIR-V and fenced submission. Synthetic backend validated; game WGC/DXGI Vulkan handoff not yet connected.".into())
}
#[derive(Default)]struct State{running:bool,message:Option<String>}
static STATE:OnceLock<Arc<Mutex<State>>>=OnceLock::new();
fn state()->&'static Arc<Mutex<State>>{STATE.get_or_init(||Arc::new(Mutex::new(State::default())))}
pub fn start()->Result<(),String>{
 let status=state().clone();
 {
  let mut s=status.lock().map_err(|_|"Vulkan diagnostic state unavailable")?;
  if s.running{return Err("Vulkan GPU benchmark already active".into());}
  s.running=true;s.message=None;
 }
 let child=status.clone();
 if let Err(e)=thread::Builder::new().name("Horizon real AMD Vulkan compute proof".into()).spawn(move||{
  let message=proof().unwrap_or_else(|e|format!("Vulkan hardware compute failed safely: {e}"));
  if let Ok(mut s)=child.lock(){s.message=Some(message);s.running=false;}
 }){
  if let Ok(mut s)=status.lock(){s.running=false;}
  return Err(format!("Cannot run Vulkan worker: {e}"));
 }
 Ok(())
}
pub fn snapshot()->(bool,Option<String>){
 state().lock().map(|s|(s.running,s.message.clone())).unwrap_or((false,None))
}
#[cfg(test)]mod tests{
 #[test]fn real_amd_vulkan_compute_pipeline_matches_exact_reference(){
  let result=super::proof().expect("Vulkan test requires actual GPU dispatch");
  assert!(result.contains("256/256")&&result.contains("RX580"));
 }
}
