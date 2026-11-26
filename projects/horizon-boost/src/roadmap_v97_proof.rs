//! Source-level proof of real D3D11 split composition on captured pixel textures.
#[cfg(test)]mod tests{
 use crate::{photo_lab::Gpu,native_pipeline::NativeGpuPipeline,native_filter::PROGRAMS};
 #[test]fn live_ab_split_left_original_right_gpu_effect_with_alpha(){
  let gpu=Gpu::new().unwrap();
  let width=93;let height=51;
  let im=image::RgbaImage::from_fn(width,height,|x,y|{
    image::Rgba([((x*7+y*5)%130+100)as u8,((x*2+y*3)%110+70)as u8,(x+y+37)as u8,((x+y)%190+25)as u8])
  });
  let source=gpu.upload(&im).unwrap();
  let pipe=NativeGpuPipeline::new(&gpu.device,width,height).unwrap();
  let stage=66usize;
  assert!(PROGRAMS[stage].0.contains("ACES"));
  let full=gpu.read(&pipe.dispatch(&gpu.device,&gpu.ctx,&source,&[stage]).unwrap()).unwrap();
  assert_ne!(full,im);
  let mut settings=crate::lighting::LightTuning::default();
  settings.layers.split=Some(0.5);
  let split=gpu.read(&pipe.dispatch_with_cached_srv_tuned(&gpu.ctx,&source,
    &{
      let mut view=None;
      unsafe{gpu.device.CreateShaderResourceView(&source,None,Some(&mut view))}.unwrap();
      view.unwrap()
    },
    &[stage],&settings).unwrap()).unwrap();
  for y in [0,1,25,50]{
   for x in [0,10,46]{
    let expected=if x<46{im.get_pixel(x,y)}else{full.get_pixel(x,y)};
    assert_eq!(split.get_pixel(x,y),expected,"split coordinate={x},{y}");
   }
   for x in [47,73,92]{assert_eq!(split.get_pixel(x,y),full.get_pixel(x,y));}
  }
  for (a,b) in split.pixels().zip(im.pixels()){assert_eq!(a[3],b[3]);}
  let differences=full.pixels().zip(im.pixels()).filter(|(a,b)|a!=b).count();
  assert!(differences>500);
 }
}
