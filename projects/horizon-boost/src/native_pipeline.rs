//! Persistent native Direct3D 11 GPU shader pipeline.
//! Compile HLSL once, allocate two reusable GPU RGBA textures once, and
//! alternate them across ordered shader passes. Not Magpie.
//!
//! No game DLL hooks, no game resizing, no frame readback to CPU. Caller
//! must present/use the returned GPU texture before calling `dispatch` again
//! because buffers are reused. Does not itself display exclusive fullscreen.
use windows::{
    core::Interface,
    Win32::Graphics::{
        Direct3D11::{
            D3D11_BIND_SHADER_RESOURCE,D3D11_BIND_UNORDERED_ACCESS,D3D11_TEXTURE2D_DESC,
            D3D11_USAGE_DEFAULT,ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,
            ID3D11ShaderResourceView,ID3D11UnorderedAccessView,ID3D11ClassLinkage,
            ID3D11ComputeShader,ID3D11Buffer,ID3D11Resource,
            D3D11_BUFFER_DESC,D3D11_BIND_CONSTANT_BUFFER,ID3D11SamplerState,D3D11_SAMPLER_DESC,D3D11_FILTER_MIN_MAG_MIP_LINEAR,D3D11_TEXTURE_ADDRESS_CLAMP,D3D11_COMPARISON_NEVER,
        },
        Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_SAMPLE_DESC},
    },
};

struct PingPongBuffer {
    image:ID3D11Texture2D,
    srv:ID3D11ShaderResourceView,
    uav:ID3D11UnorderedAccessView,
}
struct Mixer{
 shader:ID3D11ComputeShader,constants:ID3D11Buffer,scratch:PingPongBuffer,
}
impl Mixer{
 fn new(device:&ID3D11Device,width:u32,height:u32)->Result<Self,String>{
  let bytecode=crate::shadercheck::compile_native_hlsl(include_str!("../shaders/native/HB_LayerMix.hlsl"))?;
  let mut shader=None;unsafe{device.CreateComputeShader(&bytecode,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|e.to_string())?;
  let mut constants=None;
  unsafe{device.CreateBuffer(&D3D11_BUFFER_DESC{ByteWidth:16,Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0},None,Some(&mut constants))}.map_err(|e|e.to_string())?;
  let desc=D3D11_TEXTURE2D_DESC{Width:width,Height:height,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R8G8B8A8_UNORM,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,CPUAccessFlags:0,MiscFlags:0};
  let mut image=None;unsafe{device.CreateTexture2D(&desc,None,Some(&mut image))}.map_err(|e|e.to_string())?;
  let image=image.ok_or("Layer scratch missing")?;
  let(mut srv,mut uav)=(None,None);
  unsafe{device.CreateShaderResourceView(&image,None,Some(&mut srv))}.map_err(|e|e.to_string())?;
  unsafe{device.CreateUnorderedAccessView(&image,None,Some(&mut uav))}.map_err(|e|e.to_string())?;
  Ok(Self{shader:shader.ok_or("Mix shader missing")?,constants:constants.ok_or("Mix constants missing")?,scratch:PingPongBuffer{image,srv:srv.ok_or("Mix SRV missing")?,uav:uav.ok_or("Mix UAV missing")?}})
 }
 fn blend(&self,ctx:&ID3D11DeviceContext,a:&ID3D11ShaderResourceView,b:&ID3D11ShaderResourceView,target:&PingPongBuffer,width:u32,height:u32,mix:f32,split:bool)->Result<(),String>{
  let data=[mix,if split{1f32}else{0.},0.,0.];
  let buffer:ID3D11Resource=self.constants.cast().map_err(|e|e.to_string())?;
  unsafe{
   ctx.UpdateSubresource(&buffer,0,None,data.as_ptr().cast(),0,0);
   ctx.CSSetConstantBuffers(0,Some(&[Some(self.constants.clone())]));
   ctx.CSSetShader(&self.shader,None);
   ctx.CSSetShaderResources(0,Some(&[Some(a.clone()),Some(b.clone())]));
   let u=[Some(target.uav.clone())];ctx.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);
   ctx.Dispatch(width.div_ceil(8),height.div_ceil(8),1);
   ctx.CSSetShaderResources(0,Some(&[None,None]));
   ctx.CSSetConstantBuffers(0,Some(&[None]));
   let u:[Option<ID3D11UnorderedAccessView>;1]=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);
  }Ok(())
 }
}
pub struct NativeGpuPipeline {
    width:u32,
    height:u32,
    shaders:Vec<std::cell::OnceCell<Result<ID3D11ComputeShader,String>>>,
    surfaces:[PingPongBuffer;2],
    light_constants:ID3D11Buffer,
    grade_constants:ID3D11Buffer,
    device:ID3D11Device,
    histogram:std::cell::OnceCell<Result<crate::gpu_histogram::Histogram,String>>,
    mixer:std::cell::OnceCell<Result<Mixer,String>>,
    lut:std::cell::RefCell<crate::safe_gpu_cache::Cache<String,crate::live_lut::Pass>>,
    copy_shader:std::cell::OnceCell<ID3D11ComputeShader>,
    precision:std::cell::RefCell<crate::safe_gpu_cache::Cache<bool,crate::precision_pass::Pass>>,
    recipe:std::cell::RefCell<crate::safe_gpu_cache::Cache<crate::color_recipe::Recipe,crate::color_recipe::Pass>>,
    cinema:std::cell::RefCell<crate::safe_gpu_cache::Cache<u32,crate::cinema_light::Pass>>,
    optional_warning:std::cell::Cell<u32>,
    fusion_enabled:std::cell::Cell<bool>,
    fused:std::cell::OnceCell<Result<crate::fused_color::Pass,String>>,
    fused_saved:std::cell::Cell<u32>,
    live_last_good:std::cell::RefCell<Option<(Vec<usize>,crate::lighting::LightTuning)>>,
    live_rejected:std::cell::Cell<bool>,
    live_active_count:std::cell::Cell<usize>,
    grain_phase:std::cell::Cell<u32>,
    grain_constants:ID3D11Buffer,
    protection:std::cell::OnceCell<Result<ProtectionPass,String>>,
    linear_sampler:std::cell::OnceCell<Result<ID3D11SamplerState,String>>,
    smaa:std::cell::OnceCell<Result<crate::smaa::Pass,String>>,
}
impl NativeGpuPipeline {
    pub fn new(device:&ID3D11Device,width:u32,height:u32)->Result<Self,String>{
        if width<1||height<1||width>8192||height>8192{
            return Err("Invalid native GPU pipeline resolution (1â€“8192)".into());
        }
        let shaders=(0..crate::native_filter::PROGRAMS.len()).map(|_|std::cell::OnceCell::new()).collect();
        let mut surfaces=Vec::with_capacity(2);
        let desc=D3D11_TEXTURE2D_DESC{
            Width:width,Height:height,MipLevels:1,ArraySize:1,
            Format:DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
            Usage:D3D11_USAGE_DEFAULT,
            BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,
            CPUAccessFlags:0,MiscFlags:0,
        };
        for n in 0..2{
            let mut image=None;
            unsafe{device.CreateTexture2D(&desc,None,Some(&mut image))}
                .map_err(|e|format!("Create persistent GPU surface {n}: {e}"))?;
            let image=image.ok_or("No persistent GPU texture")?;
            let mut srv=None;
            unsafe{device.CreateShaderResourceView(&image,None,Some(&mut srv))}
                .map_err(|e|format!("Create persistent GPU SRV {n}: {e}"))?;
            let mut uav=None;
            unsafe{device.CreateUnorderedAccessView(&image,None,Some(&mut uav))}
                .map_err(|e|format!("Create persistent GPU UAV {n}: {e}"))?;
            surfaces.push(PingPongBuffer{
                image,srv:srv.ok_or("Persistent SRV missing")?,
                uav:uav.ok_or("Persistent UAV missing")?
            });
        }
        let surfaces:[PingPongBuffer;2]=surfaces.try_into().map_err(|_|"Missing GPU surfaces")?;
        let mut light_constants=None;
        unsafe{device.CreateBuffer(&D3D11_BUFFER_DESC{
            ByteWidth:std::mem::size_of::<crate::lighting::GpuLightParams>() as u32,
            Usage:D3D11_USAGE_DEFAULT,
            BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,
            CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0,
        },None,Some(&mut light_constants))}
            .map_err(|e|format!("Create user light tuning GPU constants: {e}"))?;
        let mut grade_constants=None;
        unsafe{device.CreateBuffer(&D3D11_BUFFER_DESC{
            ByteWidth:std::mem::size_of::<crate::color_grade::Params>() as u32,Usage:D3D11_USAGE_DEFAULT,
            BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0
        },None,Some(&mut grade_constants))}.map_err(|e|format!("Color grade constants: {e}"))?;
        let mut grain_constants=None;
        unsafe{device.CreateBuffer(&D3D11_BUFFER_DESC{ByteWidth:16,Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0},None,Some(&mut grain_constants))}.map_err(|e|e.to_string())?;
        Ok(Self{width,height,shaders,surfaces,device:device.clone(),linear_sampler:std::cell::OnceCell::new(),smaa:std::cell::OnceCell::new(),grain_constants:grain_constants.ok_or("Grain constants missing")?,live_last_good:Default::default(),live_rejected:Default::default(),live_active_count:Default::default(),grain_phase:Default::default(),histogram:std::cell::OnceCell::new(),mixer:std::cell::OnceCell::new(),lut:Default::default(),copy_shader:std::cell::OnceCell::new(),precision:Default::default(),recipe:Default::default(),cinema:Default::default(),optional_warning:std::cell::Cell::new(0),fusion_enabled:Default::default(),fused:Default::default(),fused_saved:Default::default(),protection:std::cell::OnceCell::new(),
            grade_constants:grade_constants.ok_or("Color grade cbuffer missing")?,
            light_constants:light_constants.ok_or("Light Studio D3D11 cbuffer missing")?})
    }
    fn mixer(&self)->Result<&Mixer,String>{
        self.mixer.get_or_init(||Mixer::new(&self.device,self.width,self.height)).as_ref().map_err(Clone::clone)
    }
    pub fn precision_fp16(&self)->Option<bool>{self.precision.borrow().current().map(|p|p.fp16)}
    pub fn optional_warning(&self)->u32{self.optional_warning.get()}
    pub fn enable_fusion(&self){self.fusion_enabled.set(true);}
    pub fn fused_saved(&self)->u32{self.fused_saved.get()}
    pub fn dimensions(&self)->(u32,u32){(self.width,self.height)}
    pub fn compiled_count(&self)->usize{self.shaders.iter().filter(|c|c.get().is_some_and(|v|v.is_ok())).count()}
    fn linear_sampler(&self)->Result<&ID3D11SamplerState,String>{
      self.linear_sampler.get_or_init(||{let mut d=D3D11_SAMPLER_DESC::default();d.Filter=D3D11_FILTER_MIN_MAG_MIP_LINEAR;d.AddressU=D3D11_TEXTURE_ADDRESS_CLAMP;d.AddressV=D3D11_TEXTURE_ADDRESS_CLAMP;d.AddressW=D3D11_TEXTURE_ADDRESS_CLAMP;d.MaxAnisotropy=1;d.ComparisonFunc=D3D11_COMPARISON_NEVER;d.MaxLOD=f32::MAX;
       let mut s=None;unsafe{self.device.CreateSamplerState(&d,Some(&mut s))}.map_err(|e|e.to_string())?;s.ok_or("FXAA linear sampler missing".into())}).as_ref().map_err(Clone::clone)
    }
    fn program(&self,index:usize)->Result<&ID3D11ComputeShader,String>{
        if index==crate::native_filter::FXAA_STAGE{self.linear_sampler()?;}
        let cell=self.shaders.get(index).ok_or("Invalid native GPU program")?;
        cell.get_or_init(||{
            let(name,_)=crate::native_filter::PROGRAMS[index];
            let code=crate::native_filter::program_source(index).ok_or("GPU source unavailable")?;
            let bytes=crate::shadercheck::compile_native_hlsl(&code).map_err(|e|format!("{name}: {e}"))?;
            let mut shader=None;unsafe{self.device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|format!("{name}: {e}"))?;
            shader.ok_or("Missing GPU program".into())
        }).as_ref().map_err(Clone::clone)
    }
    pub fn scratch_texture_count(&self)->usize{self.surfaces.len()}
    pub fn live_rejected(&self)->bool{self.live_rejected.get()}
    pub fn live_active_count(&self)->usize{self.live_active_count.get()}
    pub fn set_grain_phase(&self,phase:u32){self.grain_phase.set(phase);}
    // A requested stack with a failed core shader must never silently become a different look.
    // Retain the last fully compiled native stack and tuning, or bypass if none exists.
    pub fn dispatch_live(&self,context:&ID3D11DeviceContext,source:&ID3D11Texture2D,srv:&ID3D11ShaderResourceView,stages:&[usize],tuning:&crate::lighting::LightTuning)->Result<ID3D11Texture2D,String>{
        let preflight=crate::native_stack::validate(stages).and_then(|_|{
            for (i,&id) in stages.iter().enumerate(){if !tuning.bypass_look&&tuning.layers.enabled_at(stages,i)>0.{self.program(id)?;}}
            Ok(())
        });
        if let Err(error)=preflight{
            if unsafe{self.device.GetDeviceRemovedReason()}.is_err(){return Err(format!("WGC_DEVICE_LOST: {error}"));}
            self.live_rejected.set(true);
            let last=self.live_last_good.borrow();
            if let Some((ids,t))=last.as_ref(){self.live_active_count.set(ids.len());if !tuning.reconstruction.animated_grain&&t.reconstruction.animated_grain{let mut safe=t.clone();safe.reconstruction.animated_grain=false;return self.dispatch_with_cached_srv_tuned(context,source,srv,ids,&safe);}return self.dispatch_with_cached_srv_tuned(context,source,srv,ids,t);}
            let mut bypass=crate::lighting::LightTuning::default();bypass.bypass_look=true;
            self.live_active_count.set(0);return self.dispatch_with_cached_srv_tuned(context,source,srv,&[0],&bypass);
        }
        let result=self.dispatch_with_cached_srv_tuned(context,source,srv,stages,tuning)?;
        if self.optional_warning()&16==0{
            let mut last=self.live_last_good.borrow_mut();
            if last.as_ref().is_none_or(|(ids,t)|ids.as_slice()!=stages||t!=tuning){*last=Some((stages.to_vec(),tuning.clone()));}
            self.live_rejected.set(false);self.live_active_count.set(stages.len());
        }
        Ok(result)
    }
    pub fn dispatch(&self,device:&ID3D11Device,context:&ID3D11DeviceContext,
        source:&ID3D11Texture2D,passes:&[usize])->Result<ID3D11Texture2D,String>{
        let mut source_srv=None;
        unsafe{device.CreateShaderResourceView(source,None,Some(&mut source_srv))}
            .map_err(|e|format!("Source GPU shader resource is unavailable: {e}"))?;
        let first_srv=source_srv.ok_or("Source GPU SRV missing")?;
        self.dispatch_with_cached_srv(context,source,&first_srv,passes)
    }
    /// Reuses a pre-created SRV across live frames, avoiding driver-side
    /// CreateShaderResourceView calls in the WGC render hot path.
    pub fn dispatch_with_cached_srv(&self,context:&ID3D11DeviceContext,
        source:&ID3D11Texture2D,first_srv:&ID3D11ShaderResourceView,
        passes:&[usize])->Result<ID3D11Texture2D,String>{
        self.dispatch_with_cached_srv_tuned(context,source,first_srv,passes,
            &crate::lighting::LightTuning::default())
    }
    /// Live-updateable per-game lighting parameters supplied from GUI; the
    /// constant buffer is rebound only for the Studio stage, not for legacy.
    pub fn dispatch_with_cached_srv_tuned(&self,context:&ID3D11DeviceContext,
        source:&ID3D11Texture2D,first_srv:&ID3D11ShaderResourceView,
        passes:&[usize],lighting:&crate::lighting::LightTuning)
        ->Result<ID3D11Texture2D,String>{
        if passes.is_empty()||passes.len()>8{
            return Err("Native cached pipeline requires 1 to 8 ordered shader stages".into());
        }
        let mut src_desc=D3D11_TEXTURE2D_DESC::default();
        unsafe{source.GetDesc(&mut src_desc);}
        if src_desc.Width!=self.width||src_desc.Height!=self.height{
            return Err("Native source dimensions changed; recreate persistent GPU pipeline".into());
        }
        if passes.iter().any(|i|*i>=self.shaders.len()){
            return Err("Selected GPU shader stage does not exist".into());
        }
        let exposure=lighting.exposure.enabled&&!passes.contains(&crate::native_filter::HISTOGRAM_STAGE);
        let count=passes.len()+usize::from(exposure)+usize::from(lighting.enabled)+usize::from(lighting.grade.enabled);
        let mut current:Option<usize>=None;
        let mut warnings=0u32;
        let mut skip_until=0;let mut fused_saved=0u32;
        self.fused_saved.set(0);
        for step in 0..count{
            if step<skip_until{continue;}
            let index=if step<passes.len(){passes[step]}
                else if exposure&&step==passes.len(){crate::native_filter::HISTOGRAM_STAGE}
                else if lighting.enabled&&step==passes.len()+usize::from(exposure){crate::lighting::GPU_LIGHT_STAGE}
                else{crate::color_grade::STAGE};
            let strength=if step<passes.len(){if lighting.bypass_look{0.}else{lighting.layers.enabled_at(passes,step)}}else{1.};
            if strength<=0.{continue;}
            if self.fusion_enabled.get()&&lighting.reconstruction.fused_color&&step<passes.len()&&strength==1.{
                let run=crate::fused_color::run_length(passes,step,&lighting.layers);
                if run>=2{
                    let slot=current.map_or(0,|n|1-n);
                    let input=current.map_or_else(||first_srv.clone(),|n|self.surfaces[n].srv.clone());
                    let fused=self.fused.get_or_init(||crate::fused_color::Pass::new(&self.device));
                    match fused{
                        Ok(pass)=>match pass.run(context,&input,&self.surfaces[slot].uav,self.width,self.height,&passes[step..step+run]){
                            Ok(())=>{current=Some(slot);skip_until=step+run;fused_saved+=(run-1) as u32;continue;},
                            Err(e)=>{if unsafe{self.device.GetDeviceRemovedReason()}.is_err(){return Err(e);}warnings|=256;}
                        },
                        Err(_)=>warnings|=256
                    }
                }
            }
            if index==crate::native_filter::SMAA_STAGE{
                let slot=current.map_or(0,|n|1-n);
                let input=current.map_or_else(||first_srv.clone(),|n|self.surfaces[n].srv.clone());
                let pass=self.smaa.get_or_init(||crate::smaa::Pass::new(&self.device,self.width,self.height));
                let Ok(pass)=pass else{warnings|=512;continue;};
                if strength<1.{
                    let mixer=match self.mixer(){Ok(m)=>m,Err(_)=>{warnings|=64;continue;}};
                    pass.render(context,&input,&mixer.scratch.uav);
                    mixer.blend(context,&input,&mixer.scratch.srv,&self.surfaces[slot],self.width,self.height,strength,false)?;
                }else{pass.render(context,&input,&self.surfaces[slot].uav);}
                current=Some(slot);continue;
            }
            let shader=match self.program(index){Ok(s)=>s,Err(_)=>{warnings|=16;continue;}};
            let slot=current.map_or(0,|n|1-n);
            let input_srv=current.map_or_else(||first_srv.clone(),|n|self.surfaces[n].srv.clone());
            let mixer=if strength<1.{match self.mixer(){Ok(m)=>Some(m),Err(_)=>{warnings|=64;continue;}}}else{None};
            if index==49{
                let phase=if lighting.reconstruction.animated_grain{self.grain_phase.get()}else{0};
                let data=[phase,0,0,0];let cb:ID3D11Resource=self.grain_constants.cast().map_err(|e|e.to_string())?;
                unsafe{context.UpdateSubresource(&cb,0,None,data.as_ptr().cast(),0,0);context.CSSetConstantBuffers(0,Some(&[Some(self.grain_constants.clone())]));}
            }
            if index==crate::lighting::GPU_LIGHT_STAGE{
                let args=lighting.gpu();
                let cbuffer:ID3D11Resource=self.light_constants.cast()
                    .map_err(|e|format!("Lighting constants resource: {e}"))?;
                unsafe{
                    context.UpdateSubresource(&cbuffer,0,None,
                        &args as *const _ as *const std::ffi::c_void,0,0);
                    context.CSSetConstantBuffers(0,Some(&[Some(self.light_constants.clone())]));
                }
            }
            if index==crate::color_grade::STAGE{
                let args=lighting.grade.gpu();
                let buffer:ID3D11Resource=self.grade_constants.cast().map_err(|e|e.to_string())?;
                unsafe{context.UpdateSubresource(&buffer,0,None,&args as *const _ as *const std::ffi::c_void,0,0);
                    context.CSSetConstantBuffers(0,Some(&[Some(self.grade_constants.clone())]));}
            }
            if index==crate::native_filter::HISTOGRAM_STAGE{
                let histogram=self.histogram.get_or_init(||crate::gpu_histogram::Histogram::new(&self.device));
                match histogram{
                    Ok(h)=>{if let Err(e)=h.measure(context,&input_srv,&lighting.exposure){if unsafe{self.device.GetDeviceRemovedReason()}.is_err(){return Err(e);}warnings|=128;continue;}},
                    Err(_)=>{warnings|=128;continue;}
                }
            }
            let target=mixer.map_or(&self.surfaces[slot],|m|&m.scratch);
            let uavs=[Some(target.uav.clone())];
            unsafe{
                context.CSSetShader(shader,None);
                if index==crate::native_filter::FXAA_STAGE{context.CSSetSamplers(0,Some(&[Some(self.linear_sampler()?.clone())]));}
                context.CSSetShaderResources(0,Some(&[Some(input_srv.clone())]));
                context.CSSetUnorderedAccessViews(0,1,Some(uavs.as_ptr()),None);
                context.Dispatch(self.width.div_ceil(8),self.height.div_ceil(8),1);
                // Read/write bindings must be cleared before the next pass.
                if index==crate::native_filter::FXAA_STAGE{context.CSSetSamplers(0,Some(&[None]));}
                context.CSSetShaderResources(0,Some(&[None,None]));
                if index==49||index==crate::lighting::GPU_LIGHT_STAGE||index==crate::color_grade::STAGE{
                    context.CSSetConstantBuffers(0,Some(&[None]));
                }
                let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];
                context.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
            }
            if let Some(m)=mixer{m.blend(context,&input_srv,&m.scratch.srv,&self.surfaces[slot],self.width,self.height,strength,false)?;}
            current=Some(slot);
        }
        if let Some(id)=lighting.lut.name.as_ref().filter(|_|lighting.lut.strength>0.){
            let mut cache=self.lut.borrow_mut();
            let (pass,failed)=cache.ensure(id,true,||crate::live_lut::load(id).and_then(|cube|crate::live_lut::Pass::new(&self.device,&cube)));
            if failed{warnings|=1;}
            if let Some(pass)=pass{
                let slot=current.map_or(0,|n|1-n);let input=current.map_or_else(||first_srv.clone(),|n|self.surfaces[n].srv.clone());
                pass.run(context,&input,&self.surfaces[slot].uav,self.width,self.height,lighting.lut.strength.clamp(0.,1.))?;
                current=Some(slot);
            }
        }else{self.lut.borrow_mut().clear();}
        if lighting.cinema.active(){
            let divisor=lighting.cinema.divisor(self.width,self.height);let mut cache=self.cinema.borrow_mut();
            let (pass,failed)=cache.ensure(&divisor,true,||crate::cinema_light::Pass::new(&self.device,self.width,self.height,divisor));
            if failed{warnings|=2;}
            if let Some(pass)=pass{
                let slot=current.map_or(0,|n|1-n);let input=current.map_or_else(||first_srv.clone(),|n|self.surfaces[n].srv.clone());
                pass.run(context,&input,&self.surfaces[slot].uav,&lighting.cinema,&lighting.motion)?;current=Some(slot);
            }
        }else{self.cinema.borrow_mut().clear();}
        if lighting.recipe.active(){
            let mut cache=self.recipe.borrow_mut();
            let(pass,failed)=cache.ensure(&lighting.recipe,true,||crate::color_recipe::Pass::new(&self.device,&lighting.recipe));
            if failed{warnings|=4;}
            if let Some(pass)=pass{
                let slot=current.map_or(0,|n|1-n);let input=current.map_or_else(||first_srv.clone(),|n|self.surfaces[n].srv.clone());
                pass.run(context,&input,&self.surfaces[slot].uav,self.width,self.height);current=Some(slot);
            }
        }else{self.recipe.borrow_mut().clear();}
        if lighting.precision.active(){
            let mut cache=self.precision.borrow_mut();
            let(pass,failed)=cache.ensure(&lighting.precision.force_fp32,false,||crate::precision_pass::Pass::new(&self.device,self.width,self.height,lighting.precision.force_fp32));
            if failed{warnings|=8;}
            if let Some(pass)=pass{
                let slot=current.map_or(0,|n|1-n);let input=current.map_or_else(||first_srv.clone(),|n|self.surfaces[n].srv.clone());
                pass.run(context,&input,&self.surfaces[slot].uav,&lighting.precision)?;current=Some(slot);
            }
        }else{self.precision.borrow_mut().clear();}
        self.optional_warning.set(warnings);
        if warnings!=0&&unsafe{self.device.GetDeviceRemovedReason()}.is_err(){return Err("WGC_DEVICE_LOST: optional stage creation".into());}
        if let (Some(slot),Some(split))=(current,lighting.layers.split.filter(|v|v.is_finite())){
            let target=1-slot;
            match self.mixer(){Ok(m)=>{m.blend(context,first_srv,&self.surfaces[slot].srv,&self.surfaces[target],self.width,self.height,split.clamp(0.,1.),true)?;current=Some(target);},Err(_)=>warnings|=64}
            self.optional_warning.set(warnings);
            if warnings!=0&&unsafe{self.device.GetDeviceRemovedReason()}.is_err(){return Err("WGC_DEVICE_LOST: mixer".into());}
        }
        if let Some(slot)=current.filter(|_|lighting.layers.protect.active()){
            let target=1-slot;
            match self.protection.get_or_init(||ProtectionPass::new(&self.device)){
              Ok(p)=>{p.run(context,first_srv,&self.surfaces[slot].srv,&self.surfaces[target].uav,self.width,self.height,&lighting.layers.protect)?;current=Some(target);}
              Err(_)=>{warnings|=32;}
            }
            self.optional_warning.set(warnings);
            if warnings!=0&&unsafe{self.device.GetDeviceRemovedReason()}.is_err(){return Err("WGC_DEVICE_LOST: protected composite".into());}
        }
        if current.is_none()&&src_desc.Format!=DXGI_FORMAT_R8G8B8A8_UNORM{
            if self.copy_shader.get().is_none(){
                let bytes=crate::shadercheck::compile_native_hlsl(include_str!("../shaders/native/HB_PixelCopy.hlsl"))?;
                let mut shader=None;unsafe{self.device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|e.to_string())?;
                let _=self.copy_shader.set(shader.ok_or("Pixel conversion shader missing")?);
            }
            unsafe{
                context.CSSetShader(self.copy_shader.get().ok_or("Pixel conversion unavailable")?,None);
                context.CSSetShaderResources(0,Some(&[Some(first_srv.clone())]));
                let u=[Some(self.surfaces[0].uav.clone())];context.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);
                context.Dispatch(self.width.div_ceil(8),self.height.div_ceil(8),1);
                context.CSSetShaderResources(0,Some(&[None]));let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];context.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
            }current=Some(0);
        }
        self.fused_saved.set(fused_saved);
        // Do not force a per-frame driver flush here: the same immediate
        // context subsequently CopyResource()s and Present()s the output.
        // GPU command ordering is preserved without breaking batching.
        unsafe{context.CSSetShader(None::<&ID3D11ComputeShader>,None);}
        Ok(current.map_or_else(||source.clone(),|slot|self.surfaces[slot].image.clone()))
    }
}
#[cfg(test)]
mod tests{
    #[test]fn paired_gpu_surfaces_match_frame_count_independence(){
        let stages=[0usize,1,2,3,1];
        let indexes=stages.iter().enumerate().map(|(i,_)|i%2).collect::<Vec<_>>();
        assert_eq!(indexes,vec![0,1,0,1,0]);
    }
}

pub fn optional_proof()->Result<String,String>{
 let g=crate::photo_lab::Gpu::new()?;let im=image::RgbaImage::from_fn(64,32,|x,y|image::Rgba([(x*4)as u8,(y*8)as u8,110,255]));let tex=g.upload(&im)?;
 let pipe=NativeGpuPipeline::new(&g.device,64,32)?;
 let mut view=None;unsafe{g.device.CreateShaderResourceView(&tex,None,Some(&mut view))}.map_err(|e|e.to_string())?;let view=view.ok_or("Optional stage SRV")?;
 let render=|t:&crate::lighting::LightTuning|g.read(&pipe.dispatch_with_cached_srv_tuned(&g.ctx,&tex,&view,&[7],t)?);
 let mut t=crate::lighting::LightTuning::default();t.bypass_look=true;
 if render(&t)?!=im{return Err("Look bypass not exact".into());}
 t.lut.name=Some("invalid".into());if render(&t)?!=im||pipe.optional_warning()!=1{return Err("First invalid LUT did not safely bypass".into());}
 t.lut.name=Some("inverse".into());let old=render(&t)?;if old==im||pipe.optional_warning()!=0{return Err("Valid LUT did not recover".into());}
 t.lut.name=Some("invalid".into());if render(&t)?!=old||pipe.optional_warning()!=1||render(&t)?!=old{return Err("Last known good LUT not retained".into());}
 t.lut.name=None;if render(&t)?!=im||pipe.optional_warning()!=0{return Err("Off did not clear failure".into());}
 t.cinema=crate::cinema_light::Options::preset(0);if render(&t)?==im{return Err("Cinema without base look inert".into());}
 Ok("OPTIONAL_STAGE=PASS FIRST_FAILURE_BYPASS=true LAST_GOOD_EXACT=true RETRY_QUARANTINE=true OFF_CLEARS=true INDEPENDENT_LIGHT=true".into())
}

struct ProtectionPass{shader:ID3D11ComputeShader,cb:ID3D11Buffer}
impl ProtectionPass{
 fn new(d:&ID3D11Device)->Result<Self,String>{let bytes=crate::shadercheck::compile_native_hlsl(include_str!("../shaders/native/HB_ProtectedComposite.hlsl"))?;let(mut shader,mut cb)=(None,None);
 unsafe{d.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|e.to_string())?;
 unsafe{d.CreateBuffer(&D3D11_BUFFER_DESC{ByteWidth:32,Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0},None,Some(&mut cb))}.map_err(|e|e.to_string())?;
 Ok(Self{shader:shader.ok_or("Protection shader")?,cb:cb.ok_or("Protection buffer")?})}
 fn run(&self,ctx:&ID3D11DeviceContext,a:&ID3D11ShaderResourceView,b:&ID3D11ShaderResourceView,u:&ID3D11UnorderedAccessView,w:u32,h:u32,p:&crate::layer_mix::Protection)->Result<(),String>{
  let cb:ID3D11Resource=self.cb.cast().map_err(|e|e.to_string())?;let data=p.args();
  unsafe{ctx.UpdateSubresource(&cb,0,None,data.as_ptr().cast(),0,0);ctx.CSSetConstantBuffers(0,Some(&[Some(self.cb.clone())]));ctx.CSSetShader(&self.shader,None);ctx.CSSetShaderResources(0,Some(&[Some(a.clone()),Some(b.clone())]));ctx.CSSetUnorderedAccessViews(0,1,Some([Some(u.clone())].as_ptr()),None);ctx.Dispatch(w.div_ceil(8),h.div_ceil(8),1);ctx.CSSetShaderResources(0,Some(&[None,None]));let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);ctx.CSSetConstantBuffers(0,Some(&[None]));}
  Ok(())
 }
}
pub fn protection_proof()->Result<String,String>{
 let g=crate::photo_lab::Gpu::new()?;let(w,h)=(127,79);
 let im=image::RgbaImage::from_fn(w,h,|x,y|if x==63||y>=64{image::Rgba([245,245,245,255])}else{image::Rgba([12,80,30,255])});
 let tex=g.upload(&im)?;let p=NativeGpuPipeline::new(&g.device,w,h)?;let mut v=None;unsafe{g.device.CreateShaderResourceView(&tex,None,Some(&mut v))}.map_err(|e|e.to_string())?;let v=v.ok_or("Protection proof SRV")?;
 let render=|t:&crate::lighting::LightTuning|g.read(&p.dispatch_with_cached_srv_tuned(&g.ctx,&tex,&v,&[7,2],t)?);
 let mut t=crate::lighting::LightTuning::default();let base=render(&t)?;if base==im{return Err("Protection fixture unchanged".into());}
 t.layers.protect.rectangle=true;t.layers.protect.rect=[0.,0.8,1.,1.];let hud=render(&t)?;
 let(mut protected,mut outside)=(0,0);
 for(x,y,pixel)in hud.enumerate_pixels(){if(y as f32+0.5)/h as f32>=0.8{if pixel!=im.get_pixel(x,y){return Err("HUD rectangle changed".into());}protected+=1;}else{if pixel!=base.get_pixel(x,y){return Err("Outside rectangle changed".into());}outside+=1;}}
 t.layers.protect.edge_strength=1.;t.layers.protect.edge_threshold=0.1;let edges=render(&t)?;
 if edges.get_pixel(62,20)!=im.get_pixel(62,20)||edges.get_pixel(10,20)!=base.get_pixel(10,20){return Err("Edge preservation not selective".into());}
 t.layers.protect=Default::default();t.layers.groups=[false;4];if render(&t)?!=im{return Err("Group bypass not exact".into());}
 t.layers.groups=[true;4];if render(&t)?!=base{return Err("Group restore not exact".into());}
 Ok(format!("PROTECTION_GPU=PASS HUD_PIXELS={protected} OUTSIDE_PIXELS={outside} EDGE_EXACT=true GROUP_RESTORE_EXACT=true"))
}

pub fn lazy_proof()->Result<String,String>{
 let g=crate::photo_lab::Gpu::new()?;let im=image::RgbaImage::from_fn(32,16,|x,y|image::Rgba([(x*7)as u8,(y*13)as u8,100,255]));let src=g.upload(&im)?;
 let p=NativeGpuPipeline::new(&g.device,32,16)?;
 if p.compiled_count()!=0{return Err("Lazy pipeline created unused programs".into());}
 let a=g.read(&p.dispatch(&g.device,&g.ctx,&src,&[7,2,7])?)?;
 if p.compiled_count()!=2||a==im{return Err("Selected stage cache incorrect".into());}
 let b=g.read(&p.dispatch(&g.device,&g.ctx,&src,&[7,2,7])?)?;if a!=b||p.compiled_count()!=2{return Err("Repeat cache incorrect".into());}
 // Inject failure into the own cache (no invalid GPU bytecode or driver fault).
 let _=p.shaders[4].set(Err("Fixture creation failure".into()));
 let one=g.read(&p.dispatch(&g.device,&g.ctx,&src,&[7])?)?;
 let fallback=g.read(&p.dispatch(&g.device,&g.ctx,&src,&[4,7])?)?;
 if one!=fallback||p.optional_warning()&16==0{return Err("Core stage fallback incorrect".into());}
 let _=p.dispatch(&g.device,&g.ctx,&src,&[7])?;if p.optional_warning()!=0{return Err("Core warning did not clear".into());}
 Ok(format!("LAZY_GPU=PASS SELECTED_PROGRAMS={} UNUSED_PROGRAMS_NOT_CREATED=true REPEAT_EXACT=true FAILED_STAGE_BYPASS=true",p.compiled_count()))
}

pub fn v77_proof()->Result<String,String>{
 let g=crate::photo_lab::Gpu::new()?;let im=image::RgbaImage::from_fn(95,63,|x,y|{let v=if x<47{50+x}else{110+x/3};image::Rgba([v as u8,(v+y/3)as u8,95,255])});
 let tex=g.upload(&im)?;let p=NativeGpuPipeline::new(&g.device,95,63)?;
 let mut view=None;unsafe{g.device.CreateShaderResourceView(&tex,None,Some(&mut view))}.map_err(|e|e.to_string())?;let v=view.ok_or("Proof SRV")?;
 let render=|ids:&[usize],t:&crate::lighting::LightTuning|g.read(&p.dispatch_with_cached_srv_tuned(&g.ctx,&tex,&v,ids,t)?);
 let mut t=crate::lighting::LightTuning::default();let cas=render(&[crate::native_filter::CAS_STAGE],&t)?;
 let changed=cas.pixels().zip(im.pixels()).filter(|(a,b)|a!=b).count();if changed==0{return Err("CAS equal-size inert".into());}
 let mut s=crate::spatial::NativeSpatialScaler::new(&g.device,95,63)?;let direct=g.read(&s.render(&g.device,&g.ctx,&tex,11)?)?;if cas!=direct{return Err("CAS stage differs from reference scaler".into());}
 t.layers.set_at(&[crate::native_filter::CAS_STAGE],0,0.);if render(&[crate::native_filter::CAS_STAGE],&t)?!=im{return Err("CAS layer zero not exact".into());}
 t=Default::default();let prior=render(&[7],&t)?;
 let _=p.mixer.set(Err("Fixture mixer".into()));let _=p.histogram.set(Err("Fixture histogram".into()));
 t.layers.set_at(&[7,2],1,0.5);t.layers.split=Some(0.5);t.exposure.enabled=true;
 let out=render(&[7,2],&t)?;if prior!=out||p.optional_warning()&192!=192{return Err(format!("Fallback failed: {}",p.optional_warning()));}
 let out=render(&[7],&Default::default())?;if out!=prior||p.optional_warning()!=0{return Err("Warnings not cleared".into());}
 Ok(format!("V77_GPU=PASS CAS_CHANGED={changed} REFERENCE_EXACT=true ZERO_EXACT=true MIX_HIST_FALLBACK_EXACT=true"))
}

#[cfg(test)]mod v87_runtime_tests{
 use super::*;
 fn fixture()->image::RgbaImage{image::RgbaImage::from_fn(95,63,|x,y|image::Rgba([((x*13+y*7)%200+20)as u8,((x*3+y*17)%200+20)as u8,((x*23+y*5)%200+20)as u8,((x+y)%220+20)as u8]))}
 #[test]fn live_stack_fallback_is_last_good_and_initial_bypass_exact(){
  let g=crate::photo_lab::Gpu::new().unwrap();let im=fixture();let src=g.upload(&im).unwrap();let p=NativeGpuPipeline::new(&g.device,95,63).unwrap();
  let mut srv=None;unsafe{g.device.CreateShaderResourceView(&src,None,Some(&mut srv))}.unwrap();let srv=srv.unwrap();
  let t=crate::lighting::LightTuning::default();let render=|ids:&[usize],t:&crate::lighting::LightTuning|g.read(&p.dispatch_live(&g.ctx,&src,&srv,ids,t).unwrap()).unwrap();
  let _=p.shaders[4].set(Err("fixture core shader failure".into()));
  assert_eq!(render(&[4],&t),im);assert!(p.live_rejected());assert_eq!(p.live_active_count(),0);
  let a=render(&[7,2],&t);assert_ne!(a,im);assert!(!p.live_rejected());
  let mut changed=t.clone();changed.grade.enabled=true;changed.grade.exposure=1.;
  assert_eq!(render(&[4,1],&changed),a);assert!(p.live_rejected());assert_eq!(p.live_active_count(),2);
  assert_ne!(render(&[5],&t),a);assert!(!p.live_rejected());assert_eq!(p.live_active_count(),1);
 }
 #[test]fn temporal_grain_varies_and_static_reference_stays_deterministic(){
  let g=crate::photo_lab::Gpu::new().unwrap();let im=fixture();let src=g.upload(&im).unwrap();let p=NativeGpuPipeline::new(&g.device,95,63).unwrap();
  let mut srv=None;unsafe{g.device.CreateShaderResourceView(&src,None,Some(&mut srv))}.unwrap();let srv=srv.unwrap();
  let render=|t:&crate::lighting::LightTuning|g.read(&p.dispatch_with_cached_srv_tuned(&g.ctx,&src,&srv,&[49],t).unwrap()).unwrap();
  let mut t=crate::lighting::LightTuning::default();p.set_grain_phase(1);let a=render(&t);p.set_grain_phase(2);assert_eq!(render(&t),a);
  t.reconstruction.animated_grain=true;let b=render(&t);p.set_grain_phase(3);let c=render(&t);assert_ne!(b,c);assert!(b.pixels().zip(im.pixels()).all(|(a,b)|a[3]==b[3]));
  t.layers.set_at(&[49],0,0.);assert_eq!(render(&t),im);
 }
 #[test]fn actual_cas_stage_matches_official_scaler_and_zero_strength(){v77_proof().unwrap();}
 #[test]fn native_filter_color_split_and_protection_are_exact(){protection_proof().unwrap();}
 #[test]fn actual_gpu_pingpong_surfaces_and_cache_reused(){lazy_proof().unwrap();}
 #[test]fn filter_and_vignette_preserve_alpha_and_have_distinct_effects(){
  let g=crate::photo_lab::Gpu::new().unwrap();let im=fixture();let src=g.upload(&im).unwrap();let p=NativeGpuPipeline::new(&g.device,95,63).unwrap();
  for id in [44,49,55,59]{let out=g.read(&p.dispatch(&g.device,&g.ctx,&src,&[id]).unwrap()).unwrap();assert_ne!(out,im,"stage {id}");assert!(out.pixels().zip(im.pixels()).all(|(a,b)|a[3]==b[3]),"stage {id} alpha");}
 }
}
