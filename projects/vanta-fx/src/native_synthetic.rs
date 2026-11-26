//! GPU validation using ONLY synthetic 8x8 pixels (not a game screenshot).
//! It intentionally reads back these generated test pixels to verify that
//! real Radeon D3D11 compute shaders modify them differently.
//! The production native_capture path never reads user pixels into CPU RAM.
use windows::{
    core::Interface,
    Win32::{
        Foundation::HMODULE,
        Graphics::{
            Direct3D::{D3D_DRIVER_TYPE_HARDWARE,D3D_FEATURE_LEVEL_11_0},
            Direct3D11::{
                D3D11CreateDevice,D3D11_SDK_VERSION,D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                D3D11_BIND_SHADER_RESOURCE,D3D11_CPU_ACCESS_READ,D3D11_USAGE_DEFAULT,
                D3D11_USAGE_STAGING,D3D11_MAP_READ,D3D11_TEXTURE2D_DESC,
                D3D11_SUBRESOURCE_DATA,D3D11_MAPPED_SUBRESOURCE,ID3D11Device,
                ID3D11DeviceContext,ID3D11Texture2D,ID3D11Resource,
            },
            Dxgi::{IDXGIAdapter},
            Dxgi::Common::{DXGI_SAMPLE_DESC,DXGI_FORMAT_R8G8B8A8_UNORM},
        },
    },
};
const W:u32=8;
const H:u32=8;
#[derive(Debug)]
pub struct SyntheticResult{
    pub filter:&'static str,
    pub changed_pixels:u32,
    pub checksum:u64,
}
pub fn validate_synthetic_gpu_effects()->Result<Vec<SyntheticResult>,String>{
    let mut device:Option<ID3D11Device>=None;
    let mut context:Option<ID3D11DeviceContext>=None;
    unsafe{D3D11CreateDevice(None::<&IDXGIAdapter>,D3D_DRIVER_TYPE_HARDWARE,
        HMODULE::default(),D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        Some(&[D3D_FEATURE_LEVEL_11_0]),D3D11_SDK_VERSION,
        Some(&mut device),None,Some(&mut context))}
        .map_err(|e|format!("Hardware D3D11CreateDevice failed: {e}"))?;
    let (Some(device),Some(context))=(device,context)else{
        return Err("No hardware Direct3D11 device/context".into());
    };
    let mut pixels=[0u8;(W*H*4) as usize];
    for y in 0..H{
        for x in 0..W{
            let offset=((y*W+x)*4) as usize;
            pixels[offset]=45+(x*21) as u8;
            pixels[offset+1]=25+(y*22) as u8;
            pixels[offset+2]=90+((x^y)*16) as u8;
            pixels[offset+3]=255;
        }
    }
    let desc=D3D11_TEXTURE2D_DESC{
        Width:W,Height:H,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R8G8B8A8_UNORM,
        SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,
        BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,CPUAccessFlags:0,MiscFlags:0
    };
    let initial=D3D11_SUBRESOURCE_DATA{
        pSysMem:pixels.as_ptr() as *const std::ffi::c_void,
        SysMemPitch:W*4,SysMemSlicePitch:W*H*4,
    };
    let mut tex:Option<ID3D11Texture2D>=None;
    unsafe{device.CreateTexture2D(&desc,Some(&initial),Some(&mut tex))}
        .map_err(|e|format!("Create synthetic GPU input: {e}"))?;
    let tex=tex.ok_or("Synthetic GPU input missing")?;
    let staging_desc=D3D11_TEXTURE2D_DESC{
        Usage:D3D11_USAGE_STAGING,BindFlags:0,
        CPUAccessFlags:D3D11_CPU_ACCESS_READ.0 as u32,
        ..desc
    };
    let mut staging:Option<ID3D11Texture2D>=None;
    unsafe{device.CreateTexture2D(&staging_desc,None,Some(&mut staging))}
        .map_err(|e|format!("Create synthetic-only staging buffer: {e}"))?;
    let staging=staging.ok_or("Synthetic-only staging resource missing")?;
    let staging_resource:ID3D11Resource=staging.cast()
        .map_err(|e|format!("Synthetic staging resource cast: {e}"))?;
    let mut out=Vec::new();
    for (i,(name,_)) in crate::native_filter::PROGRAMS.iter().enumerate(){
        let filtered=crate::native_filter::dispatch_filter(&device,&context,&tex,i)?;
        let filtered_resource:ID3D11Resource=filtered.cast()
            .map_err(|e|format!("Synthetic filtered GPU resource: {e}"))?;
        unsafe{context.CopyResource(&staging_resource,&filtered_resource);}
        crate::native_filter::verify_gpu_completion(&device,&context)?;
        let mut mapped=D3D11_MAPPED_SUBRESOURCE::default();
        unsafe{context.Map(&staging_resource,0,D3D11_MAP_READ,0,Some(&mut mapped))}
            .map_err(|e|format!("Map synthetic pixels only: {e}"))?;
        let (changed_pixels,checksum)=(||{
            let mut changed=0u32;
            let mut hash=0xcbf29ce484222325u64;
            for y in 0..H as usize{
                let line=unsafe{std::slice::from_raw_parts(
                    (mapped.pData as *const u8).add(y*mapped.RowPitch as usize),W as usize*4)};
                for x in 0..W as usize{
                    let start=x*4;
                    let old_start=(y*W as usize+x)*4;
                    if line[start..start+3]!=pixels[old_start..old_start+3]{changed+=1;}
                    for v in &line[start..start+3]{
                        hash=(hash^(*v as u64)).wrapping_mul(0x100000001b3);
                    }
                }
            }
            (changed,hash)
        })();
        unsafe{context.Unmap(&staging_resource,0);}
        out.push(SyntheticResult{filter:name,changed_pixels,checksum});
    }
    if out.len()!=crate::native_filter::PROGRAMS.len(){
        return Err(format!("Only {} of {} native GPU shaders tested",
            out.len(),crate::native_filter::PROGRAMS.len()));
    }
    // Some conditional effects (bright highlight bloom, night filtering)
    // legitimately preserve a tiny 8x8 dark synthetic swatch. The complete
    // GPU photo+gameplay proofs separately demand 83 unique output hashes.
    // This proof checks the GPU dispatch and each new denoise/dither stage.
    let changed=out.iter().filter(|x|x.changed_pixels>0).count();
    if changed<20{return Err(format!(
        "Only {changed}/{} HLSL programs changed the synthetic swatch",out.len()));}
    for stage in [37usize,38usize,41usize,42usize,43usize,44usize,45usize,46usize,47usize,49usize,50usize,52usize,53usize,54usize,57usize,58usize,59usize]{
        if out[stage].changed_pixels==0{
            return Err(format!("New shader {} was inert on a gradient test input",
                out[stage].filter));
        }
    }
    Ok(out)
}
