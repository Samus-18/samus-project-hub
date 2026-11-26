use crate::{photo_lab::Gpu,effect_bench::{Row,measure},native_pipeline::NativeGpuPipeline,spatial::NativeSpatialScaler,framegen::FlowInterpolator};
use windows::{core::Interface,Win32::Graphics::Direct3D11::ID3D11Resource};
pub fn run(stack:&[usize])->Result<Vec<Row>,String>{
 crate::native_stack::validate(stack)?;let g=Gpu::new()?;let mut rows=Vec::new();
 for(w,h)in [(1280,720),(1920,1080),(2560,1440)]{
  let image=|shift:u32|image::RgbaImage::from_fn(w,h,|x,y|{let u=(x+shift)%w;image::Rgba([(u%251) as u8,(y%241) as u8,((u^y)%233) as u8,255])});
  let a=g.upload(&image(0))?;let b=g.upload(&image(4))?;
  let ar:ID3D11Resource=a.cast().map_err(|e|e.to_string())?;let br:ID3D11Resource=b.cast().map_err(|e|e.to_string())?;
  let pipe=NativeGpuPipeline::new(&g.device,w,h)?;
  let mut small=NativeSpatialScaler::new(&g.device,w/2,h/2)?;
  let mut large=NativeSpatialScaler::new(&g.device,w,h)?;
  let fg=FlowInterpolator::new(&g.device,w,h)?;
  let mut add=|name:&str,value:Option<f64>|rows.push(Row{label:name.into(),width:w,height:h,gpu_ms:value});
  add("GPU resource copy (not WGC)",measure(&g.device,&g.ctx,||{unsafe{g.ctx.CopyResource(&ar,&br);}Ok(())})?);
  add("Selected shader stack",measure(&g.device,&g.ctx,||pipe.dispatch(&g.device,&g.ctx,&a,stack).map(|_|()))?);
  add("Area downsample to half dimensions",measure(&g.device,&g.ctx,||small.render_area(&g.device,&g.ctx,&a).map(|_|()))?);
  let reduced=small.render_area(&g.device,&g.ctx,&a)?;
  add("FSR1 reconstruct",measure(&g.device,&g.ctx,||large.render(&g.device,&g.ctx,&reduced,8).map(|_|()))?);
  add("NIS reconstruct",measure(&g.device,&g.ctx,||large.render(&g.device,&g.ctx,&reduced,9).map(|_|()))?);
  add("SGSR1 reconstruct",measure(&g.device,&g.ctx,||large.render(&g.device,&g.ctx,&reduced,13).map(|_|()))?);
  add("Anime4K CNN S reconstruct",measure(&g.device,&g.ctx,||large.render(&g.device,&g.ctx,&reduced,14).map(|_|()))?);
  let prev=g.upload(&image(0))?;fg.save_real_frame(&g.ctx,&prev);
  add("Flow estimate + cut detector",measure(&g.device,&g.ctx,||fg.estimate(&g.device,&g.ctx,&b).map(|_|()))?);
  let pair=fg.estimate(&g.device,&g.ctx,&b)?;
  add("Single synthetic warp",measure(&g.device,&g.ctx,||fg.synth(&g.ctx,&pair,2,1).map(|_|()))?);
  add("Filter + area + reconstruct",measure(&g.device,&g.ctx,||{
   let f=pipe.dispatch(&g.device,&g.ctx,&a,stack)?;let s=small.render_area(&g.device,&g.ctx,&f)?;large.render(&g.device,&g.ctx,&s,8).map(|_|())
  })?);
 }Ok(rows)
}
pub fn proof()->Result<String,String>{
 let r=run(&[37,39])?;if r.len()!=30||r.iter().any(|v|v.gpu_ms.is_none()){return Err("Pipeline timestamp proof incomplete".into());}
 Ok(format!("PIPELINE_BENCH=PASS ROWS={} RESOLUTIONS=3 SYNTHETIC_ONLY=true\n{}",r.len(),serde_json::to_string(&r).map_err(|e|e.to_string())?))
}
