use windows::core::Interface;
use windows::Win32::Graphics::{
 Direct3D11::{ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11ShaderResourceView,
 ID3D11UnorderedAccessView,ID3D11ComputeShader,ID3D11ClassLinkage,D3D11_TEXTURE2D_DESC,
 D3D11_USAGE_DEFAULT,D3D11_BIND_SHADER_RESOURCE,D3D11_BIND_UNORDERED_ACCESS,ID3D11Resource,ID3D11Buffer,D3D11_BUFFER_DESC,D3D11_BIND_CONSTANT_BUFFER},
 Dxgi::Common::{DXGI_FORMAT_R32_FLOAT,DXGI_SAMPLE_DESC},
};
pub struct Histogram{_image:ID3D11Texture2D,srv:ID3D11ShaderResourceView,uav:ID3D11UnorderedAccessView,shader:ID3D11ComputeShader,
 device:ID3D11Device,history:std::cell::OnceCell<Smoother>}
struct Smoother{old:ID3D11Texture2D,output:ID3D11Texture2D,old_srv:ID3D11ShaderResourceView,
 output_srv:ID3D11ShaderResourceView,output_uav:ID3D11UnorderedAccessView,shader:ID3D11ComputeShader,constants:ID3D11Buffer,last:std::cell::Cell<Option<std::time::Instant>>}
impl Smoother{
 fn new(d:&ID3D11Device)->Result<Self,String>{
  let desc=D3D11_TEXTURE2D_DESC{Width:1,Height:1,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R32_FLOAT,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,CPUAccessFlags:0,MiscFlags:0};
  let make=||->Result<(ID3D11Texture2D,ID3D11ShaderResourceView),String>{
   let mut tex=None;unsafe{d.CreateTexture2D(&desc,None,Some(&mut tex))}.map_err(|e|e.to_string())?;
   let tex=tex.ok_or("Exposure history texture missing")?;let mut srv=None;unsafe{d.CreateShaderResourceView(&tex,None,Some(&mut srv))}.map_err(|e|e.to_string())?;
   Ok((tex,srv.ok_or("Exposure history SRV missing")?))
  };
  let(old,old_srv)=make()?;let(output,output_srv)=make()?;
  let mut output_uav=None;unsafe{d.CreateUnorderedAccessView(&output,None,Some(&mut output_uav))}.map_err(|e|e.to_string())?;
  let bytes=crate::shadercheck::compile_native_hlsl(include_str!("../shaders/native/HB_ExposureSmooth.hlsl"))?;
  let mut shader=None;unsafe{d.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|e.to_string())?;
  let mut constants=None;unsafe{d.CreateBuffer(&D3D11_BUFFER_DESC{ByteWidth:16,Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0},None,Some(&mut constants))}.map_err(|e|e.to_string())?;
  Ok(Self{old,output,old_srv,output_srv,output_uav:output_uav.ok_or("Exposure history UAV missing")?,shader:shader.ok_or("Exposure smoothing shader missing")?,constants:constants.ok_or("Exposure smoothing cbuffer missing")?,last:std::cell::Cell::new(None)})
 }
 fn run(&self,ctx:&ID3D11DeviceContext,input:&ID3D11ShaderResourceView,options:&crate::exposure_options::Options)->Result<(),String>{
  let now=std::time::Instant::now();let dt=self.last.replace(Some(now)).map(|t|now.saturating_duration_since(t).as_secs_f32());
  let reset=dt.is_none_or(|t|t>0.5);
  let alpha=1.-(-dt.unwrap_or(0.).clamp(0.,0.1)/options.seconds.clamp(0.05,3.)).exp();
  let p=[alpha,options.reset_delta.clamp(0.1,0.75),if reset{1.}else{0.},0.];
  let constant:ID3D11Resource=self.constants.cast().map_err(|e|e.to_string())?;
  let old:ID3D11Resource=self.old.cast().map_err(|e|e.to_string())?;let output:ID3D11Resource=self.output.cast().map_err(|e|e.to_string())?;
  unsafe{
   ctx.UpdateSubresource(&constant,0,None,p.as_ptr().cast(),0,0);ctx.CSSetConstantBuffers(0,Some(&[Some(self.constants.clone())]));
   ctx.CSSetShader(&self.shader,None);ctx.CSSetShaderResources(0,Some(&[Some(input.clone()),Some(self.old_srv.clone())]));
   let u=[Some(self.output_uav.clone())];ctx.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);ctx.Dispatch(1,1,1);
   ctx.CSSetShaderResources(0,Some(&[None,None]));ctx.CSSetConstantBuffers(0,Some(&[None]));
   let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
   ctx.CopyResource(&old,&output);
  }Ok(())
 }
}
impl Histogram{
 pub fn new(device:&ID3D11Device)->Result<Self,String>{
  let bytes=crate::shadercheck::compile_native_hlsl(include_str!("../shaders/native/HB_HistogramReduce.hlsl"))?;
  let mut shader=None;
  unsafe{device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|e.to_string())?;
  let desc=D3D11_TEXTURE2D_DESC{Width:1,Height:1,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R32_FLOAT,
    SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,
    BindFlags:(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32,CPUAccessFlags:0,MiscFlags:0};
  let mut image=None;unsafe{device.CreateTexture2D(&desc,None,Some(&mut image))}.map_err(|e|e.to_string())?;
  let image=image.ok_or("Histogram exposure texture missing")?;
  let(mut srv,mut uav)=(None,None);
  unsafe{device.CreateShaderResourceView(&image,None,Some(&mut srv))}.map_err(|e|e.to_string())?;
  unsafe{device.CreateUnorderedAccessView(&image,None,Some(&mut uav))}.map_err(|e|e.to_string())?;
  Ok(Self{_image:image,srv:srv.ok_or("Histogram SRV missing")?,uav:uav.ok_or("Histogram UAV missing")?,shader:shader.ok_or("Histogram shader missing")?,device:device.clone(),history:std::cell::OnceCell::new()})
 }
 pub fn measure(&self,ctx:&ID3D11DeviceContext,input:&ID3D11ShaderResourceView,options:&crate::exposure_options::Options)->Result<(),String>{
  unsafe{
   ctx.CSSetShader(&self.shader,None);
   ctx.CSSetShaderResources(0,Some(&[Some(input.clone())]));
   let u=[Some(self.uav.clone())];ctx.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);
   ctx.Dispatch(1,1,1);
   ctx.CSSetShaderResources(0,Some(&[None]));
   let empty=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(empty.as_ptr()),None);
  }
  let result=if options.temporal{
   if self.history.get().is_none(){let _=self.history.set(Smoother::new(&self.device)?);}
   let h=self.history.get().ok_or("Exposure history missing")?;h.run(ctx,&self.srv,options)?;h.output_srv.clone()
  }else{if let Some(h)=self.history.get(){h.last.set(None);}self.srv.clone()};
  unsafe{ctx.CSSetShaderResources(1,Some(&[Some(result)]));}
  Ok(())
 }
}
#[cfg(test)]mod tests{
 #[test]fn actual_histogram_reduce_compiles(){
  assert!(crate::shadercheck::compile_native_hlsl(include_str!("../shaders/native/HB_HistogramReduce.hlsl")).is_ok());
 }
}
