//! Native Windows.Graphics.Capture window-to-GPU path.
//!
//! This module captures only the HWND explicitly selected by the user.
//! No game DLL, no Magpie, no automatic process modifications. Source pixels
//! never map to CPU; the captured D3D11 resource is copied on the GPU while
//! the WinRT frame is still alive. An own output renderer is still required.
use windows::{
    core::Interface,
    Foundation::TypedEventHandler,
    Graphics::{
        Capture::{GraphicsCaptureItem,Direct3D11CaptureFramePool,GraphicsCaptureSession,
                 IDirect3D11CaptureFramePoolStatics2,IGraphicsCaptureSessionStatics},
        DirectX::{DirectXPixelFormat,Direct3D11::IDirect3DDevice},
        SizeInt32,
    },
    Win32::{
        Foundation::HWND,
        Graphics::{
            Direct3D11::{
                D3D11_TEXTURE2D_DESC,D3D11_BIND_SHADER_RESOURCE,D3D11_USAGE_DEFAULT,
                D3D11_QUERY_DESC,D3D11_QUERY_EVENT,
                ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11Resource,
                ID3D11ShaderResourceView,ID3D11Query,
            },
            Dxgi::IDXGIDevice,
        },
        System::WinRT::{
            Direct3D11::{CreateDirect3D11DeviceFromDXGIDevice,IDirect3DDxgiInterfaceAccess},
            Graphics::Capture::IGraphicsCaptureItemInterop,
        },
    },
};
use std::sync::{Arc,Mutex,Condvar,atomic::{AtomicU64,Ordering}};
// Resolve WinRT initializers dynamically; GNU Rust's MinGW toolchain does
// not ship Microsoft's runtimeobject.lib import library.
#[link(name="kernel32")]
unsafe extern "system"{
    fn LoadLibraryW(name:*const u16)->*mut std::ffi::c_void;
    fn GetProcAddress(module:*mut std::ffi::c_void,name:*const u8)
        ->*const std::ffi::c_void;
    fn FreeLibrary(module:*mut std::ffi::c_void)->i32;
}
type RoInit=unsafe extern "system" fn(u32)->i32;
type RoDone=unsafe extern "system" fn();
type RoGetFactory=unsafe extern "system" fn(*mut std::ffi::c_void,*const windows::core::GUID,*mut *mut std::ffi::c_void)->i32;
type CreateHString=unsafe extern "system" fn(*const u16,u32,*mut *mut std::ffi::c_void)->i32;
type DeleteHString=unsafe extern "system" fn(*mut std::ffi::c_void)->i32;
#[derive(Clone,Copy)]
struct WinRtFns{initialize:RoInit,done:RoDone,get_factory:RoGetFactory,
 create_hstring:CreateHString,delete_hstring:DeleteHString}
static WINRT_FNS:std::sync::OnceLock<Result<WinRtFns,String>>=std::sync::OnceLock::new();
fn winrt_exports()->Result<WinRtFns,String>{
    WINRT_FNS.get_or_init(||{
        let library="combase.dll".encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
        let module=unsafe{LoadLibraryW(library.as_ptr())};
        if module.is_null(){return Err("Cannot load WinRT combase.dll".into());}
        let setup=unsafe{GetProcAddress(module,b"RoInitialize\0".as_ptr())};
        let finish=unsafe{GetProcAddress(module,b"RoUninitialize\0".as_ptr())};
        let get_factory=unsafe{GetProcAddress(module,b"RoGetActivationFactory\0".as_ptr())};
        let create_hstring=unsafe{GetProcAddress(module,b"WindowsCreateString\0".as_ptr())};
        let delete_hstring=unsafe{GetProcAddress(module,b"WindowsDeleteString\0".as_ptr())};
        if setup.is_null()||finish.is_null()||get_factory.is_null()||
           create_hstring.is_null()||delete_hstring.is_null(){
            unsafe{FreeLibrary(module);}
            return Err("Windows RoInitialize/RoUninitialize API unavailable".into());
        }
        // The single retained combase reference is process-lifetime by
        // design: WinRT asynchronous callbacks can still be in flight when a
        // capture session is closed. Do not LoadLibrary again on each start.
        Ok(WinRtFns{
            initialize:unsafe{std::mem::transmute(setup)},
            done:unsafe{std::mem::transmute(finish)},
            get_factory:unsafe{std::mem::transmute(get_factory)},
            create_hstring:unsafe{std::mem::transmute(create_hstring)},
            delete_hstring:unsafe{std::mem::transmute(delete_hstring)},
        })
    }).clone()
}
struct WinRtApartment{done:RoDone}
impl WinRtApartment {
    fn new()->Result<Self,String>{
        let fns=winrt_exports()?;
        let hr=unsafe{(fns.initialize)(1)};
        if hr<0{
            return Err(format!("WGC WinRT apartment setup failed: 0x{:08x}",hr as u32));
        }
        Ok(Self{done:fns.done})
    }
}
impl Drop for WinRtApartment{
    fn drop(&mut self){unsafe{(self.done)();}}
}
// WinRT activation factories may be cached by windows-rs. Tearing down the
// apartment between WGC restarts in the SAME thread invalidates that cache.
// Keep the apartment initialized until the whole capture worker thread exits.
thread_local!{
    static THREAD_WINRT:Result<WinRtApartment,String>=WinRtApartment::new();
}
fn require_thread_winrt()->Result<(),String>{
    THREAD_WINRT.with(|r|r.as_ref().map(|_|()).map_err(Clone::clone))
}

// Do not use windows-rs global FactoryCache for capture WinRT classes.
// An earlier WGC worker can exit and RoUninitialize its apartment. Global
// factory pointers retained from that apartment can then crash the *next*
// capture worker (confirmed with two consecutive HWND fixtures on this PC).
// Acquire COM factories in the current WinRT apartment on every start.
fn fresh_factory<T:windows::core::Interface>(name:&str)->Result<T,String>{
    require_thread_winrt()?;
    let f=winrt_exports()?;
    let wide=name.encode_utf16().collect::<Vec<u16>>();
    let mut hstring=std::ptr::null_mut::<std::ffi::c_void>();
    let created=unsafe{(f.create_hstring)(wide.as_ptr(),wide.len()as u32,&mut hstring)};
    if created<0||hstring.is_null(){return Err(format!("WinRT WindowsCreateString {name}: 0x{:08x}",created as u32));}
    let mut ptr=std::ptr::null_mut::<std::ffi::c_void>();
    let status=unsafe{(f.get_factory)(hstring,&T::IID,&mut ptr)};
    unsafe{(f.delete_hstring)(hstring);}
    if status<0||ptr.is_null(){return Err(format!("WinRT factory {name}: 0x{:08x}",status as u32));}
    Ok(unsafe{T::from_raw(ptr)})
}
fn capture_supported()->Result<bool,String>{
    let statics:IGraphicsCaptureSessionStatics=fresh_factory(
        "Windows.Graphics.Capture.GraphicsCaptureSession")?;
    let mut supported=false;
    unsafe{(windows::core::Interface::vtable(&statics).IsSupported)(
        windows::core::Interface::as_raw(&statics),&mut supported)}
        .ok().map_err(|e|format!("WGC IsSupported: {e}"))?;
    Ok(supported)
}
fn free_threaded_pool(device:&IDirect3DDevice,size:SizeInt32)
    ->Result<Direct3D11CaptureFramePool,String>{
    let statics:IDirect3D11CaptureFramePoolStatics2=fresh_factory(
        "Windows.Graphics.Capture.Direct3D11CaptureFramePool")?;
    let mut raw=std::ptr::null_mut();
    unsafe{(windows::core::Interface::vtable(&statics).CreateFreeThreaded)(
        windows::core::Interface::as_raw(&statics),
        windows::core::Interface::as_raw(device),
        DirectXPixelFormat::B8G8R8A8UIntNormalized,3,size,&mut raw)}
        .ok().map_err(|e|format!("WGC free-threaded pool factory: {e}"))?;
    if raw.is_null(){return Err("WGC pool factory returned null".into());}
    Ok(unsafe{Direct3D11CaptureFramePool::from_raw(raw)})
}


pub struct WgcCapture {
    pub device:ID3D11Device,
    context:ID3D11DeviceContext,
    _item:GraphicsCaptureItem,
    pool:Direct3D11CaptureFramePool,
    session:GraphicsCaptureSession,
    frame_arrived_token:i64,
    arrivals:Arc<AtomicU64>,
    wake:Arc<(Mutex<()>,Condvar)>,
    native:crate::native_pipeline::NativeGpuPipeline,
    downscale:Option<crate::spatial::NativeSpatialScaler>,
    reduced_srv:Option<ID3D11ShaderResourceView>,
    dropped:u64,
    processed_size:(u32,u32),
    scratch:ID3D11Texture2D,
    scratch_resource:ID3D11Resource,
    scratch_srv:ID3D11ShaderResourceView,
    analyzer:Option<crate::capture_sample::Analyzer>,
    sample:Option<crate::capture_sample::Sample>,
    sample_failed:bool,
    capture_age:Option<f64>,
    capture_stamp:Option<i64>,
    last_valid_stamp:Option<i64>,
    duplicate_frames:u64,
    stale_pre_filter:u64,
    pressure:crate::render_pressure::StaleGate,
    copy_timer:crate::live_gpu_timing::Sampler,
    filter_timer:crate::live_gpu_timing::Sampler,
    copy_complete:ID3D11Query,
    copy_fence:Option<crate::gpu_copy_fence::CopyFence>,
    used_fence:bool,
    width:u32,
    height:u32,
    frames:u64,
    // WinRT apartment is kept thread-local, and tears down only at thread exit.
}

impl WgcCapture {
    /// Starts capturing exactly the Windows HWND specified by user. The API
    /// can refuse protected windows; we report that rather than bypassing.
    pub fn for_window(window:HWND)->Result<Self,String>{Self::for_window_scaled(window,100)}
    pub fn for_window_scaled(window:HWND,percent:u8)->Result<Self,String>{Self::for_window_scaled_on(window,percent,None)}
    pub fn for_window_scaled_on(window:HWND,percent:u8,adapter:Option<&str>)->Result<Self,String>{
        if window.0.is_null(){return Err("WGC requires a valid window handle".into());}
        require_thread_winrt()?;
        if !capture_supported()?{
            return Err("This Windows build does not support GraphicsCaptureSession".into());
        }
        let factory:IGraphicsCaptureItemInterop=fresh_factory(
            "Windows.Graphics.Capture.GraphicsCaptureItem")?;
        let item:GraphicsCaptureItem=unsafe{factory.CreateForWindow(window)}
            .map_err(|e|format!("WGC refused target window: {e}"))?;
        let size=item.Size().map_err(|e|format!("WGC window size: {e}"))?;
        if size.Width<1||size.Height<1||size.Width>8192||size.Height>8192{
            return Err(format!("WGC window size invalid {}x{}",size.Width,size.Height));
        }
        let(device,context)=crate::adapter_select::create(adapter)?;
        let dxgi:IDXGIDevice=device.cast().map_err(|e|format!("WGC DXGI device cast: {e}"))?;
        let rt_device:IDirect3DDevice=unsafe{CreateDirect3D11DeviceFromDXGIDevice(&dxgi)}
            .map_err(|e|format!("WGC WinRT D3D11 bridge: {e}"))?
            .cast().map_err(|e|format!("WGC WinRT device cast: {e}"))?;
        let pool=free_threaded_pool(&rt_device,
            SizeInt32{Width:size.Width,Height:size.Height})?;
        let arrivals=Arc::new(AtomicU64::new(0));
        let signal=Arc::clone(&arrivals);
        let wake=Arc::new((Mutex::new(()),Condvar::new()));
        let notify=Arc::clone(&wake);
        let handler=TypedEventHandler::new(move |_,_|{
            // Bounded pool has three slots; this is a notification hint, not a frame count.
            let _guard=notify.0.lock().unwrap_or_else(|e|e.into_inner());
            let _=signal.try_update(Ordering::AcqRel,Ordering::Acquire,|n|Some(n.saturating_add(1).min(3)));
            notify.1.notify_one();
            Ok(())
        });
        let frame_arrived_token=pool.FrameArrived(&handler)
            .map_err(|e|format!("WGC FrameArrived registration: {e}"))?;
        let session=pool.CreateCaptureSession(&item)
            .map_err(|e|format!("WGC session: {e}"))?;
        let (width,height)=(size.Width as u32,size.Height as u32);
        let desc=D3D11_TEXTURE2D_DESC{
            Width:width,Height:height,MipLevels:1,ArraySize:1,
            Format:windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc:windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC{Count:1,Quality:0},
            Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,
            CPUAccessFlags:0,MiscFlags:0,
        };
        let mut scratch=None;
        unsafe{device.CreateTexture2D(&desc,None,Some(&mut scratch))}
            .map_err(|e|format!("WGC VRAM source target: {e}"))?;
        let scratch=scratch.ok_or("WGC scratch texture missing")?;
        let mut srv=None;
        unsafe{device.CreateShaderResourceView(&scratch,None,Some(&mut srv))}
            .map_err(|e|format!("WGC reusable source SRV: {e}"))?;
        let scratch_srv=srv.ok_or("WGC reusable source SRV unavailable")?;
        let scratch_resource:ID3D11Resource=scratch.cast().map_err(|e|format!("WGC reusable resource: {e}"))?;
        let mut query=None;
        unsafe{device.CreateQuery(&D3D11_QUERY_DESC{
            Query:D3D11_QUERY_EVENT,MiscFlags:0
        },Some(&mut query))}
            .map_err(|e|format!("WGC persistent GPU completion query: {e}"))?;
        let copy_complete=query.ok_or("WGC persistent GPU query unavailable")?;
        let copy_fence=crate::gpu_copy_fence::CopyFence::new(&device,&context);
        let processed_size=crate::render_budget::size((width,height),percent);
        let downscale=if processed_size!=(width,height){
            Some(crate::spatial::NativeSpatialScaler::new(&device,processed_size.0,processed_size.1)?)
        }else{None};
        let reduced_srv=if let Some(scaler)=downscale.as_ref(){
            let mut view=None;
            unsafe{device.CreateShaderResourceView(scaler.output_texture(),None,Some(&mut view))}
                .map_err(|e|format!("Persistent reduced GPU SRV: {e}"))?;
            Some(view.ok_or("Reduced SRV missing")?)
        }else{None};
        let native=crate::native_pipeline::NativeGpuPipeline::new(&device,processed_size.0,processed_size.1)?;
        native.enable_fusion();
        session.StartCapture().map_err(|e|format!("WGC StartCapture: {e}"))?;
        Ok(Self{device,context,_item:item,pool,session,
            frame_arrived_token,arrivals,wake,native,downscale,reduced_srv,dropped:0,processed_size,scratch,scratch_resource,scratch_srv,copy_complete,copy_fence,used_fence:false,
            width,height,frames:0,analyzer:None,sample:None,sample_failed:false,capture_age:None,capture_stamp:None,last_valid_stamp:None,duplicate_frames:0,stale_pre_filter:0,pressure:Default::default(),copy_timer:Default::default(),filter_timer:Default::default()})
    }

    pub fn backlog(&self)->u64{self.arrivals.load(Ordering::Acquire)}
    pub fn wait_for_frame(&self,timeout:std::time::Duration){
        let guard=self.wake.0.lock().unwrap_or_else(|e|e.into_inner());
        let _result=self.wake.1.wait_timeout_while(guard,timeout,|_|self.arrivals.load(Ordering::Acquire)==0);
    }
    pub fn dimensions(&self)->(u32,u32){(self.width,self.height)}
    pub fn processed_size(&self)->(u32,u32){self.processed_size}
    pub fn context(&self)->&ID3D11DeviceContext{&self.context}
    pub fn discarded_frames(&self)->u64{self.dropped}
    pub fn frame_stamp(&self)->Option<i64>{self.capture_stamp}
    pub fn duplicate_frames(&self)->u64{self.duplicate_frames}
    pub fn stale_pre_filter(&self)->u64{self.stale_pre_filter}
    pub fn set_pressure_cap(&mut self,cap:Option<u32>){self.pressure.set_cap(cap);}
    pub fn set_min_interval(&self,ticks:i64)->bool{
        self.session.SetMinUpdateInterval(windows::Foundation::TimeSpan{Duration:ticks}).is_ok()
    }
    pub fn capture_age(&self)->Option<f64>{self.capture_age}
    pub fn current_capture_age(&self)->Option<f64>{self.capture_stamp.and_then(|s|crate::capture_clock::qpc_100ns().and_then(|n|crate::capture_clock::age_ms(s,n)))}
    pub fn filter_reading(&self,now:std::time::Instant)->Option<crate::live_gpu_timing::Reading>{self.filter_timer.reading(now)}
    pub fn phase_times(&self)->[Option<f64>;2]{let n=std::time::Instant::now();[self.copy_timer.value(n),self.filter_timer.value(n)]}
    pub fn optional_warning(&self)->u32{self.native.optional_warning()}
    pub fn fused_saved(&self)->u32{self.native.fused_saved()}
    pub fn live_look_rejected(&self)->bool{self.native.live_rejected()}
    pub fn live_active_count(&self)->usize{self.native.live_active_count()}
    pub fn used_copy_fence(&self)->bool{self.used_fence}
    pub fn sample_status(&self)->(Option<crate::capture_sample::Sample>,bool){(self.sample,self.sample_failed)}

    /// Returns NONE when no new desktop-composited frame is available.
    /// If the captured window resizes, return a REBUILD error so caller drops
    /// the old resources and recreates them in a clean state.
    pub fn next_filtered_frame(&mut self,stages:&[usize],
        light:&crate::lighting::LightTuning)
        ->Result<Option<ID3D11Texture2D>,String>{
        // Never invoke TryGetNextFrame against an empty free-threaded WinRT
        // queue. On some Polaris/Windows driver combinations this was seen to
        // AV during the very first compositor tick.
        if self.arrivals.try_update(Ordering::AcqRel,Ordering::Acquire,
            |n|n.checked_sub(1)).is_err(){
            return Ok(None);
        }
        let mut frame=match self.pool.TryGetNextFrame(){
            Ok(value)=>value,
            Err(err) if err.code().0==0x80004005u32 as i32 =>return Ok(None),
            Err(err)=>return Err(format!("WGC next frame unavailable: {err}")),
        };
        // Drain at most two *actually available* older frames. Never wait on
        // an empty queue, and never queue an unbounded backlog of GPU work.
        for _ in 0..2{
            if self.arrivals.try_update(Ordering::AcqRel,Ordering::Acquire,|n|n.checked_sub(1)).is_err(){break;}
            match self.pool.TryGetNextFrame(){
                Ok(newer)=>{drop(frame);frame=newer;self.dropped+=1;},
                Err(err) if err.code().0==0x80004005u32 as i32=>break,
                Err(err)=>return Err(format!("WGC latest-frame dequeue: {err}")),
            }
        }
        let now=std::time::Instant::now();
        let stamp=frame.SystemRelativeTime().ok().map(|v|v.Duration).filter(|v|*v>0);
        if crate::capture_cadence::duplicate(self.last_valid_stamp,stamp){self.duplicate_frames=self.duplicate_frames.saturating_add(1);return Ok(None);}
        self.last_valid_stamp=stamp;
        self.capture_stamp=stamp;
        self.capture_age=self.current_capture_age();
        if self.pressure.skip(self.capture_age,self.backlog()){
            self.stale_pre_filter=self.stale_pre_filter.saturating_add(1);return Ok(None);
        }
        if light.reconstruction.live_gpu_timing{
            self.copy_timer.poll(&self.context,now);self.filter_timer.poll(&self.context,now);
        }else{self.copy_timer.clear();self.filter_timer.clear();}
        let size=frame.ContentSize().map_err(|e|format!("WGC frame dimensions: {e}"))?;
        if size.Width!=self.width as i32||size.Height!=self.height as i32{
            return Err(format!("WGC_REBUILD_REQUIRED: {}x{} -> {}x{}",
                self.width,self.height,size.Width,size.Height));
        }
        let winrt_surface=frame.Surface().map_err(|e|format!("WGC surface: {e}"))?;
        let access:IDirect3DDxgiInterfaceAccess=winrt_surface.cast()
            .map_err(|e|format!("WGC surface DXGI access: {e}"))?;
        let captured:ID3D11Texture2D=unsafe{access.GetInterface()}
            .map_err(|e|format!("WGC D3D11 captured texture: {e}"))?;
        let source_resource:ID3D11Resource=captured.cast()
            .map_err(|e|format!("WGC captured resource: {e}"))?;

        if light.reconstruction.live_gpu_timing{self.copy_timer.begin(&self.device,&self.context,std::time::Instant::now());}
        unsafe{self.context.CopyResource(&self.scratch_resource,&source_resource);}
        self.copy_timer.end(&self.context,std::time::Instant::now());
        // Hold the WGC frame until GPU copy completion. On supported drivers,
        // wait on an event rather than waking the capture thread repeatedly.
        self.used_fence=self.copy_fence.as_mut().is_some_and(|f|f.complete(&self.context));
        if !self.used_fence{
            unsafe{self.context.End(&self.copy_complete);self.context.Flush();}
            let mut completed=false;
            let deadline=std::time::Instant::now()+std::time::Duration::from_millis(400);
            while std::time::Instant::now()<deadline{
                let mut done=0i32;
                unsafe{self.context.GetData(&self.copy_complete,Some(&mut done as *mut i32 as *mut std::ffi::c_void),
                    std::mem::size_of::<i32>() as u32,1)}
                    .map_err(|e|format!("WGC persistent GPU completion query: {e}"))?;
                if done!=0{completed=true;break;}
                std::thread::sleep(std::time::Duration::from_micros(250));
            }
            if !completed{return Err("WGC_REBUILD_REQUIRED: GPU copy completion timeout".into());}
            // A failed event path is not retried every frame.
            self.copy_fence=None;
        }
        drop(frame);
        if light.reconstruction.capture_diagnostic{
            if self.analyzer.is_none()&&!self.sample_failed{
                match crate::capture_sample::Analyzer::new(&self.device){Ok(a)=>self.analyzer=Some(a),Err(_)=>self.sample_failed=true,}
            }
            if let Some(a)=self.analyzer.as_mut(){
                match a.tick(&self.context,&self.scratch_srv){Ok(s)=>self.sample=s,Err(_)=>{self.sample=None;self.sample_failed=true;self.analyzer=None;}}
            }
        }else{self.analyzer=None;self.sample=None;self.sample_failed=false;}
        if light.reconstruction.live_gpu_timing{self.filter_timer.begin(&self.device,&self.context,std::time::Instant::now());}
        self.native.set_grain_phase((self.frames as u32).wrapping_add(1));
        let output=if let Some(scaler)=self.downscale.as_mut(){
            let reduced=if light.reconstruction.mip_prefilter{scaler.render_mip(&self.device,&self.context,&self.scratch)?}else{scaler.render_area(&self.device,&self.context,&self.scratch)?};
            self.native.dispatch_live(&self.context,&reduced,
                self.reduced_srv.as_ref().ok_or("Persistent reduced SRV missing")?,stages,light)?
        }else{
            self.native.dispatch_live(&self.context,&self.scratch,
                &self.scratch_srv,stages,light)?
        };
        self.filter_timer.end(&self.context,std::time::Instant::now());
        self.frames+=1;
        Ok(Some(output))
    }

}
impl Drop for WgcCapture{
    fn drop(&mut self){
        let _=self.pool.RemoveFrameArrived(self.frame_arrived_token);
        let _=self.session.Close();
        let _=self.pool.Close();
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn wgc_rejects_null_target_instead_of_capturing_unrelated_windows(){
        assert!(WgcCapture::for_window(HWND(std::ptr::null_mut())).is_err());
    }
}
