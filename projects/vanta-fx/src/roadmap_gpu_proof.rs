//! Acceptance tests for the named 200-ID roadmap. Own synthetic GPU textures only.
#[cfg(test)]mod tests{
 use crate::{photo_lab::Gpu,lighting::LightTuning,native_pipeline::NativeGpuPipeline};
 fn img()->image::RgbaImage{image::RgbaImage::from_fn(127,79,|x,y|image::Rgba([((x*7+y*11)%256)as u8,((x*13+y*3)%256)as u8,((x*19+y*17)%256)as u8,255]))}
 #[test]fn optics_and_retro_collection(){
  let g=Gpu::new().unwrap();let im=img();let src=g.upload(&im).unwrap();let p=NativeGpuPipeline::new(&g.device,127,79).unwrap();
  let mut unique=Vec::new();
  for id in [17,19,20,21,23,27,44,48,49,55,58,59,62,63]{
   let out=g.read(&p.dispatch(&g.device,&g.ctx,&src,&[id]).unwrap()).unwrap();
   let changed=out.pixels().zip(im.pixels()).filter(|(a,b)|a!=b).count();
   assert!(changed>if id==55{0}else{100},"stage {id} changed only {changed}");assert_eq!(p.optional_warning(),0,"stage {id} failed");
   assert!(!unique.contains(&out),"duplicate stage {id}");unique.push(out);
   eprintln!("ROADMAP_STAGE {id} CHANGED_PIXELS={changed}");
  }
  assert_eq!(p.scratch_texture_count(),2);
 }
 #[test]fn screen_space_optics_have_expected_spatial_behavior(){
  let g=Gpu::new().unwrap();let im=image::RgbaImage::from_fn(127,79,|x,y|image::Rgba([if (x+y)%2==0{220}else{30},100,150,255]));let src=g.upload(&im).unwrap();let p=NativeGpuPipeline::new(&g.device,127,79).unwrap();
  let tilt=g.read(&p.dispatch(&g.device,&g.ctx,&src,&[27]).unwrap()).unwrap();assert_eq!(tilt.get_pixel(63,37),im.get_pixel(63,37));
  // A constant white reference makes warm color and vignette direction measurable.
  let white=image::RgbaImage::from_pixel(127,79,image::Rgba([200,200,200,255]));let tex=g.upload(&white).unwrap();
  let warm=g.read(&p.dispatch(&g.device,&g.ctx,&tex,&[44]).unwrap()).unwrap();let c=warm.get_pixel(63,39);assert!(c[0]>c[1]&&c[1]>c[2]);
  let vignette=g.read(&p.dispatch(&g.device,&g.ctx,&tex,&[59]).unwrap()).unwrap();assert!(vignette.get_pixel(0,0)[0]<vignette.get_pixel(63,39)[0]);
  // Debanding must smooth weak quantization steps, not strong object boundaries.
  let bands=image::RgbaImage::from_fn(127,79,|x,_|{let v=if x>=100{220}else{80+(x/8)as u8*4};image::Rgba([v,v,v,255])});
  let band_tex=g.upload(&bands).unwrap();
  let deband=g.read(&p.dispatch(&g.device,&g.ctx,&band_tex,&[55]).unwrap()).unwrap();
  let boundary=|im:&image::RgbaImage| (8..96).step_by(8).map(|x|im.get_pixel(x-1,39)[0].abs_diff(im.get_pixel(x,39)[0])as u32).sum::<u32>();
  assert!(boundary(&deband)<boundary(&bands),"weak band edges not reduced");
  assert!(deband.get_pixel(99,39)[0].abs_diff(deband.get_pixel(100,39)[0])>=bands.get_pixel(99,39)[0].abs_diff(bands.get_pixel(100,39)[0]).saturating_sub(1),"strong edge blurred");
  let crt=g.read(&p.dispatch(&g.device,&g.ctx,&tex,&[62]).unwrap()).unwrap();assert_ne!(crt.get_pixel(63,38),crt.get_pixel(63,39));
  let curved=g.read(&p.dispatch(&g.device,&g.ctx,&tex,&[63]).unwrap()).unwrap();assert_eq!(curved.get_pixel(0,0).0,[0,0,0,255]);assert!(curved.get_pixel(63,39)[0]>100);
 }
 #[test]fn live_before_after_split_exact(){
  let g=Gpu::new().unwrap();let im=img();let src=g.upload(&im).unwrap();let p=NativeGpuPipeline::new(&g.device,127,79).unwrap();p.enable_fusion();
  let mut srv=None;unsafe{g.device.CreateShaderResourceView(&src,None,Some(&mut srv))}.unwrap();let srv=srv.unwrap();
  let mut t=LightTuning::default();let ids=[5,1,0];
  let base=g.read(&p.dispatch_live(&g.ctx,&src,&srv,&ids,&t).unwrap()).unwrap();
  assert_ne!(base,im);
  for split in [0.,0.25,0.5,0.75,1.]{t.layers.split=Some(split);let out=g.read(&p.dispatch_live(&g.ctx,&src,&srv,&ids,&t).unwrap()).unwrap();for(x,y,px)in out.enumerate_pixels(){let expected=if (x as f32+0.5)/127.<split{im.get_pixel(x,y)}else{base.get_pixel(x,y)};assert_eq!(px,expected);}}
 }
 #[test]fn gpu_interpolation_and_hud_mask(){let msg=crate::motion_lab::proof().unwrap();eprintln!("{msg}");let msg=crate::motion_lab::adaptive_proof().unwrap();eprintln!("{msg}");}
 #[test]fn gpu_temporal_history_and_cut_rejection(){eprintln!("{}",crate::motion_lab::temporal_proof().unwrap());}
 #[test]fn hidden_flip_model_actual_api(){let p=crate::native_present::probe_hidden_present().unwrap();assert_eq!(p.swap_buffers,2);assert_eq!(p.persistent_gpu_surfaces,2);assert!(p.submitted_frames<=30);eprintln!("{}",p.summary());}
 #[test]fn actual_gpu_phase_profiler(){eprintln!("{}",crate::live_gpu_timing::proof().unwrap());}
 #[test]fn actual_cinema_light_and_capture_protection(){eprintln!("{}",crate::cinema_light::proof().unwrap());eprintln!("{}",crate::native_pipeline::protection_proof().unwrap());}
}

#[cfg(test)]mod phase_acceptance{
 use windows::{core::Interface,Win32::Graphics::Direct3D11::ID3D11Resource};
 use std::time::Instant;
 use crate::{photo_lab::Gpu,live_gpu_timing::Sampler};
 #[test]fn five_actual_gpu_phases_use_nonblocking_sampler(){
  let g=Gpu::new().unwrap();let a=image::RgbaImage::from_fn(320,180,|x,y|image::Rgba([((x*17+y*9)%256)as u8,((x*7+y*11)%256)as u8,90,255]));
  let src=g.upload(&a).unwrap();let copy=g.upload(&image::RgbaImage::new(320,180)).unwrap();
  let src_r:ID3D11Resource=src.cast().unwrap();let dst_r:ID3D11Resource=copy.cast().unwrap();
  let pipeline=crate::native_pipeline::NativeGpuPipeline::new(&g.device,320,180).unwrap();
  let mut scaler=crate::spatial::NativeSpatialScaler::new(&g.device,640,360).unwrap();
  let fg=crate::framegen::FlowInterpolator::new(&g.device,640,360).unwrap();
  let prev=g.upload(&image::RgbaImage::from_pixel(640,360,image::Rgba([32,64,90,255]))).unwrap();
  fg.save_real_frame(&g.ctx,&prev);
  let mut p:[Sampler;5]=Default::default();
  p[0].begin(&g.device,&g.ctx,Instant::now());for _ in 0..8{unsafe{g.ctx.CopyResource(&dst_r,&src_r);}}p[0].end(&g.ctx,Instant::now());
  p[1].begin(&g.device,&g.ctx,Instant::now());let filtered=pipeline.dispatch(&g.device,&g.ctx,&copy,&[7,2]).unwrap();p[1].end(&g.ctx,Instant::now());
  p[2].begin(&g.device,&g.ctx,Instant::now());let scaled=scaler.render(&g.device,&g.ctx,&filtered,1).unwrap();p[2].end(&g.ctx,Instant::now());
  p[3].begin(&g.device,&g.ctx,Instant::now());let pair=fg.estimate(&g.device,&g.ctx,&scaled).unwrap();p[3].end(&g.ctx,Instant::now());
  p[4].begin(&g.device,&g.ctx,Instant::now());let _=fg.synth(&g.ctx,&pair,2,1).unwrap();p[4].end(&g.ctx,Instant::now());
  crate::native_filter::verify_gpu_completion(&g.device,&g.ctx).unwrap();
  let now=Instant::now();for s in &mut p{s.poll(&g.ctx,now);}
  let ms=p.map(|s|s.value(now).expect("actual GPU phase timestamp"));
  assert!(ms.iter().all(|v|*v>0.));eprintln!("FIVE_GPU_PHASES_MS={ms:?}");
 }
 #[test]fn adaptive_runtime_uses_distinct_measurements_and_display_budget(){
  use crate::live_gpu_timing::Reading;
  use std::time::Duration;
  let n=Instant::now();let mut g=crate::preview_pacer::GpuBudget::default();
  let sample=Reading{ms:12.,when:n};assert_eq!(g.observe_reading(n,8,8,60,Some(sample)),8);assert_eq!(g.observe_reading(n+Duration::from_secs(3),8,8,60,Some(sample)),8);
  let next=Reading{ms:12.,when:n+Duration::from_secs(4)};assert_eq!(g.observe_reading(n+Duration::from_secs(4),8,8,60,Some(next)),7);
  let mut fg=crate::framegen::CapturePacingGuard::default();let mut factor=1;
  for _ in 0..12{factor=fg.observe_for_display(3,Some(16.67),17.,Some(120),true);}assert_eq!(factor,2);
  for _ in 0..12{factor=fg.observe_for_display(3,Some(11.11),11.5,Some(270),true);}assert_eq!(factor,3);
  assert_eq!(fg.observe_for_display(3,None,11.,Some(270),true),1);
 }
}
// V91 real synthetic filter acceptance, not a game benchmark.
#[cfg(test)]mod v91_filters{
 #[test]fn reference_fxaa_smooths_edges_keeps_flat_alpha(){
 let g=crate::photo_lab::Gpu::new().unwrap();
 let im=image::RgbaImage::from_fn(97,65,|x,y|{let c=if x>y*4/3+4{[230,210,180,147]}else{[25,35,45,147]};image::Rgba(c)});
 let src=g.upload(&im).unwrap();let p=crate::native_pipeline::NativeGpuPipeline::new(&g.device,97,65).unwrap();
 let a=g.read(&p.dispatch(&g.device,&g.ctx,&src,&[64]).unwrap()).unwrap();
 assert!(a.pixels().all(|x|x[3]==147));assert_eq!(a.get_pixel(0,64),im.get_pixel(0,64));assert_eq!(a.get_pixel(96,0),im.get_pixel(96,0));assert!(a.pixels().zip(im.pixels()).filter(|(a,b)|a!=b).count()>40);
 let single=g.read(&crate::native_filter::dispatch_filter(&g.device,&g.ctx,&src,64).unwrap()).unwrap();assert_eq!(a,single);
 let repeat=g.read(&p.dispatch(&g.device,&g.ctx,&src,&[64]).unwrap()).unwrap();assert_eq!(a,repeat);
 eprintln!("FXAA311_GPU=PASS ALPHA_EXACT=true FLAT_EXACT=true");
 }
 #[test]fn luminance_sharpen_matches_cpu_and_aces_tone_is_monotone(){
 let g=crate::photo_lab::Gpu::new().unwrap();let im=image::RgbaImage::from_fn(31,19,|x,y|{let v=(40+(x*13+y*7)%150)as u8;image::Rgba([v,v,v,181])});let src=g.upload(&im).unwrap();let p=crate::native_pipeline::NativeGpuPipeline::new(&g.device,31,19).unwrap();
 let a=g.read(&p.dispatch(&g.device,&g.ctx,&src,&[65]).unwrap()).unwrap();
 for (x,y,px) in a.enumerate_pixels(){let c=f32::from(im.get_pixel(x,y)[0]);let mut b=0.;for j in -1i32..=1{for i in -1i32..=1{b+=f32::from(im.get_pixel((x as i32+i).clamp(0,30)as u32,(y as i32+j).clamp(0,18)as u32)[0])*(2-i.abs())as f32*(2-j.abs())as f32/16.;}}let expected=(c+((c-b)*0.65).clamp(-0.035*255.,0.035*255.)).round() as u8;assert!(px[0].abs_diff(expected)<=1);assert_eq!(px[3],181);}
 assert_ne!(a,im);
 let ramp=image::RgbaImage::from_fn(256,1,|x,_|image::Rgba([x as u8,x as u8,x as u8,181]));let tex=g.upload(&ramp).unwrap();let pp=crate::native_pipeline::NativeGpuPipeline::new(&g.device,256,1).unwrap();let tone=g.read(&pp.dispatch(&g.device,&g.ctx,&tex,&[66]).unwrap()).unwrap();assert!(tone.pixels().all(|p|p[3]==181));assert!(tone.pixels().zip(tone.pixels().skip(1)).all(|(a,b)|a[0]<=b[0]));assert_ne!(tone,ramp);
 eprintln!("LUMA9_CPU_GPU=PASS ACES_SDR_MONOTONE=PASS");
 }
}


#[cfg(test)]mod v95_tiled_lds_tests{
 #[test]fn shared_cache_matches_cpu_reference_preserves_alpha_and_odd_edges(){
  let g=crate::photo_lab::Gpu::new().unwrap();
  let im=image::RgbaImage::from_fn(35,29,|x,y|{
   let v=((x*37+y*21)%256)as u8;image::Rgba([v,255-v,(x*13%256)as u8,((x+y)%200+30)as u8])
  });
  let input=g.upload(&im).unwrap();
  let pipe=crate::native_pipeline::NativeGpuPipeline::new(&g.device,35,29).unwrap();
  let id=crate::native_filter::TILED_LDS_STAGE;
  let out=g.read(&pipe.dispatch(&g.device,&g.ctx,&input,&[id]).unwrap()).unwrap();
  assert_eq!(out.dimensions(),(35,29));assert_ne!(out,im);
  for(y,x) in [(0,0),(7,8),(28,34),(14,19),(27,31)]{
   let center=im.get_pixel(x,y);let result=out.get_pixel(x,y);
   assert_eq!(center[3],result[3]);
   for c in 0..3{
    let mut blur=0.;
    for dy in -1i32..=1{for dx in -1i32..=1{
     let xx=(x as i32+dx).clamp(0,34)as u32;let yy=(y as i32+dy).clamp(0,28)as u32;
     blur+=f32::from(im.get_pixel(xx,yy)[c])*(2-dx.abs())as f32*(2-dy.abs())as f32/16.;
    }}
    let expected=(f32::from(center[c])*0.35+blur*0.65).round()as u8;
    assert!(result[c].abs_diff(expected)<=1,"({},{}),channel{} expected={} actual={}",x,y,c,expected,result[c]);
   }
  }
  let again=g.read(&pipe.dispatch(&g.device,&g.ctx,&input,&[id]).unwrap()).unwrap();
  assert_eq!(out,again);
  eprintln!("TILED_LDS_GPU=PASS 35x29 CPU_ERROR<=1 ALPHA_EXACT=true");
 }
}
