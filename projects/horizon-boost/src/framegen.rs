//! Experimental capture-based GPU interpolation, not game-engine FG.
 //! Only processes two pre-existing external D3D11 RGBA textures.
 //! x2/x3 output affects only Horizon Boost's OWN swapchain, never the game's.
use windows::{
    core::Interface,
    Win32::Graphics::{
        Direct3D11::{
            ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,
            ID3D11ComputeShader,ID3D11ClassLinkage,ID3D11ShaderResourceView,
            ID3D11UnorderedAccessView,D3D11_TEXTURE2D_DESC,ID3D11Buffer,D3D11_BUFFER_DESC,ID3D11Resource,D3D11_BIND_CONSTANT_BUFFER,
            D3D11_USAGE_DEFAULT,D3D11_BIND_SHADER_RESOURCE,D3D11_BIND_UNORDERED_ACCESS,
        },
        Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_FORMAT_R32G32B32A32_FLOAT,DXGI_FORMAT_R32_FLOAT,
            DXGI_SAMPLE_DESC},
    },
};
const FLOW_ESTIMATE:&str=include_str!("../shaders/native/HB_FlowTile.hlsl");
const FLOW_WARP:&str=include_str!("../shaders/native/HB_FlowWarp.hlsl");
const SCENE_CUT:&str=include_str!("../shaders/native/HB_SceneCut.hlsl");
pub fn auto_tile(width:u32,height:u32)->u32{
    if (width as u64)*(height as u64)>1920*1080{32}else{16}
}
pub fn field_dimensions(width:u32,height:u32,tile:u32)->(u32,u32){
    let tile=if tile==32{32}else{16};(width.div_ceil(tile),height.div_ceil(tile))
}
pub const FACTORS:&[u8]=&[1,2,3];
/// Capture-only safeguard: never queue multiple synthetic frames when real
/// frame pacing is already slow. This is NOT game input-lag telemetry; it
/// merely protects our own present queue from extra work on Polaris.
pub fn should_synthesize(multiplier:u8,real_interval_ms:f32)->bool{
    if !real_interval_ms.is_finite()||real_interval_ms<=0.0{return false;}
    match multiplier{
        2=>real_interval_ms<=23.0,
        3=>real_interval_ms<=15.5,
        _=>false
    }
}

/// v2.3 capture-only guard: opt-in synthesis must not turn a 25 FPS
/// WGC stream into a heavier 25 FPS output with extra frame queue pressure.
/// This is an output-side performance safeguard, NOT game input latency.
#[derive(Clone,Debug)]
pub struct CapturePacingGuard{
    samples:[f32;12],
    received:usize,
    cursor:usize,
}
impl Default for CapturePacingGuard{
    fn default()->Self{Self{samples:[0.0;12],received:0,cursor:0}}
}
impl CapturePacingGuard{
    pub fn observe(&mut self,multiplier:u8,interval_ms:f32)->bool{self.observe_factor(multiplier,interval_ms,false)>1}
    pub fn observe_factor(&mut self,multiplier:u8,interval_ms:f32,adaptive:bool)->u8{
        if !interval_ms.is_finite()||!(0.4..=350.0).contains(&interval_ms){
            self.received=0;self.cursor=0;
            return 1;
        }
        self.samples[self.cursor]=interval_ms;
        self.cursor=(self.cursor+1)%self.samples.len();
        self.received=(self.received+1).min(self.samples.len());
        // Wait for a full short rolling window; a single lucky frame is not
        // evidence that compute flow has enough GPU headroom.
        if self.received<self.samples.len(){return 1;}
        let sum=self.samples.iter().sum::<f32>();
        let average=sum/self.samples.len() as f32;
        let can2=average<=19.0&&self.samples.iter().all(|&v|v<=24.0);
        let can3=average<=11.0&&self.samples.iter().all(|&v|v<=14.0);
        match multiplier{3 if can3=>3,3 if adaptive&&can2=>2,2 if can2=>2,_=>1}
    }
}
pub fn validate_factor(factor:u8)->Result<u8,String>{
    if FACTORS.contains(&factor){Ok(factor)}
    else{Err("Only Off, experimental 2x or experimental 3x is available".into())}
}
fn shader(device:&ID3D11Device,code:&str)->Result<ID3D11ComputeShader,String>{
    let bytes=crate::shadercheck::compile_native_hlsl(code)?;
    let mut output=None;
    unsafe{device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,
        Some(&mut output))}.map_err(|e|format!("FG compile native compute: {e}"))?;
    output.ok_or("No FG compute program".into())
}
pub fn compile_all()->Result<usize,String>{
    crate::shadercheck::compile_native_hlsl(FLOW_ESTIMATE)?;
    crate::shadercheck::compile_native_hlsl(SCENE_CUT)?;
    for x in [0.5f32,1.0/3.0,2.0/3.0]{
        let code=format!("#define FLOW_ALPHA {:.7}\n{FLOW_WARP}",x);
        crate::shadercheck::compile_native_hlsl(&code)?;
    }
    Ok(5)
}
pub struct FlowInterpolator{
    width:u32,height:u32,tile:u32,
    estimate:ID3D11ComputeShader,
    pyramid:Option<crate::flow_pyramid::Pyramid>,
    scene_detector:ID3D11ComputeShader,
    interpolators:[ID3D11ComputeShader;3],
    motion_srv:ID3D11ShaderResourceView,
    motion_uav:ID3D11UnorderedAccessView,
    scene_srv:ID3D11ShaderResourceView,
    scene_uav:ID3D11UnorderedAccessView,
    scene_texture:ID3D11Texture2D,
    output:ID3D11Texture2D,
    output_uav:ID3D11UnorderedAccessView,
    previous:ID3D11Texture2D,
    previous_srv:ID3D11ShaderResourceView,
    motion_constants:ID3D11Buffer,
    current_view:std::cell::RefCell<crate::reconstruction::ViewCache>,
}
impl FlowInterpolator{
    pub fn new(device:&ID3D11Device,width:u32,height:u32)->Result<Self,String>{
        Self::new_with_tile(device,width,height,auto_tile(width,height))
    }
    pub fn new_with_tile(device:&ID3D11Device,width:u32,height:u32,tile:u32)->Result<Self,String>{
        Self::new_with_pyramid(device,width,height,tile,false)
    }
    pub fn new_with_pyramid(device:&ID3D11Device,width:u32,height:u32,tile:u32,enable:bool)->Result<Self,String>{
        if width==0||height==0||width>8192||height>8192{return Err("Bad FG capture size".into());}
        if !matches!(tile,16|32){return Err("Flow tile must be 16 or 32".into());}
        let variant=|code:&str|format!("#define FLOW_TILE {tile}\n{code}");
        let estimate=shader(device,&variant(FLOW_ESTIMATE))?;
        let pyramid=if enable{Some(crate::flow_pyramid::Pyramid::new(device,width,height,tile)?)}else{None};
        let scene_detector=shader(device,&variant(SCENE_CUT))?;
        let programs=[0.5f32,1.0/3.0,2.0/3.0].map(|x|{
            shader(device,&format!("#define FLOW_TILE {tile}\n#define FLOW_ALPHA {:.7}\n{FLOW_WARP}",x))
        });
        let interpolators=[
            programs[0].as_ref().map_err(Clone::clone)?.clone(),
            programs[1].as_ref().map_err(Clone::clone)?.clone(),
            programs[2].as_ref().map_err(Clone::clone)?.clone()
        ];
        let flow_desc=D3D11_TEXTURE2D_DESC{
            Width:field_dimensions(width,height,tile).0,Height:field_dimensions(width,height,tile).1,
            MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R32G32B32A32_FLOAT,
            SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
            Usage:D3D11_USAGE_DEFAULT,
            BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,
            CPUAccessFlags:0,MiscFlags:0,
        };
        let mut flow=None;
        unsafe{device.CreateTexture2D(&flow_desc,None,Some(&mut flow))}
            .map_err(|e|format!("Motion-field surface: {e}"))?;
        let flow=flow.ok_or("No flow field")?;
        let mut motion_srv=None;let mut motion_uav=None;
        unsafe{
            device.CreateShaderResourceView(&flow,None,Some(&mut motion_srv))
                .map_err(|e|format!("Flow SRV: {e}"))?;
            device.CreateUnorderedAccessView(&flow,None,Some(&mut motion_uav))
                .map_err(|e|format!("Flow UAV: {e}"))?;
        }
        // Independent forward/backward motion is RGBA32F per tile.
        // One scalar 1x1 GPU flag rejects scene cuts without CPU-GPU stalls.
        let scene_desc=D3D11_TEXTURE2D_DESC{
            Width:1,Height:1,Format:DXGI_FORMAT_R32_FLOAT,
            ..flow_desc
        };
        let mut scene=None;
        unsafe{device.CreateTexture2D(&scene_desc,None,Some(&mut scene))}
            .map_err(|e|format!("FG GPU cut flag texture: {e}"))?;
        let scene=scene.ok_or("No GPU scene cut surface")?;
        let mut scene_srv=None;let mut scene_uav=None;
        unsafe{
            device.CreateShaderResourceView(&scene,None,Some(&mut scene_srv))
                .map_err(|e|format!("FG cut SRV: {e}"))?;
            device.CreateUnorderedAccessView(&scene,None,Some(&mut scene_uav))
                .map_err(|e|format!("FG cut UAV: {e}"))?;
        }
        let scene_srv=scene_srv.ok_or("No cut SRV")?;
        let scene_uav=scene_uav.ok_or("No cut UAV")?;
        let image_desc=D3D11_TEXTURE2D_DESC{
            Width:width,Height:height,Format:DXGI_FORMAT_R8G8B8A8_UNORM,
            ..flow_desc
        };
        let create=||->Result<ID3D11Texture2D,String>{
            let mut texture=None;
            unsafe{device.CreateTexture2D(&image_desc,None,Some(&mut texture))}
                .map_err(|e|format!("Frame interpolation RGBA GPU surface: {e}"))?;
            texture.ok_or("FG RGBA surface missing".into())
        };
        let output=create()?;
        let previous=create()?;
        let mut previous_srv=None;
        unsafe{device.CreateShaderResourceView(&previous,None,Some(&mut previous_srv))}
            .map_err(|e|format!("Cached FG previous-frame view: {e}"))?;
        let previous_srv=previous_srv.ok_or("Cached previous-frame SRV missing")?;
        let mut output_uav=None;
        unsafe{device.CreateUnorderedAccessView(&output,None,Some(&mut output_uav))}
            .map_err(|e|format!("Frame generation UAV: {e}"))?;
        let mut motion_constants=None;
        unsafe{device.CreateBuffer(&D3D11_BUFFER_DESC{ByteWidth:48,Usage:D3D11_USAGE_DEFAULT,
            BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0},None,Some(&mut motion_constants))}
            .map_err(|e|format!("Motion options cbuffer: {e}"))?;
        Ok(Self{
            motion_constants:motion_constants.ok_or("Motion constant buffer missing")?,current_view:Default::default(),
            width,height,tile,estimate,pyramid,scene_detector,interpolators,
            motion_srv:motion_srv.ok_or("Motion field SRV missing")?,
            motion_uav:motion_uav.ok_or("Motion field UAV missing")?,
            scene_srv,scene_uav,scene_texture:scene,
            output,output_uav:output_uav.ok_or("FG output UAV missing")?,previous,
            previous_srv
        })
    }
    pub fn tile(&self)->u32{self.tile}
    pub fn pyramid_active(&self)->bool{self.pyramid.is_some()}
    pub fn save_real_frame(&self,ctx:&ID3D11DeviceContext,latest:&ID3D11Texture2D){
        let dst:windows::Win32::Graphics::Direct3D11::ID3D11Resource=self.previous.cast().unwrap();
        let src:windows::Win32::Graphics::Direct3D11::ID3D11Resource=latest.cast().unwrap();
        unsafe{ctx.CopyResource(&dst,&src);}
    }
    pub fn estimate(&self,device:&ID3D11Device,ctx:&ID3D11DeviceContext,
        current:&ID3D11Texture2D)->Result<(ID3D11ShaderResourceView,ID3D11ShaderResourceView),String>{
        let previous=self.previous_srv.clone();
        let now=self.current_view.borrow_mut().get(device,current)?;
        if let Some(p)=self.pyramid.as_ref(){p.prepare(ctx,&previous,&now);}
        let inputs=[Some(previous.clone()),Some(now.clone()),self.pyramid.as_ref().map(|p|p.seed())];
        let uavs=[Some(self.motion_uav.clone())];
        unsafe{
            ctx.CSSetShader(self.pyramid.as_ref().map_or(&self.estimate,|p|&p.refine),None);
            ctx.CSSetShaderResources(0,Some(&inputs));
            ctx.CSSetUnorderedAccessViews(0,1,Some(uavs.as_ptr()),None);
            ctx.Dispatch(self.width.div_ceil(self.tile).div_ceil(8),
                self.height.div_ceil(self.tile).div_ceil(8),1);
            ctx.CSSetShaderResources(0,Some(&[None,None,None]));
            let clear=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
            // Source motion field is no longer an unordered-access target.
            // Enqueue GPU-only 8x8 distributed scene-change estimation.
            ctx.CSSetShader(&self.scene_detector,None);
            let detect=[Some(previous.clone()),Some(now.clone()),Some(self.motion_srv.clone())];
            ctx.CSSetShaderResources(0,Some(&detect));
            let cut=[Some(self.scene_uav.clone())];
            ctx.CSSetUnorderedAccessViews(0,1,Some(cut.as_ptr()),None);
            ctx.Dispatch(1,1,1);
            ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
            ctx.CSSetShaderResources(0,Some(&[None,None,None]));
            ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
        }
        Ok((previous,now))
    }
    /// Î± = 0.5 for 2Ă—, or 1/3 and 2/3 for 3Ă—.
    /// Returns own temporary GPU texture; caller presents before next warp.
    pub fn synth(&self,ctx:&ID3D11DeviceContext,
        inputs:&(ID3D11ShaderResourceView,ID3D11ShaderResourceView),
        factor:u8,subframe:u8)->Result<&ID3D11Texture2D,String>{
        self.synth_tuned(ctx,inputs,factor,subframe,&crate::motion_options::Options::default(),0)
    }
    pub fn synth_tuned(&self,ctx:&ID3D11DeviceContext,
        inputs:&(ID3D11ShaderResourceView,ID3D11ShaderResourceView),factor:u8,subframe:u8,
        options:&crate::motion_options::Options,diagnostic:u8)->Result<&ID3D11Texture2D,String>{
        let shader_index=match(factor,subframe){
            (2,1)=>0,(3,1)=>1,(3,2)=>2,_=>return Err("Invalid FG interpolation phase".into())
        };
        self.dispatch_warp(ctx,inputs,shader_index,options.gpu(diagnostic))
    }
    pub fn temporal_tuned(&self,ctx:&ID3D11DeviceContext,
        inputs:&(ID3D11ShaderResourceView,ID3D11ShaderResourceView),
        options:&crate::motion_options::Options)->Result<&ID3D11Texture2D,String>{
        if !(1..=2).contains(&options.temporal_mode){return Err("Select an explicit temporal mode".into());}
        self.dispatch_warp(ctx,inputs,0,options.temporal_gpu())
    }
    fn dispatch_warp(&self,ctx:&ID3D11DeviceContext,
        inputs:&(ID3D11ShaderResourceView,ID3D11ShaderResourceView),shader_index:usize,
        params:crate::motion_options::Params)->Result<&ID3D11Texture2D,String>{
        let buffer:ID3D11Resource=self.motion_constants.cast().map_err(|e|e.to_string())?;
        let srvs=[
            Some(inputs.0.clone()),Some(inputs.1.clone()),
            Some(self.motion_srv.clone()),Some(self.scene_srv.clone())
        ];
        let uavs=[Some(self.output_uav.clone())];
        unsafe{
            ctx.UpdateSubresource(&buffer,0,None,&params as *const _ as *const std::ffi::c_void,0,0);
            ctx.CSSetConstantBuffers(0,Some(&[Some(self.motion_constants.clone())]));
            ctx.CSSetShader(&self.interpolators[shader_index],None);
            ctx.CSSetShaderResources(0,Some(&srvs));
            ctx.CSSetUnorderedAccessViews(0,1,Some(uavs.as_ptr()),None);
            ctx.Dispatch(self.width.div_ceil(8),self.height.div_ceil(8),1);
            ctx.CSSetShaderResources(0,Some(&[None,None,None,None]));
            ctx.CSSetConstantBuffers(0,Some(&[None]));
            let clear=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
            ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
        }
        Ok(&self.output)
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn real_gpu_motion_programs_compile(){
        assert_eq!(compile_all().unwrap(),5);
        assert_eq!(FACTORS,&[1,2,3]);
        assert!(validate_factor(4).is_err());
    }
    #[test]fn low_latency_suppresses_heavy_generation_on_slow_capture(){
        assert!(!should_synthesize(1,8.0));
        assert!(should_synthesize(2,16.0));
        assert!(!should_synthesize(2,32.0));
        assert!(should_synthesize(3,13.0));
        assert!(!should_synthesize(3,20.0));
        assert!(!should_synthesize(2,f32::NAN));
    }
    #[test]fn guard_refuses_poor_capture_frame_pacing(){
        let mut guard=CapturePacingGuard::default();
        for _ in 0..60{assert!(!guard.observe(2,40.0));}
        for i in 0..12 {
            let yes=guard.observe(2,16.0);
            assert_eq!(yes,i==11);
        }
        assert!(!guard.observe(2,60.0));
        let mut g3=CapturePacingGuard::default();
        for _ in 0..12{g3.observe(3,8.0);}
        assert!(g3.observe(3,8.0));
        assert!(!g3.observe(3,16.0));
        assert!(!g3.observe(3,f32::NAN));
    }

    #[test]
    fn real_radeon_scene_cut_compute_rejects_full_image_replacement(){
        use windows::Win32::Foundation::HMODULE;
        use windows::Win32::Graphics::{
            Direct3D::{D3D_DRIVER_TYPE_HARDWARE,D3D_FEATURE_LEVEL_11_0},
            Direct3D11::{
                D3D11CreateDevice,D3D11_CREATE_DEVICE_BGRA_SUPPORT,D3D11_SDK_VERSION,
                D3D11_SUBRESOURCE_DATA,D3D11_USAGE_STAGING,D3D11_CPU_ACCESS_READ,
                D3D11_MAPPED_SUBRESOURCE,D3D11_MAP_READ
            },
            Dxgi::{IDXGIAdapter,Common::DXGI_SAMPLE_DESC},
        };
        const DIM:u32=64;
        let mut dev=None;let mut ctx=None;
        unsafe{D3D11CreateDevice(None::<&IDXGIAdapter>,D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&[D3D_FEATURE_LEVEL_11_0]),D3D11_SDK_VERSION,
            Some(&mut dev),None,Some(&mut ctx))}.unwrap();
        let dev=dev.unwrap();let ctx=ctx.unwrap();
        let fg=FlowInterpolator::new(&dev,DIM,DIM).unwrap();
        let desc=D3D11_TEXTURE2D_DESC{
            Width:DIM,Height:DIM,MipLevels:1,ArraySize:1,
            Format:DXGI_FORMAT_R8G8B8A8_UNORM,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
            Usage:D3D11_USAGE_DEFAULT,
            BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,
            CPUAccessFlags:0,MiscFlags:0
        };
        let make=|color:u8|->ID3D11Texture2D{
            let mut pixels=vec![color;(DIM*DIM*4) as usize];
            for px in pixels.chunks_exact_mut(4){px[3]=255;}
            let init=D3D11_SUBRESOURCE_DATA{
                pSysMem:pixels.as_ptr() as *const _,
                SysMemPitch:DIM*4,SysMemSlicePitch:DIM*DIM*4
            };
            let mut t=None;
            unsafe{dev.CreateTexture2D(&desc,Some(&init),Some(&mut t))}.unwrap();
            t.unwrap()
        };
        let black=make(0);
        let white=make(255);
        let flag_desc=D3D11_TEXTURE2D_DESC{
            Width:1,Height:1,MipLevels:1,ArraySize:1,
            Format:DXGI_FORMAT_R32_FLOAT,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
            Usage:D3D11_USAGE_STAGING,BindFlags:0,
            CPUAccessFlags:D3D11_CPU_ACCESS_READ.0 as u32,MiscFlags:0,
        };
        let mut staging=None;
        unsafe{dev.CreateTexture2D(&flag_desc,None,Some(&mut staging))}.unwrap();
        let staging=staging.unwrap();
        let target:windows::Win32::Graphics::Direct3D11::ID3D11Resource=
            staging.cast().unwrap();
        let result:windows::Win32::Graphics::Direct3D11::ID3D11Resource=
            fg.scene_texture.cast().unwrap();
        let read=|latest:&ID3D11Texture2D|->f32{
            fg.save_real_frame(&ctx,&black);
            let _inputs=fg.estimate(&dev,&ctx,latest).unwrap();
            unsafe{ctx.CopyResource(&target,&result);}
            crate::native_filter::verify_gpu_completion(&dev,&ctx).unwrap();
            let mut mapped=D3D11_MAPPED_SUBRESOURCE::default();
            unsafe{ctx.Map(&target,0,D3D11_MAP_READ,0,Some(&mut mapped))}.unwrap();
            let value=unsafe{*(mapped.pData as *const f32)};
            unsafe{ctx.Unmap(&target,0);}
            value
        };
        let identical=read(&black);
        let changed=read(&white);
        assert!(identical<=0.01,"Same scene incorrectly detected as cut: {identical}");
        assert!(changed>=0.99,"Disjoint black->white scene cut not flagged: {changed}");
    }
    #[test]fn tiled_resolution_is_bounded(){
        for dim in [1u32,15,16,17,1919,1920,8192]{
            let tiles=dim.div_ceil(auto_tile(dim,dim));
            let tile=auto_tile(dim,dim);assert!(tiles>0&&tiles*tile>=dim&&(tiles-1)*tile<dim);
        }
    }
}

#[cfg(test)]mod adaptive_grid_tests{
 #[test]fn adaptive_grid_and_odd_size(){assert_eq!(super::auto_tile(1920,1080),16);assert_eq!(super::auto_tile(2560,1440),32);assert_eq!(super::field_dimensions(321,181,32),(11,6));assert_eq!(super::field_dimensions(2560,1440,32),(80,45));}
}

pub fn stale_for_fg(age:Option<f64>,limit:f32)->bool{age.is_some_and(|v|!v.is_finite()||v>f64::from(limit.clamp(25.,250.)))}
#[cfg(test)]mod adaptive_factor_tests{use super::*;#[test]fn fallback_requires_full_measured_window(){let mut g=CapturePacingGuard::default();for _ in 0..11{assert_eq!(g.observe_factor(3,16.,true),1);}assert_eq!(g.observe_factor(3,16.,true),2);assert_eq!(g.observe_factor(3,16.,false),1);for _ in 0..12{g.observe_factor(3,9.,true);}assert_eq!(g.observe_factor(3,9.,true),3);assert_eq!(g.observe_factor(3,30.,true),1);assert_eq!(g.observe_factor(3,f32::NAN,true),1);assert!(stale_for_fg(Some(100.),85.));assert!(!stale_for_fg(None,85.));}}

impl CapturePacingGuard{
    // Stable real capture timing plus nominal output refresh. This is not scanout proof.
    pub fn observe_for_display(&mut self,requested:u8,source_ms:Option<f32>,wall_ms:f32,refresh:Option<u32>,adaptive:bool)->u8{
        let Some(ms)=source_ms.filter(|v|v.is_finite()&&(5.0..=45.0).contains(v)) else{*self=Self::default();return 1;};
        if !wall_ms.is_finite()||wall_ms>ms*1.55+3.0{*self=Self::default();return 1;}
        let Some(hz)=refresh.filter(|v|(24..=360).contains(v)) else{*self=Self::default();return 1;};
        self.samples[self.cursor]=ms;self.cursor=(self.cursor+1)%self.samples.len();
        self.received=(self.received+1).min(self.samples.len());
        if self.received<self.samples.len(){return 1;}
        let avg=self.samples.iter().sum::<f32>()/self.samples.len() as f32;
        if self.samples.iter().any(|v|(*v-avg).abs()>avg*0.25){return 1;}
        let fits=|n:u8| (1000.0/avg)*n as f32<=hz as f32*1.035;
        match requested{3 if fits(3)=>3,3 if adaptive&&fits(2)=>2,2 if fits(2)=>2,_=>1}
    }
}
#[cfg(test)]mod display_admission_tests{use super::*;
 #[test]fn thirty_to_sixty_and_sixty_to_120_only_if_stable(){for (ms,hz) in [(33.333,60),(16.667,120)]{let mut g=CapturePacingGuard::default();for _ in 0..11{assert_eq!(g.observe_for_display(2,Some(ms),ms,Some(hz),true),1);}assert_eq!(g.observe_for_display(2,Some(ms),ms,Some(hz),true),2);}}
 #[test]fn no_extra_frames_without_refresh_headroom(){let mut g=CapturePacingGuard::default();for _ in 0..20{assert_eq!(g.observe_for_display(2,Some(16.667),17.,Some(60),true),1);}assert_eq!(g.observe_for_display(2,None,16.,Some(120),true),1);}
 #[test]fn drops_for_slow_wall_work_and_jitter(){let mut g=CapturePacingGuard::default();for _ in 0..12{g.observe_for_display(2,Some(33.333),33.333,Some(60),true);}assert_eq!(g.observe_for_display(2,Some(33.333),80.,Some(60),true),1);for i in 0..20{assert_eq!(g.observe_for_display(2,Some(if i%2==0{20.}else{40.}),40.,Some(120),true),1);}}
}

#[cfg(test)]mod v99_disocclusion_proof{
 use super::*;
 #[test]fn color_only_occlusion_guard_changes_synthesized_pixels_on_real_gpu(){
  let gpu=crate::photo_lab::Gpu::new().unwrap();
  const W:u32=97;const H:u32=69;
  let make=|shift:i32|image::RgbaImage::from_fn(W,H,|x,y|{
    let background=if ((x/7+y/9)%2)==0{43}else{70};
    let inside=(x as i32>=23+shift&&(x as i32)<45+shift)&&(y>=20&&y<53);
    if inside{image::Rgba([235,54,30,255])}else{
      image::Rgba([background,((x*7+y*3)%65+65)as u8,background+15,255])
    }
  });
  let (a,b)=(make(0),make(11));
  let ta=gpu.upload(&a).unwrap();
  let tb=gpu.upload(&b).unwrap();
  let fg=FlowInterpolator::new(&gpu.device,W,H).unwrap();
  fg.save_real_frame(&gpu.ctx,&ta);
  let pair=fg.estimate(&gpu.device,&gpu.ctx,&tb).unwrap();
  let mut config=crate::motion_options::Options::default();
  config.disocclusion_guard=0.;
  let legacy=gpu.read(fg.synth_tuned(&gpu.ctx,&pair,2,1,&config,0).unwrap()).unwrap();
  config.disocclusion_guard=1.;
  let guarded=gpu.read(fg.synth_tuned(&gpu.ctx,&pair,2,1,&config,0).unwrap()).unwrap();
  assert_eq!(legacy.dimensions(),(W,H));assert_eq!(guarded.dimensions(),(W,H));
  assert!(guarded.pixels().all(|p|p[3]==255));
  let pixels=legacy.pixels().zip(guarded.pixels()).filter(|(p,q)|p!=q).count();
  assert!(pixels>=5,"Occlusion guard had no measurable GPU effect in moving occluded image ({pixels} pixels)");
  // Compare the newly exposed region against the latest real frame.
  let compare=|im:&image::RgbaImage|{
    let mut error=0u64;
    for y in 23..49{for x in 23..34{
      let p=im.get_pixel(x,y);
      let truth=b.get_pixel(x,y);
      for c in 0..3{error+=u64::from(p[c].abs_diff(truth[c]));}
    }}error
  };
  let old_error=compare(&legacy);let guarded_error=compare(&guarded);
  assert!(guarded_error<=old_error,
    "Guard worsened newly revealed source pixels (old {old_error}, guarded {guarded_error})");
 }
 #[test]fn old_motion_profiles_get_safe_guard_and_clamp_bad_values(){
  let old=r#"{"consistency":1.0,"max_motion":12.0}"#;
  let o:crate::motion_options::Options=serde_json::from_str(old).unwrap();
  assert_eq!(o.disocclusion_guard,0.8);
  let mut o=o;o.disocclusion_guard=f32::NAN;o.clamp();assert_eq!(o.disocclusion_guard,0.8);
  o.disocclusion_guard=0.;assert!(o.disocclusion_guard==0.);
 }
}

