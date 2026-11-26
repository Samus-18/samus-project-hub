//! Horizon-owned D3D11 spatial scaler. No external Magpie/Lossless dependency.
//! Exact output-size compute shader, starts from captured color-only texture.
//! If captured window already equals display resolution, this cannot produce
//! missing game geometry or make the game itself render fewer pixels.
use windows::{Win32::Graphics::{
    Direct3D11::{
        ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,
        ID3D11UnorderedAccessView,ID3D11ShaderResourceView,
        ID3D11ComputeShader,ID3D11ClassLinkage,D3D11_TEXTURE2D_DESC,
        D3D11_BIND_UNORDERED_ACCESS,D3D11_BIND_SHADER_RESOURCE,
        D3D11_USAGE_DEFAULT,
    },
    Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_SAMPLE_DESC}
 }};
const SHADER:&str=include_str!("../shaders/native/HB_SpatialScaler.hlsl");
pub const MODES:[&str;17]=[
    "Off / native pixels","Bilinear (smooth)","Bicubic (Catmull-Rom)",
    "Lanczos2 (crisp)","Edge-Adaptive (balanced)","Nearest (pixel art)",
    "Mitchell (natural soft)","Bounded Edge (clean sharp)",
    "AMD FSR1 (EASU + RCAS)","NVIDIA NIS (AMD GPU)",
    "Lanczos3 (high quality, costly)","AMD FidelityFX CAS (single pass)", "Integer fit (pixel perfect)","Qualcomm SGSR1 (spatial)","Anime4K CNN S (exact 2x)","CuNNy Veryfast CNN (exact 2x, GPLv3)","FSRCNNX x2 8-0-4-1 CNN (GPLv3, high GPU cost)"
];
pub const RATIOS:[f32;4]=[1.0,1.25,1.5,2.0];
pub fn clamp_ratio(ratio:f32)->f32{
    if !ratio.is_finite(){return 1.0;}
    RATIOS.iter().copied().min_by(|a,b|{
        (ratio-*a).abs().total_cmp(&(ratio-*b).abs())
    }).unwrap_or(1.0)
}
pub fn target_size(src:(u32,u32),screen:(u32,u32),requested:f32,composited:bool)->(u32,u32){
    if composited{return src;}
    let (w,h)=src;
    if w==0||h==0{return src;}
    let r=clamp_ratio(requested)
        .min(screen.0.max(w) as f32/w as f32)
        .min(screen.1.max(h) as f32/h as f32);
    ((w as f32*r).round() as u32,(h as f32*r).round() as u32)
}
pub fn needs_scaling(src:(u32,u32),dst:(u32,u32),mode:u8)->bool{src!=dst||mode==11}
// Single decision shared by the worker and scaler, so displayed algorithm matches the GPU.
pub fn effective_mode(src:(u32,u32),dst:(u32,u32),requested:u8)->u8{
 if !needs_scaling(src,dst,requested){return 0;}
 if requested==0||requested as usize>=MODES.len(){return 1;}
 if matches!(requested,8|9|13)&&(dst.0<src.0||dst.1<src.1||dst.0>src.0.saturating_mul(2)||dst.1>src.1.saturating_mul(2)){return 1;}
 if requested==11&&!crate::cas::supported(src,dst){return 1;}
 if requested==14&&(dst!=(src.0.saturating_mul(2),src.1.saturating_mul(2))||u64::from(src.0)*u64::from(src.1)>8_294_400){return 1;}
 if requested==15&&(dst!=(src.0.saturating_mul(2),src.1.saturating_mul(2))||u64::from(src.0)*u64::from(src.1)>4_194_304){return 1;}
 if requested==16&&(dst!=(src.0.saturating_mul(2),src.1.saturating_mul(2))||u64::from(src.0)*u64::from(src.1)>2_073_600){return 1;}
 requested
}
pub fn compile_all()->Result<usize,String>{
    for mode in 0..10{
        let source=format!("#define SCALE_MODE {mode}\n{SHADER}");
        crate::shadercheck::compile_native_hlsl(&source)?;
    }
    Ok(10)
}
pub struct NativeSpatialScaler{
    pub output_size:(u32,u32),
    shaders:Vec<Option<ID3D11ComputeShader>>,
    result:ID3D11Texture2D,
    target_uav:ID3D11UnorderedAccessView,
    // Lazily allocated only for users choosing exact official AMD FSR1.
    // The original seven modes do not spend memory on its extra surfaces.
    fsr1:Option<crate::fsr1::NativeFsr1Upscaler>,
    nis:Option<crate::nis::NativeNisScaler>,
    nis_input:Option<(u32,u32)>,
    mip:Option<crate::reconstruction::MipChain>,
    view_cache:crate::reconstruction::ViewCache,
    cas:Option<ID3D11ComputeShader>,
    sgsr:Option<crate::sgsr::Pass>,
    anime4k:Option<crate::anime4k::Pass>,
    cunny:Option<crate::cunny::Pass>,
    fsrcnnx:Option<crate::fsrcnnx::Pass>,
}
impl NativeSpatialScaler{
    pub fn new(device:&ID3D11Device,w:u32,h:u32)->Result<Self,String>{
        if w==0||h==0||w>8192||h>8192{
            return Err("Requested output size outside safe 8192px limit".into());
        }
        let desc=D3D11_TEXTURE2D_DESC{
            Width:w,Height:h,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
            Usage:D3D11_USAGE_DEFAULT,
            BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,
            CPUAccessFlags:0,MiscFlags:0,
        };
        let mut texture=None;
        unsafe{device.CreateTexture2D(&desc,None,Some(&mut texture))}
            .map_err(|e|format!("Native scaler output: {e}"))?;
        let result=texture.ok_or("Native spatial GPU output missing")?;
        let mut target_uav=None;
        unsafe{device.CreateUnorderedAccessView(&result,None,Some(&mut target_uav))}
            .map_err(|e|format!("Scaler UAV: {e}"))?;
        let target_uav=target_uav.ok_or("Scaler UAV missing")?;
        let shaders=vec![None;10];
        Ok(Self{output_size:(w,h),shaders,result,target_uav,fsr1:None,nis:None,nis_input:None,mip:None,view_cache:Default::default(),cas:None,sgsr:None,anime4k:None,cunny:None,fsrcnnx:None})
    }
    pub fn output_texture(&self)->&ID3D11Texture2D{&self.result}
    /// Internal downsampling, not an extra public upscaler mode.
    pub fn render_area(&mut self,device:&ID3D11Device,ctx:&ID3D11DeviceContext,
        input:&ID3D11Texture2D)->Result<ID3D11Texture2D,String>{
        self.mip=None;self.render(device,ctx,input,255)
    }
    pub fn render_mip(&mut self,device:&ID3D11Device,ctx:&ID3D11DeviceContext,input:&ID3D11Texture2D)->Result<ID3D11Texture2D,String>{
        if self.mip.as_ref().is_none_or(|m|!m.compatible(input)){self.mip=Some(crate::reconstruction::MipChain::new(device,input)?);}
        self.mip.as_ref().ok_or("Mip chain unavailable")?.render(ctx,input,&self.target_uav,self.output_size.0,self.output_size.1)?;
        Ok(self.result.clone())
    }
    pub fn render(&mut self,device:&ID3D11Device,ctx:&ID3D11DeviceContext,
        input:&ID3D11Texture2D,mode:u8)->Result<ID3D11Texture2D,String>{
        let mut size=D3D11_TEXTURE2D_DESC::default();unsafe{input.GetDesc(&mut size);}
        // Keep the last vendor scaler across basic modes, but never retain both.
        if mode==9{self.fsr1=None;}
        if mode==8{self.nis=None;self.nis_input=None;}
        let mode=if mode==255{255}else{effective_mode((size.Width,size.Height),self.output_size,mode)};
        if mode!=14{self.anime4k=None;}
        if mode!=15{self.cunny=None;}
        if mode!=16{self.fsrcnnx=None;}
        if mode==16{
            self.fsr1=None;self.nis=None;self.nis_input=None;
            let dims=(size.Width,size.Height);
            if self.fsrcnnx.as_ref().is_none_or(|v|!v.compatible(dims)){
                self.fsrcnnx=Some(crate::fsrcnnx::Pass::new(device,dims)?);
            }
            self.fsrcnnx.as_mut().ok_or("FSRCNNX neural scaler missing")?
                .render(device,ctx,input,&self.target_uav)?;
            return Ok(self.result.clone());
        }
        if mode==15{
            self.fsr1=None;self.nis=None;self.nis_input=None;
            let dims=(size.Width,size.Height);
            if self.cunny.as_ref().is_none_or(|v|!v.compatible(dims)){
                self.cunny=Some(crate::cunny::Pass::new(device,dims)?);
            }
            self.cunny.as_mut().ok_or("CuNNy native neural scaler missing")?
                .render(device,ctx,input,&self.target_uav)?;
            return Ok(self.result.clone());
        }
        if mode==14{
            self.fsr1=None;self.nis=None;self.nis_input=None;
            let dims=(size.Width,size.Height);
            if self.anime4k.as_ref().is_none_or(|a|!a.compatible(dims)){self.anime4k=Some(crate::anime4k::Pass::new(device,dims)?);}
            let srv=self.view_cache.get(device,input)?;self.anime4k.as_ref().ok_or("Anime4K missing")?.render(ctx,&srv,&self.target_uav);
            return Ok(self.result.clone());
        }
        if mode==13{
            if self.sgsr.is_none(){self.sgsr=Some(crate::sgsr::Pass::new(device)?);}
            let srv=self.view_cache.get(device,input)?;
            self.sgsr.as_ref().ok_or("SGSR1 missing")?.render(ctx,&srv,&self.target_uav,self.output_size.0,self.output_size.1);
            return Ok(self.result.clone());
        }
        if mode==9{
            if self.nis.is_none()||self.nis_input!=Some((size.Width,size.Height)){
                self.nis=Some(crate::nis::NativeNisScaler::new(
                    device,(size.Width,size.Height),self.output_size)?);
                self.nis_input=Some((size.Width,size.Height));
            }
            return self.nis.as_ref().unwrap().render(device,ctx,input);
        }
        if mode==8{
            if self.fsr1.is_none(){
                self.fsr1=Some(crate::fsr1::NativeFsr1Upscaler::new(
                    device,self.output_size.0,self.output_size.1)?);
            }
            return self.fsr1.as_ref().unwrap().render(device,ctx,input);
        }
        // UI mode 0 is native; when actual source and target differ use
        // conservative bilinear. UI 1/2/3/4/5 maps to compute 0/1/2/3/4.
        // effective_mode already validates vendor scale bounds.
        if mode==11&&self.cas.is_none(){
            let bytes=crate::shadercheck::compile_native_hlsl(&crate::cas::shader_source())?;
            let mut shader=None;unsafe{device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|e.to_string())?;
            self.cas=Some(shader.ok_or("CAS shader missing")?);
        }
        let index=if mode==255{8usize}else if mode==10{7usize}else if mode==12{9usize}else{mode.saturating_sub(1).min(6) as usize};
        if mode!=11&&self.shaders[index].is_none(){
            let source=format!("#define SCALE_MODE {index}\n{SHADER}");
            let data=crate::shadercheck::compile_native_hlsl(&source)?;
            let mut shader=None;
            unsafe{device.CreateComputeShader(&data,None::<&ID3D11ClassLinkage>,Some(&mut shader))}
                .map_err(|e|format!("Native scaler shader {index}: {e}"))?;
            self.shaders[index]=Some(shader.ok_or("Native scaler shader missing")?);
        }
        let shader=if mode==11{self.cas.as_ref().ok_or("CAS missing")?}else{self.shaders[index].as_ref().ok_or("Scaler shader missing")?};
        let source_srv=self.view_cache.get(device,input)?;
        unsafe{
            ctx.CSSetShader(shader,None);
            ctx.CSSetShaderResources(0,Some(&[Some(source_srv)]));
            let uav=[Some(self.target_uav.clone())];
            ctx.CSSetUnorderedAccessViews(0,1,Some(uav.as_ptr()),None);
            ctx.Dispatch(self.output_size.0.div_ceil(8),self.output_size.1.div_ceil(8),1);
            let empty=[None];
            ctx.CSSetUnorderedAccessViews(0,1,Some(empty.as_ptr()),None);
            ctx.CSSetShaderResources(0,Some(&[None]));
            ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
        }
        Ok(self.result.clone())
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn all_five_native_spatial_shaders_compile(){
        assert_eq!(compile_all().unwrap(),10);
        assert_eq!(MODES.len(),17);
    }
    #[test]fn scaling_respects_real_monitor_and_same_rectangle_glass(){
        assert_eq!(target_size((1280,720),(1920,1080),1.5,false),(1920,1080));
        assert_eq!(target_size((1920,1080),(1920,1080),2.0,false),(1920,1080));
        assert_eq!(target_size((960,540),(1920,1080),2.0,false),(1920,1080));
        assert_eq!(target_size((1280,720),(1920,1080),1.5,true),(1280,720));
        assert_eq!(clamp_ratio(f32::NAN),1.0);
    }
}

pub fn integer_proof()->Result<String,String>{
 let g=crate::photo_lab::Gpu::new()?;let im=image::RgbaImage::from_fn(7,5,|x,y|image::Rgba([(x*30)as u8,(y*47)as u8,137,255]));
 let src=g.upload(&im)?;let mut s=NativeSpatialScaler::new(&g.device,19,13)?;
 let actual=g.read(&s.render(&g.device,&g.ctx,&src,12)?)?;
 for(x,y,p)in actual.enumerate_pixels(){let expected=if (2..16).contains(&x)&&(1..11).contains(&y){*im.get_pixel((x-2)/2,(y-1)/2)}else{image::Rgba([0,0,0,255])};if *p!=expected{return Err(format!("Integer-fit mismatch at {x},{y}: {p:?} != {expected:?}"));}}
 let mut s2=NativeSpatialScaler::new(&g.device,14,10)?;let a=g.read(&s2.render(&g.device,&g.ctx,&src,12)?)?;
 if a.enumerate_pixels().any(|(x,y,p)|p!=im.get_pixel(x/2,y/2)){return Err("Exact 2x integer duplication failed".into());}
 let mut tiny=NativeSpatialScaler::new(&g.device,3,3)?;let a=g.read(&tiny.render(&g.device,&g.ctx,&src,12)?)?;let b=g.read(&tiny.render(&g.device,&g.ctx,&src,5)?)?;if a!=b{return Err("Integer downsize fallback mismatch".into());}
 Ok("INTEGER_GPU=PASS ODD_BORDERS_EXACT=true DOUBLE_EXACT=true SMALL_NEAREST_EXACT=true".into())
}

#[cfg(test)]mod live_scaler_tests{use super::*;
 #[test]fn native_size_only_cas_requires_pass(){assert!(!needs_scaling((1920,1080),(1920,1080),8));assert!(needs_scaling((1920,1080),(1920,1080),11));assert!(needs_scaling((960,540),(1920,1080),0));}
 #[test]fn lazy_shader_initialization_and_reuse_on_synthetic_gpu(){let g=crate::photo_lab::Gpu::new().unwrap();let im=image::RgbaImage::from_pixel(16,16,image::Rgba([50,100,150,255]));let src=g.upload(&im).unwrap();let mut s=NativeSpatialScaler::new(&g.device,32,32).unwrap();assert!(s.shaders.iter().all(Option::is_none));s.render(&g.device,&g.ctx,&src,1).unwrap();assert_eq!(s.shaders.iter().filter(|x|x.is_some()).count(),1);s.render(&g.device,&g.ctx,&src,5).unwrap();assert_eq!(s.shaders.iter().filter(|x|x.is_some()).count(),2);s.render(&g.device,&g.ctx,&src,1).unwrap();assert_eq!(s.shaders.iter().filter(|x|x.is_some()).count(),2);}
}

#[cfg(test)]mod effective_mode_tests{
 use super::*;
 #[test]fn reports_bypass_and_bilinear_for_native_and_invalid(){assert_eq!(effective_mode((1920,1080),(1920,1080),8),0);assert_eq!(effective_mode((960,540),(1920,1080),0),1);assert_eq!(effective_mode((960,540),(1920,1080),99),1);}
 #[test]fn bounds_vendor_algorithms_and_keeps_cas_native(){for m in [8,9,11]{assert_eq!(effective_mode((640,360),(1920,1080),m),1);assert_eq!(effective_mode((1920,1080),(960,540),m),1);assert_eq!(effective_mode((960,540),(1920,1080),m),m);}assert_eq!(effective_mode((1920,1080),(1920,1080),11),11);}
 #[test]fn invalid_vendor_ratio_fallback_matches_bilinear_gpu(){let g=crate::photo_lab::Gpu::new().unwrap();let src=g.upload(&image::RgbaImage::from_fn(11,7,|x,y|image::Rgba([(x*19)as u8,(y*29)as u8,55,255]))).unwrap();let mut s=NativeSpatialScaler::new(&g.device,35,23).unwrap();let a=g.read(&s.render(&g.device,&g.ctx,&src,1).unwrap()).unwrap();for m in [8,9,11]{let b=g.read(&s.render(&g.device,&g.ctx,&src,m).unwrap()).unwrap();assert_eq!(a,b);}}
}
