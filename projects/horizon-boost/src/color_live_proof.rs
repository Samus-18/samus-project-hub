use crate::{photo_lab::Gpu,native_pipeline::NativeGpuPipeline,lighting::LightTuning};
pub fn proof()->Result<String,String>{
 let g=Gpu::new()?;let img=image::RgbaImage::from_fn(32,16,|x,y|image::Rgba([(x*8) as u8,(y*16) as u8,((x+y)*5) as u8,255]));
 let src=g.upload(&img)?;let pipe=NativeGpuPipeline::new(&g.device,32,16)?;
 let mut srv=None;unsafe{g.device.CreateShaderResourceView(&src,None,Some(&mut srv))}.map_err(|e|e.to_string())?;let srv=srv.ok_or("Color proof SRV missing")?;
 let draw=|t:&LightTuning|g.read(&pipe.dispatch_with_cached_srv_tuned(&g.ctx,&src,&srv,&[2],t)?);
 let mut t=LightTuning::default();let base=draw(&t)?;
 t.lut.name=Some("identity".into());if draw(&t)?!=base{return Err("Live identity LUT not exact".into());}
 t.lut.name=Some("inverse".into());let inverse=draw(&t)?;
 for(a,b)in base.pixels().zip(inverse.pixels()){if (0..3).any(|k|a[k].abs_diff(255-b[k])>1){return Err("Live inverse LUT inaccurate".into());}}
 t.lut.strength=0.;if draw(&t)?!=base{return Err("Live LUT bypass not exact".into());}
 t.lut.strength=0.5;let middle=draw(&t)?;
 if middle.pixels().any(|p|(0..3).any(|k|p[k].abs_diff(128)>1)){return Err("Live LUT strength inaccurate".into());}
 t.lut.name=None;t.grade.enabled=true;t.grade.curves[0][2]=0.9;
 let mut profiles=crate::display_color::Profiles::new();
 if !crate::display_color::store(&mut profiles,"test-output",&t.grade){return Err("Output profile save failed".into());}
 let mut override_state=LightTuning::default();let base_state=override_state.clone();
 if !crate::display_color::apply(&mut override_state,&profiles,Some("test-output"))||draw(&override_state)?!=draw(&t)?{return Err("Output grade did not reach actual GPU pipeline".into());}
 let mut fallback=base_state.clone();
 if crate::display_color::apply(&mut fallback,&profiles,Some("different-output"))||draw(&fallback)?!=base{return Err("Output grade leaked to other output".into());}
 let mut grad=LightTuning::default();grad.grade.enabled=true;grad.grade.gradient=[[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]];
 let zero=draw(&grad)?;grad.grade.gradient_mix=1.;let mapped=draw(&grad)?;
 if mapped==zero||mapped.pixels().zip(zero.pixels()).filter(|(a,b)|a!=b).count()<100{return Err("Live gradient inactive".into());}
 grad.grade.gradient_mix=0.;if draw(&grad)?!=zero{return Err("Gradient bypass not exact".into());}
 let curves=draw(&t)?;let changed=curves.pixels().zip(base.pixels()).filter(|(a,b)|a!=b).count();
 if changed<100{return Err("Five-point curve inactive".into());}
 // Compare persistent temporal exposure with immediate exposure after a moderate brightness change.
 let gray=|v:u8|g.upload(&image::RgbaImage::from_pixel(32,16,image::Rgba([v,v,v,255])));
 let dark=gray(64)?;let mid=gray(100)?;
 let temp=NativeGpuPipeline::new(&g.device,32,16)?;
 let render=|tex:&windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,p:&NativeGpuPipeline,t:&LightTuning|->Result<image::RgbaImage,String>{
  let mut s=None;unsafe{g.device.CreateShaderResourceView(tex,None,Some(&mut s))}.map_err(|e|e.to_string())?;
  g.read(&p.dispatch_with_cached_srv_tuned(&g.ctx,tex,&s.ok_or("Exposure SRV missing")?,&[50],t)?)
 };
 let mut e=LightTuning::default();e.exposure.temporal=true;e.exposure.seconds=3.;e.exposure.reset_delta=0.75;
 let immediate=render(&mid,&temp,&LightTuning::default())?;
 let _=render(&dark,&temp,&e)?;let smoothed=render(&mid,&temp,&e)?;
 let v=smoothed.get_pixel(0,0)[0];let reference=immediate.get_pixel(0,0)[0];
 if v<=reference+3{return Err(format!("Temporal exposure not active: {v} vs {reference}"));}
 std::thread::sleep(std::time::Duration::from_millis(550));
 let reset=render(&mid,&temp,&e)?;if reset!=immediate{return Err("Exposure reset after idle failed".into());}
 Ok(format!("LIVE_COLOR=PASS GRADIENT_CHANGED=true GRADIENT_ZERO_EXACT=true DISPLAY_OVERRIDE_EXACT=true DISPLAY_FALLBACK_EXACT=true IDENTITY_EXACT=true INVERSE=true HALF_MIX=true CURVE_CHANGED={changed} TEMPORAL_PIXEL={v} IMMEDIATE_PIXEL={reference} IDLE_RESET=true"))
}
