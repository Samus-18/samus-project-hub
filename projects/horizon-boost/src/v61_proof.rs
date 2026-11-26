pub fn proof()->Result<String,String>{
 let g=crate::photo_lab::Gpu::new()?;let im=image::RgbaImage::from_fn(64,32,|x,y|{let v=if x%12<3&&y%12<3{45}else{180+((x+y)%40) as u8};image::Rgba([v,v,v,255])});
 let t=g.upload(&im)?;
 let mut scaler=crate::spatial::NativeSpatialScaler::new(&g.device,128,64)?;
 let a=g.read(&scaler.render(&g.device,&g.ctx,&t,1)?)?;
 let b=g.read(&scaler.render(&g.device,&g.ctx,&t,11)?)?;
 let changed=a.pixels().zip(b.pixels()).filter(|(a,b)|a!=b).count();if changed==0{return Err("CAS equals bilinear".into());}
 let fallback=crate::spatial::NativeSpatialScaler::new(&g.device,192,96)?;
 let mut fallback=fallback;let a=g.read(&fallback.render(&g.device,&g.ctx,&t,1)?)?;let b=g.read(&fallback.render(&g.device,&g.ctx,&t,11)?)?;if a!=b{return Err("CAS scale cap fallback not exact".into());}
 let pipe=crate::native_pipeline::NativeGpuPipeline::new(&g.device,64,32)?;
 let ao=g.read(&pipe.dispatch(&g.device,&g.ctx,&t,&[60])?)?;
 let n=im.pixels().zip(ao.pixels()).filter(|(a,b)|a!=b).count();
 let flat=image::RgbaImage::from_pixel(64,32,image::Rgba([128,128,128,255]));let f=g.upload(&flat)?;
 if n==0||g.read(&pipe.dispatch(&g.device,&g.ctx,&f,&[60])?)?!=flat{return Err("Relief shade not responsive or neutral".into());}
 Ok(format!("V61_GPU=PASS CAS_CHANGED={changed} CAS_CAP_EXACT=true RELIEF_CHANGED={n} FLAT_EXACT=true DEPTH_PROXY_ONLY=true"))
}
