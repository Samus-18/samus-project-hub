//! Own synthetic GPU probe for incomplete SDR tone-mapping/AA entries.
#[cfg(test)]mod tests{
 use crate::{photo_lab::Gpu,native_pipeline::NativeGpuPipeline};
 #[test]fn actual_gpu_tone_map_preserves_source_alpha_and_sharpen_changes_edges(){
  let gpu=Gpu::new().unwrap();
  let raw=image::RgbaImage::from_fn(95,63,|x,y|{
   let rgb=if x>=41{[220,190,150]}else{[31,52,78]};
   image::Rgba([rgb[0],rgb[1],rgb[2],(x+y)as u8])
  });
  let tex=gpu.upload(&raw).unwrap();
  let p=NativeGpuPipeline::new(&gpu.device,95,63).unwrap();
  for id in [53,65,66]{
   let out=gpu.read(&p.dispatch(&gpu.device,&gpu.ctx,&tex,&[id]).unwrap()).unwrap();
   assert_eq!(out.dimensions(),raw.dimensions());
   assert!(out.pixels().zip(raw.pixels()).all(|(a,b)|a[3]==b[3]),"Stage {id} destroyed source transparency");
   eprintln!("V96_SHADER id={} warnings={} input={:?} output={:?} pixels_changed={}",id,p.optional_warning(),raw.get_pixel(45,20),out.get_pixel(45,20),out.pixels().zip(raw.pixels()).filter(|(a,b)|a!=b).count());
   if id==65{
    assert_eq!(out.get_pixel(4,4),raw.get_pixel(4,4),"Uniform color must not be sharpened");
    assert!(out.get_pixel(40,30)!=raw.get_pixel(40,30),"Luma edge must sharpen");
   }else{
    assert!(out.pixels().zip(raw.pixels()).filter(|(a,b)|a!=b).count()>10,"SDR tone mapper must change at least 10 pixels");
   }
  }
 }
}
