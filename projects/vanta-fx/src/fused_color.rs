//! Single GPU dispatch for adjacent full-strength pixel-local color operations.
//! The shader preserves original stage order and intermediate UNORM quantization.
use windows::{core::Interface,Win32::Graphics::Direct3D11::*};
pub fn eligible(i:usize)->bool{matches!(i,0|1|3|4|5|6|7|8|9|11)}
pub fn run_length(ids:&[usize],start:usize,layers:&crate::layer_mix::Layers)->usize{
 ids.iter().enumerate().skip(start).take_while(|(i,id)|eligible(**id)&&layers.enabled_at(ids,*i)==1.).count()
}
pub struct Pass{shader:ID3D11ComputeShader,constants:ID3D11Buffer}
impl Pass{
 pub fn new(device:&ID3D11Device)->Result<Self,String>{
  let bytes=crate::shadercheck::compile_native_hlsl(include_str!("../shaders/native/HB_FusedColor.hlsl"))?;
  let mut shader=None;unsafe{device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|e.to_string())?;
  let mut buffer=None;unsafe{device.CreateBuffer(&D3D11_BUFFER_DESC{ByteWidth:48,Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0},None,Some(&mut buffer))}.map_err(|e|e.to_string())?;
  Ok(Self{shader:shader.ok_or("Fused shader missing")?,constants:buffer.ok_or("Fused constants missing")?})
 }
 pub fn run(&self,ctx:&ID3D11DeviceContext,input:&ID3D11ShaderResourceView,target:&ID3D11UnorderedAccessView,w:u32,h:u32,ids:&[usize])->Result<(),String>{
  if !(2..=8).contains(&ids.len())||ids.iter().any(|&i|!eligible(i)){return Err("Invalid fused color sequence".into());}
  let mut args=[0u32;12];for(i,&id)in ids.iter().enumerate(){args[i]=id as u32;}args[8]=ids.len() as u32;
  let cb:ID3D11Resource=self.constants.cast().map_err(|e|e.to_string())?;
  unsafe{
   ctx.UpdateSubresource(&cb,0,None,args.as_ptr().cast(),0,0);
   ctx.CSSetConstantBuffers(0,Some(&[Some(self.constants.clone())]));
   ctx.CSSetShader(&self.shader,None);ctx.CSSetShaderResources(0,Some(&[Some(input.clone())]));
   let out=[Some(target.clone())];ctx.CSSetUnorderedAccessViews(0,1,Some(out.as_ptr()),None);
   ctx.Dispatch(w.div_ceil(8),h.div_ceil(8),1);
   ctx.CSSetShaderResources(0,Some(&[None]));ctx.CSSetConstantBuffers(0,Some(&[None]));
   let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
   ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
  }Ok(())
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn respects_neighbor_order_and_partial_strength(){
  let mut l=crate::layer_mix::Layers::default();let ids=[5,1,0,2,7,8];
  assert_eq!(run_length(&ids,0,&l),3);assert_eq!(run_length(&ids,3,&l),0);assert_eq!(run_length(&ids,4,&l),2);
  l.set_at(&ids,1,0.5);assert_eq!(run_length(&ids,0,&l),1);assert_eq!(run_length(&ids,1,&l),0);
 }
 #[test]fn fused_real_gpu_matches_reference_and_preserves_alpha(){
  let g=crate::photo_lab::Gpu::new().unwrap();
  let img=image::RgbaImage::from_fn(73,41,|x,y|image::Rgba([((x*17+y*11)%256)as u8,((x*5+y*31)%256)as u8,((x*31+y*7)%256)as u8,((x*13+y*3)%256)as u8]));
  let src=g.upload(&img).unwrap();
  let plain=crate::native_pipeline::NativeGpuPipeline::new(&g.device,73,41).unwrap();
  let fused=crate::native_pipeline::NativeGpuPipeline::new(&g.device,73,41).unwrap();fused.enable_fusion();
  for ids in [vec![5,1,0],vec![6,4,3],vec![7,8,9,11],vec![0,2,5,6],vec![5,5,4,1,11,0,7,8]]{
   let a=g.read(&plain.dispatch(&g.device,&g.ctx,&src,&ids).unwrap()).unwrap();
   let b=g.read(&fused.dispatch(&g.device,&g.ctx,&src,&ids).unwrap()).unwrap();
   let max=a.as_raw().iter().zip(b.as_raw()).map(|(x,y)|x.abs_diff(*y)).max().unwrap_or(0);
   assert!(max<=2,"Fused difference {max} for {ids:?}");
   assert!(a.pixels().zip(b.pixels()).all(|(x,y)|x[3]==y[3]),"Alpha changed");
   assert!(fused.fused_saved()>0,"No actual dispatch removed");
   eprintln!("FUSED_GPU sequence={ids:?} removed={} max_byte_error={max}",fused.fused_saved());
  }
 }
}
