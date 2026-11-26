use windows::Win32::Graphics::{Direct3D11::*,Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM,DXGI_SAMPLE_DESC}};
pub fn proof()->Result<String,String>{
 let g=crate::photo_lab::Gpu::new()?;
 let w=64;let h=64;
 let img=image::RgbaImage::from_fn(w,h,|x,y|image::Rgba([if (x+y)%2==0{0}else{255},(x*4) as u8,(y*4) as u8,255]));
 let input=g.upload(&img)?;let mut scale=crate::spatial::NativeSpatialScaler::new(&g.device,32,32)?;
 let filtered=g.read(&scale.render_mip(&g.device,&g.ctx,&input)?)?;
 if filtered.pixels().any(|p|p[0].abs_diff(128)>2){return Err("Mip prefilter did not average checkerboard".into());}
 let mut bgra=img.clone().into_raw();for p in bgra.chunks_exact_mut(4){p.swap(0,2);}
 let d=D3D11_TEXTURE2D_DESC{Width:w,Height:h,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_B8G8R8A8_UNORM,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,CPUAccessFlags:0,MiscFlags:0};
 let init=D3D11_SUBRESOURCE_DATA{pSysMem:bgra.as_ptr().cast(),SysMemPitch:w*4,SysMemSlicePitch:w*h*4};
 let mut t=None;unsafe{g.device.CreateTexture2D(&d,Some(&init),Some(&mut t))}.map_err(|e|e.to_string())?;let t=t.ok_or("BGRA proof texture missing")?;
 let mut srv=None;unsafe{g.device.CreateShaderResourceView(&t,None,Some(&mut srv))}.map_err(|e|e.to_string())?;
 let pipe=crate::native_pipeline::NativeGpuPipeline::new(&g.device,w,h)?;
 let mut light=crate::lighting::LightTuning::default();light.layers.strength.insert(2,0.);
 let result=g.read(&pipe.dispatch_with_cached_srv_tuned(&g.ctx,&t,&srv.ok_or("BGRA proof SRV missing")?,&[2],&light)?)?;
 if result!=img{return Err("All-bypassed BGRA->RGBA channel conversion incorrect".into());}
 let mip_bgra=g.read(&scale.render_mip(&g.device,&g.ctx,&t)?)?;
 if mip_bgra!=filtered{return Err("BGRA mip output differs".into());}
 let pattern=image::RgbaImage::from_fn(w,h,|x,y|{let v=(100+((x*17+y*7)%47)) as u8;image::Rgba([v,v,v,255])});
 let src=g.upload(&pattern)?;let mut sharp=crate::reconstruction::Sharpen::new(&g.device,w,h)?;
 if g.read(&sharp.render(&g.device,&g.ctx,&src,0.)?)?!=pattern{return Err("Sharpen zero not exact".into());}
 let out=g.read(&sharp.render(&g.device,&g.ctx,&src,1.)?)?;
 let changed=out.pixels().zip(pattern.pixels()).filter(|(a,b)|a!=b).count();if changed<100{return Err("Sharpen not active".into());}
 Ok(format!("RECONSTRUCTION=PASS MIP_RGBA=true MIP_BGRA=true BYPASS_BGRA_EXACT=true SHARPEN_CHANGED={changed}"))
}
