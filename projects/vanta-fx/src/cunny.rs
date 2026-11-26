//! CuNNy Veryfast 4-pass neural 2x upscaler, native D3D11.
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
 Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_SAMPLE_DESC},
}};
pub const SOURCES:[&str;4]=[
 include_str!("../shaders/native/HB_CuNNyVeryFastPass1.hlsl"),
 include_str!("../shaders/native/HB_CuNNyVeryFastPass2.hlsl"),
 include_str!("../shaders/native/HB_CuNNyVeryFastPass3.hlsl"),
 include_str!("../shaders/native/HB_CuNNyVeryFastPass4.hlsl"),
];
pub fn compile_all()->Result<usize,String>{
 for (i,code) in SOURCES.iter().enumerate(){
  crate::shadercheck::compile_native_hlsl(code).map_err(|e|format!("CuNNy native pass{} HLSL: {e}",i+1))?;
 }Ok(4)
}
fn build_array<T,const N:usize>(mut f:impl FnMut(usize)->Result<T,String>)->Result<[T;N],String>{
 (0..N).map(|i|f(i)).collect::<Result<Vec<_>,_>>()?.try_into().map_err(|_|"Neural array length mismatch".into())
}
pub struct Pass{
 size:(u32,u32),
 shaders:[ID3D11ComputeShader;4],
 textures:[ID3D11Texture2D;4],
 views:[ID3D11ShaderResourceView;4],
 outputs:[ID3D11UnorderedAccessView;4],
 point:ID3D11SamplerState,linear:ID3D11SamplerState,
 constants:ID3D11Buffer,
 source_view:crate::reconstruction::ViewCache,
}
impl Pass{
 pub fn new(device:&ID3D11Device,size:(u32,u32))->Result<Self,String>{
  let (w,h)=size;
  if w<1||h<1||w>4096||h>4096||u64::from(w)*u64::from(h)>4_194_304{
   return Err("CuNNy original x2 source must be no more than 4MP".into());
  }
  let shaders:[ID3D11ComputeShader;4]=build_array(|i|->Result<ID3D11ComputeShader,String>{
   let bytecode=crate::shadercheck::compile_native_hlsl(SOURCES[i])
     .map_err(|e|format!("CuNNy pass {} HLSL compile: {e}",i+1))?;
   let mut shader=None;
   unsafe{device.CreateComputeShader(&bytecode,None::<&ID3D11ClassLinkage>,Some(&mut shader))}
     .map_err(|e|format!("CuNNy GPU pass {}: {e}",i+1))?;
   shader.ok_or("CuNNy compiled shader missing".into())
  })?;
  let d=D3D11_TEXTURE2D_DESC{
   Width:w,Height:h,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R8G8B8A8_UNORM,
   SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,
   BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,
   CPUAccessFlags:0,MiscFlags:0,
  };
  let textures:[ID3D11Texture2D;4]=build_array(|_|->Result<ID3D11Texture2D,String>{
   let mut tex=None;unsafe{device.CreateTexture2D(&d,None,Some(&mut tex))}
    .map_err(|e|format!("CuNNy neural feature texture: {e}"))?;
   tex.ok_or("CuNNy neural feature texture missing".into())
  })?;
  let views:[ID3D11ShaderResourceView;4]=build_array(|i|->Result<ID3D11ShaderResourceView,String>{
   let mut view=None;unsafe{device.CreateShaderResourceView(&textures[i],None,Some(&mut view))}
    .map_err(|e|format!("CuNNy feature SRV: {e}"))?;
   view.ok_or("CuNNy feature SRV missing".into())
  })?;
  let outputs:[ID3D11UnorderedAccessView;4]=build_array(|i|->Result<ID3D11UnorderedAccessView,String>{
   let mut view=None;unsafe{device.CreateUnorderedAccessView(&textures[i],None,Some(&mut view))}
    .map_err(|e|format!("CuNNy feature UAV: {e}"))?;
   view.ok_or("CuNNy feature UAV missing".into())
  })?;
  let sampler=|filter|->Result<ID3D11SamplerState,String>{
   let state=D3D11_SAMPLER_DESC{
    Filter:filter,AddressU:D3D11_TEXTURE_ADDRESS_CLAMP,AddressV:D3D11_TEXTURE_ADDRESS_CLAMP,
    AddressW:D3D11_TEXTURE_ADDRESS_CLAMP,MipLODBias:0.,MaxAnisotropy:1,
    ComparisonFunc:D3D11_COMPARISON_NEVER,BorderColor:[0.;4],MinLOD:0.,MaxLOD:16.,
   };
   let mut handle=None;unsafe{device.CreateSamplerState(&state,Some(&mut handle))}
    .map_err(|e|format!("CuNNy native sampler: {e}"))?;
   handle.ok_or("CuNNy native sampler missing".into())
  };
  let mut constants=None;
  unsafe{device.CreateBuffer(&D3D11_BUFFER_DESC{
   ByteWidth:16,Usage:D3D11_USAGE_DEFAULT,
   BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0
  },None,Some(&mut constants))}
    .map_err(|e|format!("CuNNy native input/output size uniform: {e}"))?;
  Ok(Self{
   size,shaders,textures,views,outputs,
   point:sampler(D3D11_FILTER_MIN_MAG_MIP_POINT)?,
   linear:sampler(D3D11_FILTER_MIN_MAG_MIP_LINEAR)?,
   constants:constants.ok_or("CuNNy size buffer missing")?,
   source_view:Default::default(),
  })
 }
 pub fn compatible(&self,size:(u32,u32))->bool{self.size==size}
 pub fn render(&mut self,device:&ID3D11Device,ctx:&ID3D11DeviceContext,
    source:&ID3D11Texture2D,target:&ID3D11UnorderedAccessView)->Result<(),String>{
  let mut size=D3D11_TEXTURE2D_DESC::default();unsafe{source.GetDesc(&mut size);}
  if !self.compatible((size.Width,size.Height))||size.Format!=DXGI_FORMAT_R8G8B8A8_UNORM{
   return Err("CuNNy source dimensions/format changed; rebuild scaler first".into());
  }
  let input=self.source_view.get(device,source)?;
  let input_copy=input.clone();
  let dimensions=[size.Width,size.Height,size.Width*2,size.Height*2];
  let data:ID3D11Resource=self.constants.cast().map_err(|e|e.to_string())?;
  unsafe{
   ctx.UpdateSubresource(&data,0,None,dimensions.as_ptr().cast(),0,0);
   ctx.CSSetConstantBuffers(0,Some(&[Some(self.constants.clone())]));
   ctx.CSSetSamplers(0,Some(&[Some(self.point.clone()),Some(self.linear.clone())]));
  }
  let stages=[
   ([Some(input.clone()),None],[Some(self.outputs[0].clone()),Some(self.outputs[1].clone())],2usize,2usize),
   ([Some(self.views[0].clone()),Some(self.views[1].clone())],[Some(self.outputs[2].clone()),Some(self.outputs[3].clone())],2,2),
   ([Some(self.views[2].clone()),Some(self.views[3].clone())],[Some(self.outputs[0].clone()),None],2,1),
   ([Some(input_copy),Some(self.views[0].clone())],[Some(target.clone()),None],2,1),
  ];
  for (i,(srvs,uavs,n_in,n_out)) in stages.into_iter().enumerate(){
   let groups=(if i==3{self.size.0*2}else{self.size.0},
               if i==3{self.size.1*2}else{self.size.1});
   let block=if i==3{16}else{8};
   unsafe{
    ctx.CSSetShader(&self.shaders[i],None);
    ctx.CSSetShaderResources(0,Some(&srvs[..n_in]));
    ctx.CSSetUnorderedAccessViews(0,n_out as u32,Some(uavs.as_ptr()),None);
    ctx.Dispatch(groups.0.div_ceil(block),groups.1.div_ceil(block),1);
    // Detach before reusing T0 as read/write across neural passes.
    ctx.CSSetShaderResources(0,Some(&[None,None]));
    let clears:[Option<ID3D11UnorderedAccessView>;2]=[None,None];
    ctx.CSSetUnorderedAccessViews(0,2,Some(clears.as_ptr()),None);
   }
  }
  unsafe{
   ctx.CSSetConstantBuffers(0,Some(&[None]));
   ctx.CSSetSamplers(0,Some(&[None,None]));
   ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);
  }
  Ok(())
 }
}
#[cfg(test)]mod tests{
 #[test]fn original_four_neural_passes_compile_native_on_amd(){
  assert_eq!(super::compile_all().unwrap(),4);
 }
 #[test]fn real_cunny_neural_four_passes_transform_synthetic_pixels_on_radeon(){
  use windows::Win32::Graphics::Direct3D11::{
   D3D11_TEXTURE2D_DESC,D3D11_BIND_SHADER_RESOURCE,D3D11_BIND_UNORDERED_ACCESS,
   D3D11_USAGE_DEFAULT,
  };
  use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_SAMPLE_DESC};
  let gpu=crate::photo_lab::Gpu::new().unwrap();
  const W:u32=31;const H:u32=25;
  let original=image::RgbaImage::from_fn(W,H,|x,y|{
   image::Rgba([((x*11+y*3)%194+30)as u8,((x*3+y*7)%196+23)as u8,
    if (x/5+y/5)%2==0{32}else{224},255])
  });
  let input=gpu.upload(&original).unwrap();
  let desc=D3D11_TEXTURE2D_DESC{
    Width:W*2,Height:H*2,MipLevels:1,ArraySize:1,
    Format:DXGI_FORMAT_R8G8B8A8_UNORM,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
    Usage:D3D11_USAGE_DEFAULT,
    BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,
    CPUAccessFlags:0,MiscFlags:0
  };
  let mut out=None;unsafe{gpu.device.CreateTexture2D(&desc,None,Some(&mut out))}.unwrap();
  let out=out.unwrap();
  let mut uav=None;unsafe{gpu.device.CreateUnorderedAccessView(&out,None,Some(&mut uav))}.unwrap();
  let mut pass=super::Pass::new(&gpu.device,(W,H)).unwrap();
  pass.render(&gpu.device,&gpu.ctx,&input,&uav.unwrap()).unwrap();
  let image=gpu.read(&out).unwrap();
  assert_eq!(image.dimensions(),(W*2,H*2));
  assert!(image.pixels().all(|p|p[3]==255));
  let num_nonblack=image.pixels().filter(|p|p[0]>10||p[1]>10||p[2]>10).count();
  assert!(num_nonblack>W as usize*H as usize,"CuNNy multiple neural passes generated almost nothing");
  let linear=image::imageops::resize(&original,W*2,H*2,image::imageops::FilterType::Triangle);
  let differences=image.pixels().zip(linear.pixels()).filter(|(x,y)|x!=y).count();
  assert!(differences>W as usize,"CuNNy must have real neural 4-pass output different from bilinear");
 }
}
