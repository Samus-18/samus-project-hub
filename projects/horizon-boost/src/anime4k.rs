//! Native MIT Anime4K CNN x2 S (FP32 arithmetic, half float intermediate maps).
use windows::Win32::Graphics::{Direct3D11::*,Dxgi::Common::{DXGI_FORMAT_R16G16B16A16_FLOAT,DXGI_SAMPLE_DESC}};
const SOURCES:[&str;5]=[include_str!("../shaders/native/HB_Anime4K_1.hlsl"),include_str!("../shaders/native/HB_Anime4K_2.hlsl"),include_str!("../shaders/native/HB_Anime4K_3.hlsl"),include_str!("../shaders/native/HB_Anime4K_4.hlsl"),include_str!("../shaders/native/HB_Anime4K_Shuffle.hlsl")];
struct Surface{srv:ID3D11ShaderResourceView,uav:ID3D11UnorderedAccessView}
impl Surface{
 fn new(d:&ID3D11Device,(w,h):(u32,u32))->Result<Self,String>{
  let desc=D3D11_TEXTURE2D_DESC{Width:w,Height:h,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R16G16B16A16_FLOAT,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0)as u32,CPUAccessFlags:0,MiscFlags:0};
  let mut tex=None;unsafe{d.CreateTexture2D(&desc,None,Some(&mut tex))}.map_err(|e|e.to_string())?;let tex=tex.ok_or("CNN surface missing")?;
  let mut srv=None;let mut uav=None;unsafe{d.CreateShaderResourceView(&tex,None,Some(&mut srv))}.map_err(|e|e.to_string())?;unsafe{d.CreateUnorderedAccessView(&tex,None,Some(&mut uav))}.map_err(|e|e.to_string())?;
  Ok(Self{srv:srv.ok_or("CNN SRV missing")?,uav:uav.ok_or("CNN UAV missing")?})
 }
}
pub struct Pass{size:(u32,u32),kernels:Vec<ID3D11ComputeShader>,a:Surface,b:Surface}
impl Pass{
 pub fn new(d:&ID3D11Device,size:(u32,u32))->Result<Self,String>{
  if size.0==0||size.1==0||u64::from(size.0)*u64::from(size.1)>8_294_400{return Err("CNN input exceeds working memory budget".into());}
  let mut kernels=Vec::with_capacity(5);for s in SOURCES{let bytes=crate::shadercheck::compile_native_hlsl(s)?;let mut sh=None;unsafe{d.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut sh))}.map_err(|e|e.to_string())?;kernels.push(sh.ok_or("CNN shader missing")?);}
  Ok(Self{size,kernels,a:Surface::new(d,size)?,b:Surface::new(d,size)?})
 }
 pub fn compatible(&self,size:(u32,u32))->bool{self.size==size}
 pub fn render(&self,ctx:&ID3D11DeviceContext,src:&ID3D11ShaderResourceView,dst:&ID3D11UnorderedAccessView){
  let (w,h)=self.size;
  let dispatch=|i:usize,inputs:&[Option<ID3D11ShaderResourceView>],out:&ID3D11UnorderedAccessView,w:u32,h:u32|unsafe{
   ctx.CSSetShader(&self.kernels[i],None);ctx.CSSetShaderResources(0,Some(inputs));
   let u=[Some(out.clone())];ctx.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);ctx.Dispatch(w.div_ceil(8),h.div_ceil(8),1);
   ctx.CSSetShaderResources(0,Some(&[None,None]));let u:[Option<ID3D11UnorderedAccessView>;1]=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
  };
  dispatch(0,&[Some(src.clone())],&self.a.uav,w,h);dispatch(1,&[Some(self.a.srv.clone())],&self.b.uav,w,h);dispatch(2,&[Some(self.b.srv.clone())],&self.a.uav,w,h);dispatch(3,&[Some(self.a.srv.clone())],&self.b.uav,w,h);
  dispatch(4,&[Some(src.clone()),Some(self.b.srv.clone())],dst,w*2,h*2);
 }
}
#[cfg(test)]mod tests{
 #[test]fn anime4k_real_gpu_odd_source_alpha_and_edges(){
 let g=crate::photo_lab::Gpu::new().unwrap();
 let im=image::RgbaImage::from_fn(19,13,|x,y|{let v=if x>y+3{205}else{35};image::Rgba([v,v,v,173])});
 let tex=g.upload(&im).unwrap();let mut s=crate::spatial::NativeSpatialScaler::new(&g.device,38,26).unwrap();
 let base=g.read(&s.render(&g.device,&g.ctx,&tex,1).unwrap()).unwrap();
 let a=g.read(&s.render(&g.device,&g.ctx,&tex,14).unwrap()).unwrap();
 assert_eq!(a.dimensions(),(38,26));assert_ne!(base,a);assert!(a.pixels().all(|p|p[3]==173));
 assert_eq!(a,g.read(&s.render(&g.device,&g.ctx,&tex,14).unwrap()).unwrap());
 assert!(a.get_pixel(0,25)[0]<60&&a.get_pixel(37,0)[0]>170);
 eprintln!("ANIME4K_GPU=PASS ODD_INPUT=19x13 EXACT_2X=38x26 ALPHA_EXACT=true DISTINCT=true");
 }
 #[test]fn anime4k_requires_exact_two_x(){use crate::spatial::effective_mode;assert_eq!(effective_mode((640,360),(1280,720),14),14);assert_eq!(effective_mode((640,360),(960,540),14),1);assert_eq!(effective_mode((640,360),(640,360),14),0);}
}
