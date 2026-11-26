//! Offline shader compiler smoke tests using Windows' Direct3D HLSL compiler.
 //! The MagpieFX directives are comments to D3DCompile; tests check PS_5_0 shader syntax,
 //! not Magpie's parser/runtime binding.
use std::{ffi::{c_void,CString},slice};
#[link(name="kernel32")]
unsafe extern "system" {
    fn LoadLibraryExW(path:*const u16,file:*mut c_void,flags:u32)->*mut c_void;
    fn GetProcAddress(module:*mut c_void,symbol:*const u8)->*const c_void;
    fn FreeLibrary(module:*mut c_void)->i32;
}
type D3DCompile=unsafe extern "system" fn(
    *const c_void,usize,*const i8,*const c_void,*const c_void,
    *const i8,*const i8,u32,u32,*mut *mut c_void,*mut *mut c_void)->i32;
unsafe fn blob_method(blob:*mut c_void,index:usize)->*const c_void{
    let table=unsafe{*(blob as *const *const *const c_void)};
    unsafe{*table.add(index)}
}
unsafe fn blob_release(blob:*mut c_void) {
    if blob.is_null(){return;}
    let ptr=unsafe{blob_method(blob,2)};
    let f:unsafe extern "system" fn(*mut c_void)->u32=unsafe{std::mem::transmute(ptr)};
    unsafe{f(blob); }
}
unsafe fn blob_string(blob:*mut c_void)->String{
    if blob.is_null(){return String::new();}
    let get:unsafe extern "system" fn(*mut c_void)->*const c_void=
        unsafe{std::mem::transmute(blob_method(blob,3))};
    let size:unsafe extern "system" fn(*mut c_void)->usize=
        unsafe{std::mem::transmute(blob_method(blob,4))};
    let bytes=unsafe{slice::from_raw_parts(get(blob) as *const u8,size(blob))};
    String::from_utf8_lossy(bytes).to_string()
}
pub fn compile_all()->Result<usize,String>{
    let wide="d3dcompiler_47.dll".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let library=unsafe{LoadLibraryExW(wide.as_ptr(),std::ptr::null_mut(),0x800)};
    if library.is_null(){return Err("Windows D3DCompiler 47 not installed".into());}
    let api=unsafe{GetProcAddress(library,b"D3DCompile\0".as_ptr())};
    if api.is_null(){
        unsafe{FreeLibrary(library)};
        return Err("D3DCompile export not found".into());
    }
    let compile:D3DCompile=unsafe{std::mem::transmute(api)};
    let entry=CString::new("HBEntry").unwrap();
    let target=CString::new("ps_5_0").unwrap();
    let mut passed=0;
    for (name,source) in crate::engine::CUSTOM_SHADERS {
        let code=format!(
            "float2 GetInputPt();\n{}\nfloat2 GetInputPt(){{return float2(1.0/1920.0,1.0/1080.0);}}\nfloat4 HBEntry(float2 uv:TEXCOORD0):SV_Target {{return Pass1(uv);}}\n",
            source);
        let mut output:*mut c_void=std::ptr::null_mut();
        let mut error:*mut c_void=std::ptr::null_mut();
        let name_c=CString::new(*name).map_err(|e|e.to_string())?;
        let status=unsafe {
            compile(code.as_ptr() as *const c_void,code.len(),name_c.as_ptr(),
                std::ptr::null(),std::ptr::null(),
                entry.as_ptr(),target.as_ptr(),0,0,&mut output,&mut error)
        };
        if status<0{
            let message=unsafe{blob_string(error)};
            unsafe{blob_release(output);blob_release(error);FreeLibrary(library);}
            return Err(format!("{name}: D3D shader compile failed (0x{:x}): {message}",status as u32));
        }
        unsafe{blob_release(output);blob_release(error);}
        passed+=1;
    }
    unsafe{FreeLibrary(library);}
    Ok(passed)
}
/// Compiles an original native HLSL compute shader to bytes for Direct3D11.
/// Unlike Magpie effect sources, this is a normal cs_5_0 program with explicit SRV/UAV.
type BytecodeCache=std::collections::HashMap<String,Vec<u8>>;
static NATIVE_CACHE:std::sync::OnceLock<std::sync::Mutex<BytecodeCache>>=std::sync::OnceLock::new();
static CACHE_HITS:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
pub fn cache_stats()->(usize,u64){
    let count=NATIVE_CACHE.get().and_then(|c|c.lock().ok().map(|c|c.len())).unwrap_or(0);
    (count,CACHE_HITS.load(std::sync::atomic::Ordering::Relaxed))
}
pub fn compile_native_hlsl(source:&str)->Result<Vec<u8>,String>{
    let cache=NATIVE_CACHE.get_or_init(||std::sync::Mutex::new(BytecodeCache::new()));
    if let Ok(c)=cache.lock(){
        if let Some(bytes)=c.get(source){
            CACHE_HITS.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
            return Ok(bytes.clone());
        }
    }
    let bytes=compile_uncached(source)?;
    if let Ok(mut c)=cache.lock(){
        let total=c.iter().map(|(s,b)|s.len()+b.len()).sum::<usize>();
        if c.len()<128&&total+source.len()+bytes.len()<=16*1024*1024{
            c.entry(source.to_owned()).or_insert_with(||bytes.clone());
        }
    }
    Ok(bytes)
}
fn compile_uncached(source:&str)->Result<Vec<u8>,String>{
    let wide="d3dcompiler_47.dll".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let library=unsafe{LoadLibraryExW(wide.as_ptr(),std::ptr::null_mut(),0x800)};
    if library.is_null(){return Err("D3DCompiler_47.dll unavailable".into());}
    let api=unsafe{GetProcAddress(library,b"D3DCompile\0".as_ptr())};
    if api.is_null(){
        unsafe{FreeLibrary(library);}
        return Err("D3DCompile export missing".into());
    }
    let compiler:D3DCompile=unsafe{std::mem::transmute(api)};
    let entry=CString::new("CSMain").unwrap();
    let target=CString::new("cs_5_0").unwrap();
    let mut compiled:*mut c_void=std::ptr::null_mut();
    let mut errors:*mut c_void=std::ptr::null_mut();
    let result=unsafe{compiler(source.as_ptr() as *const c_void,source.len(),
        c"NativeShader".as_ptr(),std::ptr::null(),std::ptr::null(),
        entry.as_ptr(),target.as_ptr(),0,0,&mut compiled,&mut errors)};
    let info=if result<0{
        Err(format!("D3D11 CS_5_0 HLSL error (0x{:08x}): {}",result as u32,
            unsafe{blob_string(errors)}))
    }else if compiled.is_null(){
        Err("D3DCompile returned no native bytecode".to_string())
    }else{
        let get:unsafe extern "system" fn(*mut c_void)->*const c_void=unsafe{
            std::mem::transmute(blob_method(compiled,3))
        };
        let size:unsafe extern "system" fn(*mut c_void)->usize=unsafe{
            std::mem::transmute(blob_method(compiled,4))
        };
        let bytes=unsafe{slice::from_raw_parts(get(compiled) as *const u8,size(compiled))}
            .to_vec();
        Ok(bytes)
    };
    unsafe{blob_release(compiled);blob_release(errors);FreeLibrary(library);}
    info
}

#[cfg(test)]mod cache_tests{
 use super::*;
 #[test]fn exact_source_reuses_real_compiled_bytecode(){
    let code="[numthreads(1,1,1)] void CSMain(uint3 id:SV_DispatchThreadID) {}";
    let first=compile_native_hlsl(code).unwrap();let hits=cache_stats().1;
    assert_eq!(first,compile_native_hlsl(code).unwrap());
    assert!(cache_stats().1>hits);
 }
}
