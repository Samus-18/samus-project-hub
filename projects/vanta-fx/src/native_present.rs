//! Independent DXGI flip-discard presentation **proof**, no Magpie.
//!
//! Takes a synthetic D3D11 texture, passes it through four actual HLSL GPU
//! compute filters and submits it to our OWN hidden flip-model swapchain.
//! Does not touch or change any game window, driver setting, or fullscreen mode.
//!
//! This proves buffer compatibility and native Present API wiring. It does
//! NOT prove visible pixels on-screen or direct true-exclusive game filtering.
use std::{ffi::c_void,ptr};
use windows::{
    core::{Interface,BOOL},
    Win32::{
        Foundation::{HMODULE,HWND},
        Graphics::{
            Direct3D::{D3D_DRIVER_TYPE_HARDWARE,D3D_FEATURE_LEVEL_11_0},
            Direct3D11::{
                D3D11CreateDevice,D3D11_CREATE_DEVICE_BGRA_SUPPORT,D3D11_SDK_VERSION,
                D3D11_BIND_SHADER_RESOURCE,D3D11_USAGE_DEFAULT,
                D3D11_TEXTURE2D_DESC,D3D11_SUBRESOURCE_DATA,
                ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11Resource,
            },
            Dxgi::{
                CreateDXGIFactory1,IDXGIFactory2,IDXGIOutput,DXGI_SWAP_CHAIN_DESC1,
                DXGI_SWAP_EFFECT_FLIP_DISCARD,DXGI_USAGE_RENDER_TARGET_OUTPUT,
                DXGI_SCALING_STRETCH,DXGI_PRESENT,
            },
            Dxgi::Common::{
                DXGI_ALPHA_MODE_IGNORE,DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_SAMPLE_DESC,
            },
        },
    },
};
type NativeHandle=*mut c_void;
#[repr(C)]
struct WndClass{
    style:u32,
    wnd_proc:Option<unsafe extern "system" fn(NativeHandle,u32,usize,isize)->isize>,
    cls_extra:i32, wnd_extra:i32, instance:NativeHandle,
    icon:NativeHandle,cursor:NativeHandle,brush:NativeHandle,
    menu_name:*const u16,class_name:*const u16,
}
#[link(name="user32")]
unsafe extern "system"{
    fn RegisterClassW(class:*const WndClass)->u16;
    fn CreateWindowExW(ex:u32,class:*const u16,title:*const u16,style:u32,
        x:i32,y:i32,w:i32,h:i32,parent:NativeHandle,menu:NativeHandle,
        instance:NativeHandle,param:NativeHandle)->NativeHandle;
    fn DestroyWindow(hwnd:NativeHandle)->i32;
    fn DefWindowProcW(hwnd:NativeHandle,msg:u32,wparam:usize,lparam:isize)->isize;
}
#[link(name="kernel32")]
unsafe extern "system"{
    fn GetModuleHandleW(module:*const u16)->NativeHandle;
}
unsafe extern "system" fn test_window_proc(hwnd:NativeHandle,message:u32,w:usize,l:isize)->isize{
    unsafe{DefWindowProcW(hwnd,message,w,l)}
}
struct HiddenProofWindow{hwnd:NativeHandle}
impl HiddenProofWindow{
    fn create()->Result<Self,String>{
        let name="HorizonBoostNativePresentProof".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let module=unsafe{GetModuleHandleW(ptr::null())};
        let cls=WndClass{
            style:0,wnd_proc:Some(test_window_proc),cls_extra:0,wnd_extra:0,
            instance:module,icon:ptr::null_mut(),cursor:ptr::null_mut(),
            brush:ptr::null_mut(),menu_name:ptr::null(),class_name:name.as_ptr(),
        };
        let registered=unsafe{RegisterClassW(&cls)};
        if registered==0&&unsafe{windows::Win32::Foundation::GetLastError()}.0!=1410{return Err("Cannot register isolated proof HWND".into());}
        let title="Horizon Boost Native Present Proof".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        // Hidden 96x96 window. Never show or activate it over a running game.
        let hwnd=unsafe{CreateWindowExW(0,name.as_ptr(),title.as_ptr(),0x80000000,
            0,0,96,96,ptr::null_mut(),ptr::null_mut(),module,ptr::null_mut())};
        if hwnd.is_null(){return Err("Unable to create isolated proof window".into());}
        Ok(Self{hwnd})
    }
}
impl Drop for HiddenProofWindow{
    fn drop(&mut self){if !self.hwnd.is_null(){unsafe{DestroyWindow(self.hwnd);}}}
}

pub fn swapchain_descriptor(width:u32,height:u32)->DXGI_SWAP_CHAIN_DESC1{
    DXGI_SWAP_CHAIN_DESC1{
        Width:width,Height:height,Format:DXGI_FORMAT_R8G8B8A8_UNORM,
        Stereo:BOOL(0),SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
        BufferUsage:DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount:2,Scaling:DXGI_SCALING_STRETCH,
        SwapEffect:DXGI_SWAP_EFFECT_FLIP_DISCARD,AlphaMode:DXGI_ALPHA_MODE_IGNORE,
        Flags:0,
    }
}
#[derive(Debug,Clone)]
pub struct NativePresentProof{
    pub width:u32,pub height:u32,
    pub shader_passes:usize,
    pub swap_buffers:u32,
    /// HResult from Present; can be success or STATUS_OCCLUDED if hidden.
    pub present_hresult:i32,
    pub submitted_frames:u32,
    pub shader_programs_compiled_once:usize,
    pub persistent_gpu_surfaces:usize,
}
impl NativePresentProof{
    pub fn summary(&self)->String{
        format!("GPU_NATIVE_PRESENT=ATTEMPTED_ON_OWN_HIDDEN_SWAPCHAIN SIZE={}x{} PASSES={} FLIP_BUFFERS={} SUBMITTED_FRAMES={} CACHED_SHADER_PROGRAMS={} REUSED_GPU_SURFACES={} PRESENT_HRESULT=0x{:08X} VISIBLE_GAME_OUTPUT=false",
            self.width,self.height,self.shader_passes,self.swap_buffers,
            self.submitted_frames,self.shader_programs_compiled_once,self.persistent_gpu_surfaces,
            self.present_hresult as u32)
    }
}

pub fn probe_hidden_present()->Result<NativePresentProof,String>{
    const SIZE:u32=64;
    let window=HiddenProofWindow::create()?;
    let mut device:Option<ID3D11Device>=None;
    let mut context:Option<ID3D11DeviceContext>=None;
    unsafe{D3D11CreateDevice(None::<&windows::Win32::Graphics::Dxgi::IDXGIAdapter>,
        D3D_DRIVER_TYPE_HARDWARE,HMODULE::default(),D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        Some(&[D3D_FEATURE_LEVEL_11_0]),D3D11_SDK_VERSION,
        Some(&mut device),None,Some(&mut context))}
        .map_err(|e|format!("Hardware D3D11 device: {e}"))?;
    let device=device.ok_or("Missing native hardware device")?;
    let context=context.ok_or("Missing native hardware context")?;
    let mut pixels=vec![0u8;(SIZE*SIZE*4) as usize];
    for y in 0..SIZE{
        for x in 0..SIZE{
            let j=((y*SIZE+x)*4) as usize;
            pixels[j]=(x*3+25) as u8;
            pixels[j+1]=(y*3+28) as u8;
            pixels[j+2]=((x^y)*2+36) as u8;
            pixels[j+3]=255;
        }
    }
    let texture_desc=D3D11_TEXTURE2D_DESC{
        Width:SIZE,Height:SIZE,MipLevels:1,ArraySize:1,
        Format:DXGI_FORMAT_R8G8B8A8_UNORM,
        SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
        Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,
        CPUAccessFlags:0,MiscFlags:0,
    };
    let initial=D3D11_SUBRESOURCE_DATA{
        pSysMem:pixels.as_ptr() as *const c_void,
        SysMemPitch:SIZE*4,SysMemSlicePitch:SIZE*SIZE*4,
    };
    let mut source:Option<ID3D11Texture2D>=None;
    unsafe{device.CreateTexture2D(&texture_desc,Some(&initial),Some(&mut source))}
        .map_err(|e|format!("Synthetic proof texture upload: {e}"))?;
    let source=source.ok_or("Missing synthetic GPU texture")?;
    // Compile the four distinct native HLSL programs once and keep only
    // two ping-pong GPU textures for the entire 30-frame proof.
    let cached=crate::native_pipeline::NativeGpuPipeline::new(&device,SIZE,SIZE)?;
    assert_eq!(cached.dimensions(),(SIZE,SIZE));
    let factory:IDXGIFactory2=unsafe{CreateDXGIFactory1()}
        .map_err(|e|format!("Create DXGI 1.2 factory: {e}"))?;
    let desc=swapchain_descriptor(SIZE,SIZE);
    let chain=unsafe{factory.CreateSwapChainForHwnd(&device,HWND(window.hwnd),
        &desc,None,None::<&IDXGIOutput>)}
        .map_err(|e|format!("Flip-model CreateSwapChainForHwnd: {e}"))?;
    let back:ID3D11Texture2D=unsafe{chain.GetBuffer(0)}
        .map_err(|e|format!("Get native swapchain backbuffer: {e}"))?;
    let back_resource:ID3D11Resource=back.cast()
        .map_err(|e|format!("Backbuffer cast: {e}"))?;
    let mut status=windows::core::HRESULT(0);let mut submitted=0u32;
    for _frame in 0..30 {
        let processed=cached.dispatch(&device,&context,&source,&[0,1,2,3])?;
        let processed_resource:ID3D11Resource=processed.cast()
            .map_err(|e|format!("Processed GPU texture cast: {e}"))?;
        unsafe{context.CopyResource(&back_resource,&processed_resource);}
        crate::native_filter::verify_gpu_completion(&device,&context)?;
        status=unsafe{chain.Present(0,DXGI_PRESENT(0))};
        status.ok().map_err(|e|format!("Native flip Present: {e}"))?;
        if crate::present_status::classify(status.0)==crate::present_status::Outcome::Submitted{submitted+=1;}
    }
    Ok(NativePresentProof{
        width:SIZE,height:SIZE,shader_passes:4,
        swap_buffers:desc.BufferCount,present_hresult:status.0,
        submitted_frames:submitted,
        shader_programs_compiled_once:cached.compiled_count(),
        persistent_gpu_surfaces:cached.scratch_texture_count(),
    })
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn descriptor_is_flip_discard_not_legacy_blt(){
        let desc=swapchain_descriptor(512,256);
        assert_eq!(desc.Width,512);
        assert_eq!(desc.Height,256);
        assert_eq!(desc.BufferCount,2);
        assert_eq!(desc.SwapEffect,DXGI_SWAP_EFFECT_FLIP_DISCARD);
        assert_eq!(desc.Flags,0);
    }
    #[test]fn hidden_proof_does_not_claim_game_fullscreen(){
        let p=NativePresentProof{
            width:64,height:64,shader_passes:4,swap_buffers:2,present_hresult:0,
            submitted_frames:30,shader_programs_compiled_once:12,persistent_gpu_surfaces:2
        };
        assert!(p.summary().contains("VISIBLE_GAME_OUTPUT=false"));
        assert!(!p.summary().contains("EXCLUSIVE_GAME_FILTER=true"));
    }
}

#[cfg(test)]
#[test]
fn direct_composition_hidden_synthetic_submission(){
    let win=HiddenProofWindow::create().unwrap();
    let g=crate::photo_lab::Gpu::new().unwrap();
    let factory:IDXGIFactory2=unsafe{CreateDXGIFactory1()}.unwrap();
    let mut output=crate::composition_output::Output::new(&g.device,&factory,HWND(win.hwnd),64,64).unwrap();
    output.fit((96,96)).unwrap();
    let im=image::RgbaImage::from_pixel(64,64,image::Rgba([32,120,210,255]));
    let src=g.upload(&im).unwrap();
    let pipeline=crate::native_pipeline::NativeGpuPipeline::new(&g.device,64,64).unwrap();
    let image=pipeline.dispatch(&g.device,&g.ctx,&src,&[0,2]).unwrap();
    let back:ID3D11Texture2D=unsafe{output.chain.GetBuffer(0)}.unwrap();
    let dst:ID3D11Resource=back.cast().unwrap();
    let src:ID3D11Resource=image.cast().unwrap();
    unsafe{g.ctx.CopyResource(&dst,&src);}
    crate::native_filter::verify_gpu_completion(&g.device,&g.ctx).unwrap();
    let result=unsafe{output.chain.Present(0,DXGI_PRESENT(0))};
    assert!(result.is_ok(),"Hidden DirectComposition Present {result:?}");
}
