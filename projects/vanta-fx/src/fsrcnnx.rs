//! True native GPU FSRCNNX_x2_8-0-4-1 six-pass neural scaler.
//! Original GPLv3 weights and convolutions from Magpie FSRCNNX source.
//! D3D11 FP16 feature maps, no Magpie EXE, CPU image inference or fake sharpen.
use windows::{core::Interface,Win32::Graphics::{
 Direct3D11::{
  ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11ShaderResourceView,
  ID3D11UnorderedAccessView,ID3D11SamplerState,ID3D11Buffer,ID3D11ComputeShader,
  ID3D11ClassLinkage,ID3D11Resource,D3D11_TEXTURE2D_DESC,D3D11_BUFFER_DESC,
  D3D11_SAMPLER_DESC,D3D11_USAGE_DEFAULT,D3D11_BIND_SHADER_RESOURCE,
  D3D11_BIND_UNORDERED_ACCESS,D3D11_BIND_CONSTANT_BUFFER,
  D3D11_FILTER_MIN_MAG_MIP_POINT,D3D11_FILTER_MIN_MAG_MIP_LINEAR,
  D3D11_TEXTURE_ADDRESS_CLAMP,D3D11_COMPARISON_NEVER,
 },
 Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_FORMAT_R16G16B16A16_FLOAT,DXGI_SAMPLE_DESC},
}};
pub const SHADERS:[&str;6]=[
 include_str!("../shaders/native/HB_FSRCNNXPass1.hlsl"),
 include_str!("../shaders/native/HB_FSRCNNXPass2.hlsl"),
 include_str!("../shaders/native/HB_FSRCNNXPass3.hlsl"),
 include_str!("../shaders/native/HB_FSRCNNXPass4.hlsl"),
 include_str!("../shaders/native/HB_FSRCNNXPass5.hlsl"),
 include_str!("../shaders/native/HB_FSRCNNXPass6.hlsl"),
];
fn make_array<T,const N:usize>(f:impl FnMut(usize)->Result<T,String>)->Result<[T;N],String>{
 (0..N).map(f).collect::<Result<Vec<_>,_>>()?.try_into().map_err(|_|"FSRCNNX array mismatch".into())
}
pub fn compile_all()->Result<usize,String>{
 for (i,code) in SHADERS.iter().enumerate(){
  crate::shadercheck::compile_native_hlsl(code)
    .map_err(|e|format!("FSRCNNX pass {} compile: {e}",i+1))?;
 }Ok(6)
}
pub struct Pass{
 size:(u32,u32),
 shaders:[ID3D11ComputeShader;6],
 surfaces:[ID3D11Texture2D;6],
 views:[ID3D11ShaderResourceView;6],
 outputs:[ID3D11UnorderedAccessView;6],
 point:ID3D11SamplerState,linear:ID3D11SamplerState,
 constants:ID3D11Buffer,
 cache:crate::reconstruction::ViewCache,
}
impl Pass{
 pub fn new(device:&ID3D11Device,size:(u32,u32))->Result<Self,String>{
  let (w,h)=size;
  if w<1||h<1||w>4096||h>4096||u64::from(w)*u64::from(h)>2_073_600{
   return Err("Original FSRCNNX x2 limited to 2,073,600 input pixels to protect RX580 VRAM".into());
  }
  let shaders:[ID3D11ComputeShader;6]=make_array(|i|{
   let bytes=crate::shadercheck::compile_native_hlsl(SHADERS[i])
     .map_err(|e|format!("FSRCNNX neural pass {} compile: {e}",i+1))?;
   let mut shader=None;
   unsafe{device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}
     .map_err(|e|format!("FSRCNNX GPU compute pass {}: {e}",i+1))?;
   shader.ok_or("FSRCNNX compute shader missing".into())
  })?;
  let desc=D3D11_TEXTURE2D_DESC{
   Width:w,Height:h,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R16G16B16A16_FLOAT,
   SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,
   BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,
   CPUAccessFlags:0,MiscFlags:0,
  };
  let surfaces:[ID3D11Texture2D;6]=make_array(|_|{
   let mut tex=None;
   unsafe{device.CreateTexture2D(&desc,None,Some(&mut tex))}
     .map_err(|e|format!("FSRCNNX FP16 neural feature map allocation: {e}"))?;
   tex.ok_or("FSRCNNX FP16 feature map missing".into())
  })?;
  let views:[ID3D11ShaderResourceView;6]=make_array(|i|{
   let mut view=None;
   unsafe{device.CreateShaderResourceView(&surfaces[i],None,Some(&mut view))}
     .map_err(|e|format!("FSRCNNX feature map SRV: {e}"))?;
   view.ok_or("FSRCNNX input view missing".into())
  })?;
  let outputs:[ID3D11UnorderedAccessView;6]=make_array(|i|{
   let mut view=None;
   unsafe{device.CreateUnorderedAccessView(&surfaces[i],None,Some(&mut view))}
     .map_err(|e|format!("FSRCNNX feature map UAV: {e}"))?;
   view.ok_or("FSRCNNX feature UAV missing".into())
  })?;
  let sampler=|filter|->Result<ID3D11SamplerState,String>{
   let desc=D3D11_SAMPLER_DESC{
    Filter:filter,AddressU:D3D11_TEXTURE_ADDRESS_CLAMP,AddressV:D3D11_TEXTURE_ADDRESS_CLAMP,
    AddressW:D3D11_TEXTURE_ADDRESS_CLAMP,MipLODBias:0.,MaxAnisotropy:1,
    ComparisonFunc:D3D11_COMPARISON_NEVER,BorderColor:[0.;4],MinLOD:0.,MaxLOD:16.
   };
   let mut s=None;unsafe{device.CreateSamplerState(&desc,Some(&mut s))}
     .map_err(|e|format!("FSRCNNX sampler: {e}"))?;
   s.ok_or("FSRCNNX sampler missing".into())
  };
  let mut constants=None;
  unsafe{device.CreateBuffer(&D3D11_BUFFER_DESC{
   ByteWidth:16,Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,
   CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0,
  },None,Some(&mut constants))}
    .map_err(|e|format!("FSRCNNX 2x dimensions buffer: {e}"))?;
  Ok(Self{
   size,shaders,surfaces,views,outputs,
   point:sampler(D3D11_FILTER_MIN_MAG_MIP_POINT)?,
   linear:sampler(D3D11_FILTER_MIN_MAG_MIP_LINEAR)?,
   constants:constants.ok_or("FSRCNNX dims buffer missing")?,cache:Default::default()
  })
 }
 pub fn compatible(&self,d:(u32,u32))->bool{self.size==d}
 pub fn render(&mut self,device:&ID3D11Device,ctx:&ID3D11DeviceContext,
    source:&ID3D11Texture2D,target:&ID3D11UnorderedAccessView)->Result<(),String>{
  let mut d=D3D11_TEXTURE2D_DESC::default();unsafe{source.GetDesc(&mut d);}
  if (d.Width,d.Height)!=self.size||d.Format!=DXGI_FORMAT_R8G8B8A8_UNORM{
   return Err("FSRCNNX expects RGBA8 source at matching x2 dimensions".into());
  }
  let source_srv=self.cache.get(device,source)?;
  let sizes=[d.Width,d.Height,d.Width*2,d.Height*2];
  let constants:ID3D11Resource=self.constants.cast().map_err(|e|e.to_string())?;
  unsafe{
   ctx.UpdateSubresource(&constants,0,None,sizes.as_ptr().cast(),0,0);
   ctx.CSSetConstantBuffers(0,Some(&[Some(self.constants.clone())]));
   ctx.CSSetSamplers(0,Some(&[Some(self.point.clone()),Some(self.linear.clone())]));
  }
  let v=&self.views;
  let u=&self.outputs;
  let passes=[
   ([Some(source_srv.clone()),None,None,None],[Some(u[0].clone()),Some(u[1].clone())],1,2),
   ([Some(v[0].clone()),Some(v[1].clone()),None,None],[Some(u[2].clone()),Some(u[3].clone())],2,2),
   ([Some(v[2].clone()),Some(v[3].clone()),None,None],[Some(u[4].clone()),Some(u[5].clone())],2,2),
   ([Some(v[4].clone()),Some(v[5].clone()),None,None],[Some(u[2].clone()),Some(u[3].clone())],2,2),
   ([Some(v[2].clone()),Some(v[3].clone()),Some(v[0].clone()),Some(v[1].clone())],[Some(u[4].clone()),Some(u[5].clone())],4,2),
   ([Some(v[4].clone()),Some(v[5].clone()),Some(source_srv.clone()),None],[Some(target.clone()),None],3,1),
  ];
  for (i,(inputs,outputs,nin,nout)) in passes.into_iter().enumerate(){
   let(w,h)=(if i==5{d.Width*2}else{d.Width},if i==5{d.Height*2}else{d.Height});
   let bw=if i==0{32}else if i==5{16}else{8};
   let bh=if i==0{24}else if i==5{16}else{8};
   unsafe{
    ctx.CSSetShader(&self.shaders[i],None);
    ctx.CSSetShaderResources(0,Some(&inputs[..nin]));
    ctx.CSSetUnorderedAccessViews(0,nout,Some(outputs.as_ptr()),None);
    ctx.Dispatch(w.div_ceil(bw),h.div_ceil(bh),1);
    ctx.CSSetShaderResources(0,Some(&[None,None,None,None]));
    let cleared:[Option<ID3D11UnorderedAccessView>;2]=[None,None];
    ctx.CSSetUnorderedAccessViews(0,2,Some(cleared.as_ptr()),None);
   }
  }
  unsafe{
   ctx.CSSetConstantBuffers(0,Some(&[None]));
   ctx.CSSetSamplers(0,Some(&[None,None]));
   ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
  }Ok(())
 }
}
#[cfg(test)]mod tests{
 #[test]fn compiles_all_real_original_fsrcnnx_neural_passes(){
  assert_eq!(super::compile_all().unwrap(),6);
 }
 #[test]fn rx580_gpu_neural_six_pass_x2_output_real_data_not_bilinear(){
  use windows::Win32::Graphics::Direct3D11::{
   D3D11_TEXTURE2D_DESC,D3D11_BIND_SHADER_RESOURCE,D3D11_BIND_UNORDERED_ACCESS,D3D11_USAGE_DEFAULT,
  };
  use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_SAMPLE_DESC};
  let gpu=crate::photo_lab::Gpu::new().unwrap();
  const W:u32=41;const H:u32=29;
  let original=image::RgbaImage::from_fn(W,H,|x,y|{
   let high=(x/6+y/4)%2==0;
   image::Rgba([if high{213}else{31},((x*3+y*11)%190+26)as u8,if high{180}else{27},255])
  });
  let source=gpu.upload(&original).unwrap();
  let mut output=None;
  unsafe{gpu.device.CreateTexture2D(&D3D11_TEXTURE2D_DESC{
    Width:W*2,Height:H*2,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R8G8B8A8_UNORM,
    SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,
    BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,
    CPUAccessFlags:0,MiscFlags:0,
  },None,Some(&mut output))}.unwrap();
  let output=output.unwrap();
  let mut target=None;unsafe{gpu.device.CreateUnorderedAccessView(&output,None,Some(&mut target))}.unwrap();
  let mut pass=super::Pass::new(&gpu.device,(W,H)).unwrap();
  pass.render(&gpu.device,&gpu.ctx,&source,&target.unwrap()).unwrap();
  let after=gpu.read(&output).unwrap();
  assert_eq!(after.dimensions(),(W*2,H*2));
  assert!(after.pixels().all(|p|p[3]==255));
  let changed=after.pixels().filter(|p|p[0]>10||p[1]>10||p[2]>10).count();
  assert!(changed>W as usize*H as usize,"FSRCNNX six-pass neural result nearly blank: {changed}");
  let bilinear=image::imageops::resize(&original,W*2,H*2,image::imageops::FilterType::Triangle);
  let diff=after.pixels().zip(bilinear.pixels()).filter(|(a,b)|a!=b).count();
  assert!(diff>W as usize*H as usize,"FSRCNNX output seems a bilinear stub");
 }
}
