//! Native Direct3D11 compute shaders for independent GPU postprocessing.
//! Does NOT use Magpie, inject into a game, change its window style, or claim
//! that the resulting GPU textures are presented over exclusive fullscreen.
//! All image processing here stays in VRAM; no CPU pixel readback.
use windows::{
    Win32::{Graphics::{
        Direct3D11::{
            D3D11_BIND_SHADER_RESOURCE,D3D11_BIND_UNORDERED_ACCESS,D3D11_TEXTURE2D_DESC,
            D3D11_USAGE_DEFAULT,ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,
            ID3D11ShaderResourceView,ID3D11UnorderedAccessView,
            ID3D11ClassLinkage,ID3D11ComputeShader,ID3D11Query,D3D11_QUERY_DESC,D3D11_QUERY_EVENT,
        },
        Dxgi::Common::DXGI_FORMAT_R8G8B8A8_UNORM,
    }},
};
pub const PROGRAMS:&[(&str,&str)]=&[
    ("Adaptive Vibrance",include_str!("../shaders/native/HB_NativeVibrance.hlsl")),
    ("Filmic Color",include_str!("../shaders/native/HB_NativeFilmic.hlsl")),
    ("Clarity Detail",include_str!("../shaders/native/HB_NativeClarity.hlsl")),
    ("Night Vision",include_str!("../shaders/native/HB_NativeNightVision.hlsl")),
    ("Shadow Boost",include_str!("../shaders/native/HB_NativeShadowBoost.hlsl")),
    ("Warm Sunset",include_str!("../shaders/native/HB_NativeWarmSunset.hlsl")),
    ("Moonlight Blue",include_str!("../shaders/native/HB_NativeMoonlight.hlsl")),
    ("Mono Noir",include_str!("../shaders/native/HB_NativeMonoNoir.hlsl")),
    ("Vintage Sepia",include_str!("../shaders/native/HB_NativeSepia.hlsl")),
    ("Color Assist",include_str!("../shaders/native/HB_NativeColorAssist.hlsl")),
    ("Retro Posterize",include_str!("../shaders/native/HB_NativePosterize.hlsl")),
    ("Cool Clean",include_str!("../shaders/native/HB_NativeCoolClean.hlsl")),
    ("Prism Bloom",include_str!("../shaders/native/HB_PrismBloom.hlsl")),
    ("Pathlight Shafts (screen-space)",include_str!("../shaders/native/HB_PathlightShafts.hlsl")),
    ("Color Bounce (2D approximation)",include_str!("../shaders/native/HB_BounceIllusion.hlsl")),
    ("Contact Shade (2D edges)",include_str!("../shaders/native/HB_ContactShade.hlsl")),
    ("Neon Rim",include_str!("../shaders/native/HB_NeonEdge.hlsl")),
    ("Cel Comic",include_str!("../shaders/native/HB_CelComic.hlsl")),
    ("Watercolor Wash",include_str!("../shaders/native/HB_Watercolor.hlsl")),
    ("Phosphor CRT",include_str!("../shaders/native/HB_PhosphorCRT.hlsl")),
    ("VHS Analog",include_str!("../shaders/native/HB_VHSAnalog.hlsl")),
    ("Anamorphic Streak",include_str!("../shaders/native/HB_Anamorphic.hlsl")),
    ("Frosted Lens",include_str!("../shaders/native/HB_FrostLens.hlsl")),
    ("Chromatic Lens",include_str!("../shaders/native/HB_ChromaticLens.hlsl")),
    ("Heat Mirage",include_str!("../shaders/native/HB_HeatMirage.hlsl")),
    ("Hologram Grid",include_str!("../shaders/native/HB_HoloGrid.hlsl")),
    ("Oil Brush",include_str!("../shaders/native/HB_OilBrush.hlsl")),
    ("Tilt Focus",include_str!("../shaders/native/HB_TiltLens.hlsl")),
    ("Physically Soft Glow",include_str!("../shaders/native/HB_RealBloom.hlsl")),
    ("Local Contact Shadow (2D)",include_str!("../shaders/native/HB_SoftContact.hlsl")),
    ("Indirect Color Spill (2D)",include_str!("../shaders/native/HB_AmbientSpill.hlsl")),
    ("Wide Latitude SDR Tone",include_str!("../shaders/native/HB_FilmLatitude.hlsl")),
    ("Conservative Adaptive Detail",include_str!("../shaders/native/HB_EdgeAdaptive.hlsl")),
    ("Wet Light Sheen (2D)",include_str!("../shaders/native/HB_WetHighlights.hlsl")),
    ("Filmic Halation",include_str!("../shaders/native/HB_HalationSoft.hlsl")),
    ("Color Edge Smoothing",include_str!("../shaders/native/HB_SoftAA.hlsl")),
    ("Lighting Studio",include_str!("../shaders/native/HB_LightingStudio.hlsl")),
    ("Edge-Aware Bilateral Denoise",include_str!("../shaders/native/HB_BilateralDenoise.hlsl")),
    ("SDR Subtle Deband Dither",include_str!("../shaders/native/HB_SDRDebandDither.hlsl")),
    ("Contrast-aware Sharpen (original)",include_str!("../shaders/native/HB_AdaptiveContrastSharp.hlsl")),
    ("Directional Edge AA (original)",include_str!("../shaders/native/HB_DirectionalEdgeAA.hlsl")),
    ("Local SDR Exposure (2D)",include_str!("../shaders/native/HB_LocalExposure.hlsl")),
    ("Center Magnifier 2x (SDR)",include_str!("../shaders/native/HB_CenterZoom2.hlsl")),
    ("Center Magnifier 4x (SDR)",include_str!("../shaders/native/HB_CenterZoom4.hlsl")),
    ("Warm Display Comfort (SDR)",include_str!("../shaders/native/HB_BlueLightComfort.hlsl")),
    ("False-color Luminance (not thermal)",include_str!("../shaders/native/HB_ThermalLuma.hlsl")),
    ("SDR Highlight Roll-off",include_str!("../shaders/native/HB_SDRHighlightRollOff.hlsl")),
    ("Luminance Detail (original)",include_str!("../shaders/native/HB_LumaDetail.hlsl")),
    ("Subtle Barrel Lens (SDR)",include_str!("../shaders/native/HB_LensBarrel.hlsl")),
    ("Fine Film Grain (SDR)",include_str!("../shaders/native/HB_SDRFilmGrain.hlsl")),
    ("Global Histogram Exposure (SDR)",include_str!("../shaders/native/HB_HistogramExposure.hlsl")),
    ("Adjustable Color Grade",include_str!("../shaders/native/HB_ColorGrade.hlsl")),
    ("Linear-light Contrast (SDR)",include_str!("../shaders/native/HB_LinearContrast.hlsl")),
    ("Reinhard-style SDR shoulder",include_str!("../shaders/native/HB_ReinhardSDR.hlsl")),
    ("Edge-protected detail",include_str!("../shaders/native/HB_EdgeGuard.hlsl")),
    ("Gradient-aware deband (SDR)",include_str!("../shaders/native/HB_AdaptiveDeband.hlsl")),
    ("Highlight diffusion",include_str!("../shaders/native/HB_HighlightDiffusion.hlsl")),
    ("Selective warm/cool separation",include_str!("../shaders/native/HB_SelectiveCool.hlsl")),
    ("Soft ink contour",include_str!("../shaders/native/HB_PaperInk.hlsl")),
    ("Soft optical vignette",include_str!("../shaders/native/HB_FocusVignette.hlsl")),
    ("Estimated relief contact shade (2D)",include_str!("../shaders/native/HB_ReliefAO.hlsl")),
    ("AMD FidelityFX CAS sharpening",""),
    ("CRT Shadow Mask",include_str!("../shaders/native/HB_CRTShadowMask.hlsl")),
    ("CRT Curved Arcade",include_str!("../shaders/native/HB_CRTCurved.hlsl")),
    ("NVIDIA FXAA 3.11 (quality 12)",include_str!("../shaders/native/HB_FXAA311.hlsl")),
    ("LumaSharpen 9-tap (original)",include_str!("../shaders/native/HB_LumaSharpen9.hlsl")),
    ("ACES-style filmic SDR (not HDR)",include_str!("../shaders/native/HB_ACESFitSDR.hlsl")),
    ("SMAA 1x High (original reference)",""),
    ("Tiled LDS Denoise 3x3 (GPU shared cache)",include_str!("../shaders/native/HB_TiledLDS.hlsl")),
    ("Custom HLSL live reload (own D3D11)",""),
    ("Estimated Horizon Occlusion (color-only 2D)",include_str!("../shaders/native/HB_EstimatedHorizonAO.hlsl")),
];
pub const HISTOGRAM_STAGE:usize=50;
pub const CAS_STAGE:usize=61;
pub const FXAA_STAGE:usize=64;
pub const SMAA_STAGE:usize=67;
pub const TILED_LDS_STAGE:usize=68;
pub const CUSTOM_STAGE:usize=69;
pub fn program_source(i:usize)->Option<std::borrow::Cow<'static,str>>{
 if i==CAS_STAGE{Some(std::borrow::Cow::Owned(crate::cas::shader_source()))}else if i==SMAA_STAGE{Some(std::borrow::Cow::Owned(crate::smaa::shader_source(0,73,51)))}else if i==CUSTOM_STAGE{Some(std::borrow::Cow::Owned(crate::custom_hlsl::source()))}else{PROGRAMS.get(i).map(|p|std::borrow::Cow::Borrowed(p.1))}
}
pub fn compile_all()->Result<usize,String>{
    for (i,(name,_)) in PROGRAMS.iter().enumerate() {
        if i==SMAA_STAGE{crate::smaa::compile_all().map_err(|e|format!("{name}: {e}"))?;continue;}
        crate::shadercheck::compile_native_hlsl(&program_source(i).ok_or("Missing shader source")?)
            .map_err(|e|format!("{name}: {e}"))?;
    }
    Ok(PROGRAMS.len())
}
/// Dispatches one real native GPU filter. Input is shader-readable D3D11
/// texture; output is a separate shader-readable + unordered-access texture.
/// All memory belongs to the same hardware D3D11 device (RX 580).
pub fn dispatch_filter(device:&ID3D11Device,context:&ID3D11DeviceContext,
    input:&ID3D11Texture2D,index:usize)->Result<ID3D11Texture2D,String>{
    if index==HISTOGRAM_STAGE||index==crate::color_grade::STAGE||index==CAS_STAGE||index==FXAA_STAGE||index==SMAA_STAGE{
        let mut desc=D3D11_TEXTURE2D_DESC::default();
        unsafe{input.GetDesc(&mut desc);}
        let pipe=crate::native_pipeline::NativeGpuPipeline::new(device,desc.Width,desc.Height)?;
        return pipe.dispatch(device,context,input,&[index]);
    }
    let (name,code)=PROGRAMS.get(index).ok_or("Invalid native filter index")?;
    let bytes=crate::shadercheck::compile_native_hlsl(code)
        .map_err(|e|format!("{name}: {e}"))?;
    let mut shader:Option<ID3D11ComputeShader>=None;
    unsafe{device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}
        .map_err(|e|format!("{name}: CreateComputeShader: {e}"))?;
    let shader=shader.ok_or("Native HLSL compute shader is empty")?;
    let mut desc=D3D11_TEXTURE2D_DESC::default();
    unsafe{input.GetDesc(&mut desc);}
    if desc.Width==0||desc.Height==0 {return Err("GPU input texture has zero dimensions".into());}
    desc.Format=DXGI_FORMAT_R8G8B8A8_UNORM;
    desc.MipLevels=1;
    desc.ArraySize=1;
    desc.Usage=D3D11_USAGE_DEFAULT;
    desc.BindFlags=(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32;
    desc.CPUAccessFlags=0;
    desc.MiscFlags=0;
    let mut output:Option<ID3D11Texture2D>=None;
    unsafe{device.CreateTexture2D(&desc,None,Some(&mut output))}
        .map_err(|e|format!("{name}: create GPU output texture: {e}"))?;
    let output=output.ok_or("No output texture allocated")?;
    let mut srv:Option<ID3D11ShaderResourceView>=None;
    unsafe{device.CreateShaderResourceView(input,None,Some(&mut srv))}
        .map_err(|e|format!("{name}: input SRV: {e}"))?;
    let mut uav:Option<ID3D11UnorderedAccessView>=None;
    unsafe{device.CreateUnorderedAccessView(&output,None,Some(&mut uav))}
        .map_err(|e|format!("{name}: output UAV: {e}"))?;
    let srv=srv.ok_or("Native shader resource not allocated")?;
    let uav=uav.ok_or("Native unordered access view not allocated")?;
    let active_uavs=[Some(uav)];
    unsafe{
        context.CSSetShader(&shader,None);
        context.CSSetShaderResources(0,Some(&[Some(srv)]));
        context.CSSetUnorderedAccessViews(0,1,Some(active_uavs.as_ptr()),None);
        context.Dispatch(desc.Width.div_ceil(8),desc.Height.div_ceil(8),1);
        // Prevent stale SRV/UAV bindings on the next pass and for game-free teardown.
        context.CSSetShaderResources(0,Some(&[None]));
        let empty_uavs:[Option<ID3D11UnorderedAccessView>;1]=[None];
        context.CSSetUnorderedAccessViews(0,1,Some(empty_uavs.as_ptr()),None);
        context.CSSetShader(None::<&ID3D11ComputeShader>,None);
        context.Flush();
    }
    Ok(output)
}
pub fn dispatch_chain(device:&ID3D11Device,context:&ID3D11DeviceContext,
    input:&ID3D11Texture2D,chain:&[usize])->Result<ID3D11Texture2D,String>{
    if chain.is_empty()||chain.len()>4{
        return Err("Native GPU chain must have 1 to 4 shader passes".into());
    }
    let mut texture=input.clone();
    for &index in chain{
        texture=dispatch_filter(device,context,&texture,index)?;
    }
    Ok(texture)
}
/// A bounded read-only GPU completion query for diagnostics. Does not map,
/// read, hash or transfer captured game/desktop pixels into system RAM.
pub fn verify_gpu_completion(device:&ID3D11Device,context:&ID3D11DeviceContext)->Result<(),String>{
    let desc=D3D11_QUERY_DESC{Query:D3D11_QUERY_EVENT,MiscFlags:0};
    let mut query:Option<ID3D11Query>=None;
    unsafe{device.CreateQuery(&desc,Some(&mut query))}
        .map_err(|e|format!("D3D11 GPU completion query create: {e}"))?;
    let query=query.ok_or("GPU completion query not created")?;
    unsafe{context.End(&query);context.Flush();}
    for _ in 0..500{
        let mut is_finished=0i32;
        let status=unsafe{context.GetData(&query,Some(&mut is_finished as *mut i32
            as *mut std::ffi::c_void),std::mem::size_of::<i32>() as u32,0)};
        if let Err(e)=status{return Err(format!("D3D11 GetData: {e}"));}
        if is_finished!=0{return Ok(());}
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    Err("GPU did not complete native compute shader passes within 1 second".into())
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn all_native_compute_filters_compile_as_real_hlsl(){
        assert_eq!(compile_all().unwrap(),PROGRAMS.len());
    }
    #[test]fn native_effects_are_distinct_programs_not_duplicate_ui_cards(){
        assert_eq!(PROGRAMS.len(),71);
        let mut seen=std::collections::BTreeSet::new();
        for (i,(name,_)) in PROGRAMS.iter().enumerate(){
            let code=program_source(i).unwrap();
            assert!(seen.insert(name.to_string()));
            assert!(seen.insert(code.to_string()));
            assert!(code.contains("RWTexture2D")&&code.contains("CSMain"));
        }
    }
}


#[cfg(test)]mod v100_estimated_horizon_ao_gpu_tests{
 #[test]fn actual_gpu_horizon_occlusion_detects_bounded_contrast_without_faking_depth(){
  let gpu=crate::photo_lab::Gpu::new().unwrap();
  let original=image::RgbaImage::from_fn(97,65,|x,_y|
    if x<48{image::Rgba([60,60,60,140])}else{image::Rgba([235,235,235,140])});
  let input=gpu.upload(&original).unwrap();
  let pipe=crate::native_pipeline::NativeGpuPipeline::new(&gpu.device,97,65).unwrap();
  let index=70;
  assert!(crate::native_filter::PROGRAMS[index].0.contains("Horizon"));
  let after=gpu.read(&pipe.dispatch(&gpu.device,&gpu.ctx,&input,&[index]).unwrap()).unwrap();
  for y in [0,10,32,64]{
   assert_eq!(after.get_pixel(10,y),original.get_pixel(10,y));
   assert_eq!(after.get_pixel(88,y),original.get_pixel(88,y));
   assert_eq!(after.get_pixel(45,y)[3],140);
  }
  let changed=after.pixels().zip(original.pixels()).filter(|(a,b)|a!=b).count();
  assert!(changed>=40,"Estimated contrast edges did not produce a real GPU effect ({changed})");
 }
}
