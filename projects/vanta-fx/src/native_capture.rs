//! Native GPU-only desktop frame capture proof-of-engine.
//!
//! This is NOT Magpie and does not resize or alter any game window.
//! IDXGIOutputDuplication -> ID3D11Texture2D -> CopyResource to another
//! shader-readable DEFAULT-usage ID3D11Texture2D; no CPU readback.
//!
//! This is *capture*, NOT presentation over true exclusive fullscreen.
//! No user pixels are read, saved, transmitted or logged by this diagnostic.
use windows::{
    core::Interface,
    Win32::{
        Foundation::HMODULE,
        Graphics::{
            Direct3D::{D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL_11_0},
            Direct3D11::{
                D3D11CreateDevice, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                D3D11_SDK_VERSION, D3D11_BIND_SHADER_RESOURCE, D3D11_USAGE_DEFAULT,
                D3D11_TEXTURE2D_DESC, ID3D11Device, ID3D11DeviceContext,
                ID3D11Resource, ID3D11Texture2D,
            },
            Dxgi::{
                CreateDXGIFactory1, IDXGIAdapter, IDXGIFactory1, IDXGIOutput1,
                IDXGIResource, DXGI_OUTDUPL_FRAME_INFO, DXGI_ERROR_WAIT_TIMEOUT,
            },
        },
    },
};

#[derive(Debug,Clone)]
pub struct NativeCaptureProbe {
    pub gpu_name:String,
    pub vendor_id:u32,
    pub output_index:u32,
    pub width:u32,
    pub height:u32,
    pub gpu_copy_complete:bool,
    pub shader_passes:usize,
    pub status:&'static str,
}

impl NativeCaptureProbe {
    pub fn report(&self)->String{
        format!("GPU={} vendor=0x{:04x} display={} size={}x{} GPU_COPY={} NATIVE_SHADER_PASSES={} STATUS={}",
            self.gpu_name,self.vendor_id,self.output_index,self.width,self.height,
            self.gpu_copy_complete,self.shader_passes,self.status)
    }
}
/// A read-only diagnostic explicitly invoked by --native-capture-probe.
/// The acquired frame is immediately released even if a GPU operation fails.
pub fn probe_gpu_only()->Result<NativeCaptureProbe,String> {
    let factory:IDXGIFactory1=unsafe{CreateDXGIFactory1()}
        .map_err(|e|format!("CreateDXGIFactory1: {e}"))?;
    let mut last_error=String::from("No DXGI adapter with capturable desktop output");
    for adapter_index in 0..8 {
        let adapter=match unsafe{factory.EnumAdapters1(adapter_index)}{
            Ok(a)=>a,Err(_)=>break,
        };
        let desc=unsafe{adapter.GetDesc1()}
            .map_err(|e|format!("DXGI adapter description: {e}"))?;
        // Microsoft WARP/basic adapter has no hardware screen output.
        if desc.VendorId==0x1414{continue;}
        let label=String::from_utf16_lossy(&desc.Description)
            .trim_matches('\0').trim().to_owned();
        let base:IDXGIAdapter=adapter.cast()
            .map_err(|e|format!("IDXGIAdapter cast: {e}"))?;
        let mut device:Option<ID3D11Device>=None;
        let mut context:Option<ID3D11DeviceContext>=None;
        let make=unsafe{D3D11CreateDevice(&base,D3D_DRIVER_TYPE_UNKNOWN,HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,Some(&[D3D_FEATURE_LEVEL_11_0]),
            D3D11_SDK_VERSION,Some(&mut device),None,Some(&mut context))};
        if let Err(err)=make {
            last_error=format!("{label}: D3D11CreateDevice: {err}");
            continue;
        }
        let (Some(device),Some(context))=(device,context)else{
            last_error=format!("{label}: missing native D3D11 device/context");
            continue;
        };
        for output_index in 0..8 {
            let out=match unsafe{adapter.EnumOutputs(output_index)}{
                Ok(a)=>a,Err(_)=>break,
            };
            let out1:IDXGIOutput1=match out.cast(){Ok(o)=>o,Err(e)=>{
                last_error=format!("{label}: IDXGIOutput1: {e}");continue;
            }};
            let duplication=match unsafe{out1.DuplicateOutput(&device)} {
                Ok(d)=>d,Err(e)=>{
                    last_error=format!("{label} monitor {output_index}: DuplicateOutput: {e}");
                    continue;
                }
            };
            let mut info=DXGI_OUTDUPL_FRAME_INFO::default();
            let mut resource:Option<IDXGIResource>=None;
            match unsafe{duplication.AcquireNextFrame(450,&mut info,&mut resource)} {
                Err(err) if err.code()==DXGI_ERROR_WAIT_TIMEOUT => {
                    let dupl_desc=unsafe{duplication.GetDesc()};
                    return Ok(NativeCaptureProbe{
                        gpu_name:label,vendor_id:desc.VendorId,output_index,
                        width:dupl_desc.ModeDesc.Width,height:dupl_desc.ModeDesc.Height,
                        gpu_copy_complete:false,shader_passes:0,status:"READY_NO_NEW_DESKTOP_FRAME",
                    });
                }
                Err(err)=>{
                    last_error=format!("{label} monitor {output_index}: AcquireNextFrame: {err}");
                    continue;
                }
                Ok(())=>{}
            }
            let copied=(||->Result<(u32,u32,ID3D11Texture2D),String>{
                let src:ID3D11Texture2D=resource
                    .ok_or("DXGI returned no GPU resource")?
                    .cast().map_err(|e|format!("Desktop texture cast: {e}"))?;
                let mut texture_desc=D3D11_TEXTURE2D_DESC::default();
                unsafe{src.GetDesc(&mut texture_desc)};
                let (w,h)=(texture_desc.Width,texture_desc.Height);
                if w==0||h==0||w>16384||h>16384 {
                    return Err(format!("Unexpected DXGI texture dimensions {w}x{h}"));
                }
                // Reusable pixel destination entirely on the RX 580 GPU.
                // No STAGING, no D3D11_MAP_READ, no CPUAccessFlags.
                texture_desc.Usage=D3D11_USAGE_DEFAULT;
                texture_desc.BindFlags=D3D11_BIND_SHADER_RESOURCE.0 as u32;
                texture_desc.CPUAccessFlags=0;
                texture_desc.MiscFlags=0;
                texture_desc.MipLevels=1;
                texture_desc.ArraySize=1;
                let mut target:Option<ID3D11Texture2D>=None;
                unsafe{device.CreateTexture2D(&texture_desc,None,Some(&mut target))}
                    .map_err(|e|format!("Create GPU texture: {e}"))?;
                let target=target.ok_or("No GPU target")?;
                let destination:ID3D11Resource=target.cast()
                    .map_err(|e|format!("Target resource: {e}"))?;
                let input:ID3D11Resource=src.cast()
                    .map_err(|e|format!("Source resource: {e}"))?;
                unsafe{
                    context.CopyResource(&destination,&input);
                    context.Flush();
                }
                // DXGI says the acquired desktop surface must not be used
                // after ReleaseFrame. Wait for GPU copy before releasing it.
                crate::native_filter::verify_gpu_completion(&device,&context)?;
                Ok((w,h,target))
            })();
            let released=unsafe{duplication.ReleaseFrame()};
            if let Err(e)=released{
                return Err(format!("{label}: DXGI ReleaseFrame: {e}"));
            }
            let (width,height,source)=copied?;
            // Real native Direct3D11 Compute Shader pipeline, after the DXGI
            // frame has been released. No Magpie or CPU pixel readbacks.
            let processed=crate::native_filter::dispatch_chain(&device,&context,&source,
                &[0,1,2,3])?;
            let mut final_desc=D3D11_TEXTURE2D_DESC::default();
            unsafe{processed.GetDesc(&mut final_desc);}
            if final_desc.Width!=width||final_desc.Height!=height{
                return Err("Native compute output dimensions differ from input".into());
            }
            crate::native_filter::verify_gpu_completion(&device,&context)?;
            return Ok(NativeCaptureProbe{
                gpu_name:label,vendor_id:desc.VendorId,output_index,width,height,
                gpu_copy_complete:true,shader_passes:4,status:"NATIVE_GPU_SHADERS_COMPLETED",
            });
        }
    }
    Err(last_error)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn probe_status_cannot_claim_filter_or_frame_generation(){
        let s=NativeCaptureProbe{
            gpu_name:"RX 580".into(),vendor_id:0x1002,output_index:0,
            width:1920,height:1080,gpu_copy_complete:true,shader_passes:4,
            status:"NATIVE_GPU_SHADERS_COMPLETED",
        }.report();
        assert!(s.contains("GPU_COPY=true"));
        assert!(!s.contains("FRAME_GENERATION"));
        assert!(!s.contains("EXCLUSIVE_FILTER"));
    }
}
