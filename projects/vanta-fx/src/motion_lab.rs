use crate::{photo_lab::Gpu,framegen::FlowInterpolator,motion_options::Options};
pub struct ResultFrames{pub frames:Vec<image::RgbaImage>,pub estimate_ms:Option<f64>,pub warp_ms:Option<f64>}
fn fixture(shift:u32)->image::RgbaImage{
 image::RgbaImage::from_fn(320,180,|x,y|{
  let u=(x+320-shift)%320;let v=((u*13+y*7+(u/12+y/10)*31)%180+30) as u8;
  let object=u>80&&u<160&&y>40&&y<120;
  let hud=y>150&&(x/6)%2==0;
  if hud{image::Rgba([230,230,240,255])}
  else if object{image::Rgba([v,70,180,255])}
  else{image::Rgba([v,v.saturating_sub(25),v/2,255])}
 })
}
pub fn render(options:&Options)->Result<ResultFrames,String>{
 let gpu=Gpu::new()?;let prev=fixture(0);let now=fixture(6);
 let a=gpu.upload(&prev)?;let b=gpu.upload(&now)?;
 let fg=FlowInterpolator::new(&gpu.device,320,180)?;
 fg.save_real_frame(&gpu.ctx,&a);
 let estimate_ms=crate::effect_bench::measure(&gpu.device,&gpu.ctx,||fg.estimate(&gpu.device,&gpu.ctx,&b).map(|_|()))?;
 let pair=fg.estimate(&gpu.device,&gpu.ctx,&b)?;
 let warp_ms=crate::effect_bench::measure(&gpu.device,&gpu.ctx,||fg.synth_tuned(&gpu.ctx,&pair,2,1,options,0).map(|_|()))?;
 let mid=gpu.read(fg.synth_tuned(&gpu.ctx,&pair,2,1,options,0)?)?;
 let rejection=gpu.read(fg.synth_tuned(&gpu.ctx,&pair,2,1,options,1)?)?;
 let vectors=gpu.read(fg.synth_tuned(&gpu.ctx,&pair,2,1,options,2)?)?;
 Ok(ResultFrames{frames:vec![prev,now,mid,rejection,vectors],estimate_ms,warp_ms})
}
pub fn proof()->Result<String,String>{
 let options=Options{mask:true,left:0.,top:0.8,right:1.,bottom:1.,smooth_flow:true,..Default::default()};
 let r=render(&options)?;
 let current=&r.frames[1];let mid=&r.frames[2];
 let mut outside=0;let mut inside=0;
 for y in 0..180{for x in 0..320{
  let same=current.get_pixel(x,y)==mid.get_pixel(x,y);
  if y>=144{if !same{return Err("GPU HUD mask did not retain current pixels".into());}inside+=1;}
  else if !same{outside+=1;}
 }}
 if outside<100||r.estimate_ms.is_none()||r.warp_ms.is_none(){return Err("Motion lab did not generate measurable output".into());}
 let reject=&r.frames[3];let vector=&r.frames[4];
 if reject.get_pixel(0,179).0!=[26,77,255,255]&&reject.get_pixel(0,179)[2]!=255{return Err("HUD mask diagnostic missing".into());}
 if vector.pixels().all(|v|*v==*vector.get_pixel(0,0)){return Err("Motion vector diagnostic is uniform".into());}
 let strict=render(&Options{max_motion:1.,..options.clone()})?;
 let changes=strict.frames[2].pixels().zip(mid.pixels()).filter(|(a,b)|a!=b).count();
 if changes==0{return Err("High-motion rejection control did not affect output".into());}
 Ok(format!("MOTION_LAB=PASS MASK_EXACT_PIXELS={inside} OUTSIDE_CHANGED={outside} MOTION_LIMIT_CHANGED={changes} ESTIMATE_GPU_MS={:.4} WARP_GPU_MS={:.4}",r.estimate_ms.unwrap(),r.warp_ms.unwrap()))
}


pub fn temporal_proof()->Result<String,String>{
 let gpu=Gpu::new()?;let prev=fixture(0);let mut now=fixture(6);
 // Tiny alternating spatial noise gives the history stabilization real work.
 for (x,y,p) in now.enumerate_pixels_mut(){if y<144{let d=if (x+y)%2==0{7i16}else{-7};for ch in 0..3{p[ch]=(p[ch] as i16+d).clamp(0,255) as u8;}}}
 let a=gpu.upload(&prev)?;let b=gpu.upload(&now)?;let fg=FlowInterpolator::new(&gpu.device,320,180)?;
 fg.save_real_frame(&gpu.ctx,&a);let pair=fg.estimate(&gpu.device,&gpu.ctx,&b)?;
 let opts=Options{mask:true,left:0.,top:0.8,right:1.,bottom:1.,smooth_flow:true,temporal_mode:1,temporal_strength:0.5,..Default::default()};
 let mut changed=[0usize;2];
 for mode in 1..=2{
  let out=gpu.read(fg.temporal_tuned(&gpu.ctx,&pair,&Options{temporal_mode:mode,..opts.clone()})?)?;
  for y in 0..180{for x in 0..320{let same=out.get_pixel(x,y)==now.get_pixel(x,y);if y>=144&&!same{return Err("Temporal HUD mask not exact".into());}if y<144&&!same{changed[mode as usize-1]+=1;}}}
  if changed[mode as usize-1]<100{return Err(format!("Temporal mode {mode} had no measurable effect"));}
  let zero=gpu.read(fg.temporal_tuned(&gpu.ctx,&pair,&Options{temporal_mode:mode,temporal_strength:0.,..opts.clone()})?)?;
  if zero!=now{return Err("Temporal strength zero not exact".into());}
 }
 let black=image::RgbaImage::from_pixel(320,180,image::Rgba([0,0,0,255]));
 let white=image::RgbaImage::from_pixel(320,180,image::Rgba([255,255,255,255]));
 let x=gpu.upload(&black)?;let y=gpu.upload(&white)?;fg.save_real_frame(&gpu.ctx,&x);let pair=fg.estimate(&gpu.device,&gpu.ctx,&y)?;
 for mode in 1..=2{if gpu.read(fg.temporal_tuned(&gpu.ctx,&pair,&Options{temporal_mode:mode,..opts.clone()})?)?!=white{return Err("Temporal scene-cut rejection not exact".into());}}
 Ok(format!("TEMPORAL_EFFECTS=PASS HISTORY_CHANGED={} BLUR_CHANGED={} MASK_EXACT=true ZERO_EXACT=true CUT_EXACT=true",changed[0],changed[1]))
}

pub fn grid_proof()->Result<String,String>{
 let gpu=Gpu::new()?;let prev=fixture(0);let now=fixture(6);
 let a=gpu.upload(&prev)?;let b=gpu.upload(&now)?;
 let mut images=Vec::new();let mut timings=Vec::new();
 for tile in [16,32]{
  let fg=FlowInterpolator::new_with_tile(&gpu.device,320,180,tile)?;
  if fg.tile()!=tile{return Err("Grid selection did not apply".into());}
  fg.save_real_frame(&gpu.ctx,&a);
  let ms=crate::effect_bench::measure(&gpu.device,&gpu.ctx,||fg.estimate(&gpu.device,&gpu.ctx,&b).map(|_|()))?.ok_or("GPU timing unavailable")?;
  let pair=fg.estimate(&gpu.device,&gpu.ctx,&b)?;
  let opts=Options{mask:true,top:0.8,smooth_flow:true,..Default::default()};
  let mid=gpu.read(fg.synth_tuned(&gpu.ctx,&pair,2,1,&opts,0)?)?;
  let changed=mid.pixels().zip(now.pixels()).filter(|(a,b)|a!=b).count();
  if changed<100{return Err("Grid did not produce interpolated pixels".into());}
  for y in 144..180{for x in 0..320{if mid.get_pixel(x,y)!=now.get_pixel(x,y){return Err("Grid HUD mask changed".into());}}}
  images.push(mid);timings.push(ms);
 }
 let changed=images[0].pixels().zip(images[1].pixels()).filter(|(a,b)|a!=b).count();
 if changed<100{return Err("Coarse and fine GPU grids produced indistinguishable images".into());}
 let (w,h)=(2560,1440);
 let large=|shift:u32|image::RgbaImage::from_fn(w,h,|x,y|{
  let u=(x+w-shift)%w;let v=((u*13+y*7+(u/12+y/10)*31)%180+30) as u8;
  image::Rgba([v,v.saturating_sub(25),v/2,255])
 });
 let a=gpu.upload(&large(0))?;let b=gpu.upload(&large(6))?;
 let mut large_ms=Vec::new();
 for tile in [16,32]{
  let fg=FlowInterpolator::new_with_tile(&gpu.device,w,h,tile)?;
  fg.save_real_frame(&gpu.ctx,&a);let _=fg.estimate(&gpu.device,&gpu.ctx,&b)?;
  let mut samples=Vec::new();
  for _ in 0..5{let ms=crate::effect_bench::measure(&gpu.device,&gpu.ctx,||fg.estimate(&gpu.device,&gpu.ctx,&b).map(|_|()))?.ok_or("1440p field timing unavailable")?;samples.push(ms);}
  samples.sort_by(f64::total_cmp);large_ms.push(samples[2]);
 }
 Ok(format!("FLOW_GRID=PASS TILE16_FIELD=20x12 TILE32_FIELD=10x6 CHANGED={changed} FINE_GPU_MS={:.4} COARSE_GPU_MS={:.4} MASK_EXACT=true 1440P_ESTIMATE_FINE_MEDIAN_MS={:.4} 1440P_ESTIMATE_COARSE_MEDIAN_MS={:.4} SAMPLES_PER_MODE=5",timings[0],timings[1],large_ms[0],large_ms[1]))

}

pub fn adaptive_proof()->Result<String,String>{
 let gpu=Gpu::new()?;let a=gpu.upload(&fixture(0))?;let now=fixture(6);let b=gpu.upload(&now)?;let fg=FlowInterpolator::new(&gpu.device,320,180)?;fg.save_real_frame(&gpu.ctx,&a);let pair=fg.estimate(&gpu.device,&gpu.ctx,&b)?;
 let mut guard=crate::framegen::CapturePacingGuard::default();let mut factor=1;for _ in 0..12{factor=guard.observe_factor(3,16.,true);}
 if factor!=2{return Err("Adaptive 3x to 2x selection failed".into());}
 let opts=Options{mask:true,top:0.8,adaptive_fg:true,..Default::default()};
 let mid=gpu.read(fg.synth_tuned(&gpu.ctx,&pair,factor,1,&opts,0)?)?;let delta=mid.pixels().zip(now.pixels()).filter(|(a,b)|a!=b).count();if delta<100{return Err("Adaptive output unchanged".into());}
 for _ in 0..12{factor=guard.observe_factor(3,9.,true);}if factor!=3{return Err("Adaptive recovery to 3x failed".into());}
 let first=gpu.read(fg.synth_tuned(&gpu.ctx,&pair,factor,1,&opts,0)?)?;let second=gpu.read(fg.synth_tuned(&gpu.ctx,&pair,factor,2,&opts,0)?)?;
 if first==second||first==mid||second==mid{return Err("Adaptive phases not distinct".into());}
 for img in [&mid,&first,&second]{for y in 144..180{for x in 0..320{if img.get_pixel(x,y)!=now.get_pixel(x,y){return Err("Adaptive FG HUD not exact".into());}}}}
 Ok(format!("ADAPTIVE_FG=PASS MID_CHANGED={delta} DISTINCT_PHASES=true HUD_EXACT=true SYNTHETIC_ONLY=true"))
}
