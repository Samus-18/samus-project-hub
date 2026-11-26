//! GPU-generated proof gallery from an actual public-domain photographic file.
//! Reads back ONLY bundled photo-derived pixels for previews, never live game
//! frames or private screen content. Native shaders run on the GPU (not Magpie).
use windows::{
    core::Interface,
    Win32::{
        Foundation::HMODULE,
        Graphics::{
            Direct3D::{D3D_DRIVER_TYPE_HARDWARE,D3D_FEATURE_LEVEL_11_0},
            Direct3D11::{
                D3D11CreateDevice,D3D11_SDK_VERSION,D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                D3D11_BIND_SHADER_RESOURCE,D3D11_CPU_ACCESS_READ,D3D11_USAGE_DEFAULT,
                D3D11_USAGE_STAGING,D3D11_MAP_READ,
                D3D11_TEXTURE2D_DESC,D3D11_SUBRESOURCE_DATA,
                D3D11_MAPPED_SUBRESOURCE,ID3D11Device,ID3D11DeviceContext,
                ID3D11Texture2D,ID3D11Resource,
            },
            Dxgi::IDXGIAdapter,
            Dxgi::Common::{DXGI_SAMPLE_DESC,DXGI_FORMAT_R8G8B8A8_UNORM},
        },
    },
};
use std::{ffi::c_void,path::Path};
const W:u32=512;
const H:u32=288;

pub fn export_native_photo_previews(target:&Path)->Result<Vec<u64>,String>{
    export_gpu_previews_from_source(target,
        include_bytes!("../assets/preview_photo_cc0.jpg"),"native_photo")
}
pub fn export_native_gameplay_previews(target:&Path)->Result<Vec<u64>,String>{
    export_gpu_previews_from_source(target,
        include_bytes!("../assets/hero_gameplay_v10.png"),"native_gameplay")
}
fn export_gpu_previews_from_source(target:&Path,source_bytes:&[u8],
    prefix:&str)->Result<Vec<u64>,String>{
    crate::native_presets::validate()?;
    std::fs::create_dir_all(target).map_err(|e|e.to_string())?;
    let picture=image::load_from_memory(source_bytes)
        .map_err(|e|format!("Read authentic image reference: {e}"))?
        .resize_to_fill(W,H,image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let pixels=picture.as_raw();
    let desc=D3D11_TEXTURE2D_DESC{
        Width:W,Height:H,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R8G8B8A8_UNORM,
        SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,
        BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,CPUAccessFlags:0,MiscFlags:0,
    };
    let mut device:Option<ID3D11Device>=None;
    let mut context:Option<ID3D11DeviceContext>=None;
    unsafe{D3D11CreateDevice(None::<&IDXGIAdapter>,D3D_DRIVER_TYPE_HARDWARE,
        HMODULE::default(),D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        Some(&[D3D_FEATURE_LEVEL_11_0]),D3D11_SDK_VERSION,
        Some(&mut device),None,Some(&mut context))}
        .map_err(|e|format!("Native GPU creation for real-photo proofs: {e}"))?;
    let device=device.ok_or("No GPU device")?;
    let context=context.ok_or("No GPU context")?;
    let initial=D3D11_SUBRESOURCE_DATA{
        pSysMem:pixels.as_ptr() as *const c_void,
        SysMemPitch:W*4,SysMemSlicePitch:W*H*4,
    };
    let mut original:Option<ID3D11Texture2D>=None;
    unsafe{device.CreateTexture2D(&desc,Some(&initial),Some(&mut original))}
        .map_err(|e|format!("Create original public-domain photo texture: {e}"))?;
    let original=original.ok_or("No photo texture on GPU")?;
    let staging_desc=D3D11_TEXTURE2D_DESC{
        Usage:D3D11_USAGE_STAGING,BindFlags:0,
        CPUAccessFlags:D3D11_CPU_ACCESS_READ.0 as u32,..desc
    };
    let mut staging:Option<ID3D11Texture2D>=None;
    unsafe{device.CreateTexture2D(&staging_desc,None,Some(&mut staging))}
        .map_err(|e|format!("Create photo-only staging texture: {e}"))?;
    let staging=staging.ok_or("No photo-only staging texture")?;
    let stage_resource:ID3D11Resource=staging.cast()
        .map_err(|e|format!("Stage resource cast: {e}"))?;
    let cached=crate::native_pipeline::NativeGpuPipeline::new(&device,W,H)?;
    let mut hashes=Vec::new();
    for (idx,preset) in crate::native_presets::PRESETS.iter().enumerate(){
        let filtered=cached.dispatch(&device,&context,&original,preset.stages)?;
        let source:ID3D11Resource=filtered.cast().map_err(|e|format!("Photo GPU output: {e}"))?;
        unsafe{context.CopyResource(&stage_resource,&source);}
        crate::native_filter::verify_gpu_completion(&device,&context)?;
        let mut mapped=D3D11_MAPPED_SUBRESOURCE::default();
        unsafe{context.Map(&stage_resource,0,D3D11_MAP_READ,0,Some(&mut mapped))}
            .map_err(|e|format!("Map photo-only GPU preview: {e}"))?;
        let copy=(||->Result<Vec<u8>,String>{
            if mapped.pData.is_null(){return Err("Mapped photo buffer empty".into());}
            let mut out=vec![0u8;(W*H*4) as usize];
            for y in 0..H as usize{
                let line=unsafe{std::slice::from_raw_parts(
                    (mapped.pData as *const u8).add(y*mapped.RowPitch as usize),W as usize*4)};
                out[y*W as usize*4..(y+1)*W as usize*4].copy_from_slice(line);
            }
            Ok(out)
        })();
        unsafe{context.Unmap(&stage_resource,0);}
        let data=copy?;
        let photo=image::RgbaImage::from_raw(W,H,data.clone())
            .ok_or("Invalid photo GPU RGBA result")?;
        let filename=target.join(format!("{prefix}_{idx:02}.png"));
        photo.save(&filename).map_err(|e|format!("Save GPU-processed public photo preview: {e}"))?;
        let mut hash=0xcbf29ce484222325u64;
        for byte in data.iter().step_by(3){hash=(hash^(*byte as u64)).wrapping_mul(0x100000001b3);}
        hashes.push(hash);
        println!("REAL_GPU_REFERENCE={} INDEX={idx:02} NAME={} PASSES={} HASH=0x{hash:016x}",
            prefix,preset.label,preset.stages.len());
    }
    if hashes.iter().collect::<std::collections::BTreeSet<_>>().len()!=hashes.len(){
        return Err("Some GPU photo presets produced identical preview frames".into());
    }
    Ok(hashes)
}
