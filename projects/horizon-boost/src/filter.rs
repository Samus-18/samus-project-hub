use crate::settings::FilterProfile;
use std::ffi::c_void;
type Module = *mut c_void;
type Initialize = unsafe extern "system" fn() -> i32;
type Uninitialize = unsafe extern "system" fn() -> i32;
type GetMatrix = unsafe extern "system" fn(*mut f32) -> i32;
type SetMatrix = unsafe extern "system" fn(*const f32) -> i32;
#[link(name="kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(file:*const u16) -> Module;
    fn GetProcAddress(dll:Module,proc:*const u8) -> *const c_void;
    fn FreeLibrary(dll:Module) -> i32;
}
#[derive(Clone, Copy)]
struct Api{initialize:Initialize,uninitialize:Uninitialize,get:GetMatrix,set:SetMatrix}

pub fn matrix(profile:&FilterProfile) -> [f32;25] {
    let mut matrix=[0.0;25];
    let intensity=profile.intensity.clamp(0.0,1.0);
    let sat=1.0+profile.vibrance.clamp(0.0,1.0)*0.8;
    let contrast=1.0+profile.contrast.clamp(-0.5,0.5)*0.72;
    let brightness=profile.brightness.clamp(-0.5,0.5)*0.42;
    let luma=[0.2126,0.7152,0.0722];
    for from in 0..3 {
        for to in 0..3 {
            let one=if from==to {1.0} else {0.0};
            let color=one*sat + luma[from]*(1.0-sat);
            matrix[from*5+to]=one*(1.0-intensity)+color*contrast*intensity;
        }
    }
    let offset=intensity*(0.5*(1.0-contrast)+brightness);
    matrix[20]=offset;matrix[21]=offset;matrix[22]=offset;
    matrix[18]=1.0;matrix[24]=1.0;
    matrix
}
pub struct LiveColor {
    original:[f32;25],
    dll:Module,
    api:Option<Api>,
    initialized:bool,
    pub active:bool,
}
impl Default for LiveColor {
    fn default()->Self{Self{original:[0.0;25],dll:std::ptr::null_mut(),api:None,initialized:false,active:false}}
}
impl LiveColor {
    fn load(&mut self)->Result<Api,String>{
        if let Some(api)=self.api{return Ok(api);}
        let name="Magnification.dll".encode_utf16().chain(std::iter::once(0)).collect::<Vec<_>>();
        // SAFETY: LoadLibraryW, GetProcAddress and FreeLibrary use their documented Win32 signatures.
        unsafe {
            let dll=LoadLibraryW(name.as_ptr());
            if dll.is_null(){return Err("Magnification.dll is not available".into());}
            let load=|symbol:&[u8]| {
                let p=GetProcAddress(dll,symbol.as_ptr());
                if p.is_null(){Err("Magnification API entry point missing".to_string())}
                else{Ok(p)}
            };
            let result=(||{
                Ok(Api{
                    initialize:std::mem::transmute::<*const c_void,Initialize>(load(b"MagInitialize\0")?),
                    uninitialize:std::mem::transmute::<*const c_void,Uninitialize>(load(b"MagUninitialize\0")?),
                    get:std::mem::transmute::<*const c_void,GetMatrix>(load(b"MagGetFullscreenColorEffect\0")?),
                    set:std::mem::transmute::<*const c_void,SetMatrix>(load(b"MagSetFullscreenColorEffect\0")?),
                })
            })();
            match result {
                Ok(api)=>{self.dll=dll;self.api=Some(api);Ok(api)}
                Err(error)=>{FreeLibrary(dll);Err(error)}
            }
        }
    }
    fn initialize_once(&mut self)->Result<(),String>{
        if !self.initialized{
            let api=self.load()?;
            unsafe{
                if (api.initialize)()==0{return Err("Windows Magnification initialization failed".into());}
                if (api.get)(self.original.as_mut_ptr())==0{
                    (api.uninitialize)();
                    return Err("Original desktop color matrix could not be read".into());
                }
            }
            self.initialized=true;
        }
        Ok(())
    }
    /// Safety probe only. Initialize Magnification API, read current native
    /// matrix, and uninitialize. NEVER calls MagSetFullscreenColorEffect.
    pub fn read_only_probe(&mut self)->Result<[f32;25],String>{
        if self.initialized{
            return Err("Read-only probe refused: Horizon already has active color effect".into());
        }
        let api=self.load()?;
        let mut before=[0.0f32;25];
        unsafe{
            if (api.initialize)()==0{return Err("Windows Magnification init refused".into());}
            let ok=(api.get)(before.as_mut_ptr())!=0;
            (api.uninitialize)();
            if !ok{return Err("MagGetFullscreenColorEffect refused read-only request".into());}
        }
        if !before.iter().all(|x|x.is_finite()){
            return Err("Windows returned non-finite color matrix".into());
        }
        Ok(before)
    }
    pub fn enable(&mut self,p:&FilterProfile)->Result<(),String> {
        self.initialize_once()?;
        self.update(p)
    }
    /// Apply a genuine Windows 5x5 display color transform; restore the exact
    /// original matrix on disable/exit. Never changes game swapchains.
    pub fn enable_matrix(&mut self,m:[f32;25])->Result<(),String>{
        if !m.iter().all(|x|x.is_finite()&&x.abs()<=8.0){
            return Err("Unsafe or non-finite color transformation".into());
        }
        self.initialize_once()?;
        self.write_matrix(&m)
    }
    fn write_matrix(&mut self,m:&[f32;25])->Result<(),String>{
        if let Some(api)=self.api{
            if unsafe{(api.set)(m.as_ptr())}==0{
                self.disable();
                return Err("Windows rejected display matrix; original colors restored".into());
            }
            self.active=true;
        }
        Ok(())
    }
    pub fn update(&mut self,p:&FilterProfile)->Result<(),String> {
        if !self.initialized{return Ok(());}
        let m=matrix(p);
        self.write_matrix(&m)
    }
    pub fn disable(&mut self) {
        if self.initialized {
            if let Some(api)=self.api {
                unsafe{(api.set)(self.original.as_ptr());(api.uninitialize)();}
            }
            self.initialized=false;
        }
        self.active=false;
    }
}
impl Drop for LiveColor {
    fn drop(&mut self){
        self.disable();
        if !self.dll.is_null(){unsafe {FreeLibrary(self.dll)};self.dll=std::ptr::null_mut();}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn identity_matrix(){
        let p=FilterProfile{vibrance:0.0,contrast:0.0,brightness:0.0,sharpen:0.0,intensity:1.0,enabled:false};
        let m=matrix(&p);
        for i in 0..5{for j in 0..5{assert!((m[i*5+j]-if i==j{1.0}else{0.0}).abs()<1e-6);}}
    }
    #[test]fn strength_zero_is_identity(){
        let mut p=FilterProfile::default();p.intensity=0.0;
        let m=matrix(&p);
        for i in 0..5{for j in 0..5{assert!((m[i*5+j]-if i==j{1.0}else{0.0}).abs()<1e-6);}}
    }
}