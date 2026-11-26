use serde::{Serialize,Deserialize};
use std::{path::{Path,PathBuf},fs,io::Read};
use crate::cube_lut::Cube;
use windows::{core::Interface,Win32::Graphics::{Direct3D11::{
 ID3D11Device,ID3D11DeviceContext,ID3D11ComputeShader,ID3D11ClassLinkage,ID3D11ShaderResourceView,
 ID3D11UnorderedAccessView,ID3D11Buffer,ID3D11Resource,D3D11_TEXTURE2D_DESC,D3D11_SUBRESOURCE_DATA,
 D3D11_BUFFER_DESC,D3D11_USAGE_DEFAULT,D3D11_BIND_SHADER_RESOURCE,D3D11_BIND_CONSTANT_BUFFER},
 Dxgi::Common::{DXGI_FORMAT_R32G32B32A32_FLOAT,DXGI_SAMPLE_DESC}}};
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]pub struct Options{pub name:Option<String>,pub strength:f32}
impl Default for Options{fn default()->Self{Self{name:None,strength:1.}}}
fn valid_id(s:&str)->bool{
 matches!(s,"identity"|"inverse")||
 (s.starts_with("lut-")&&s.ends_with(".cube")&&s.len()>9&&s.len()<100&&s[4..s.len()-5].bytes().all(|b|b.is_ascii_digit()||b==b'-'))
}
impl Options{pub fn clamp(&mut self){if self.name.as_ref().is_some_and(|s|!valid_id(s)){self.name=None;}self.strength=if self.strength.is_finite(){self.strength.clamp(0.,1.)}else{1.};}}
fn folder()->PathBuf{crate::settings::folder().join("live-luts")}
fn read_cube(path:&Path)->Result<Cube,String>{
 let mut s=String::new();fs::File::open(path).map_err(|e|e.to_string())?.take(4*1024*1024+1).read_to_string(&mut s).map_err(|e|e.to_string())?;
 Cube::parse(&s)
}
pub fn load(id:&str)->Result<Cube,String>{
 if !valid_id(id){return Err("Invalid local LUT ID".into());}
 if id=="identity"{return Ok(crate::cube_lut::fixture(false));}
 if id=="inverse"{return Ok(crate::cube_lut::fixture(true));}
 read_cube(&folder().join(id)).map_err(|e|format!("Local LUT unavailable: {e}"))
}
fn from_strip(img:&image::RgbaImage)->Result<Cube,String>{
 let(w,n)=img.dimensions();
 if !(2..=33).contains(&n)||w!=n*n{return Err("Use horizontal PNG strip N*N by N, N=2..33, red-fastest tiles".into());}
 let mut values=Vec::with_capacity(n.pow(3) as usize);
 for b in 0..n{for g in 0..n{for r in 0..n{
  let p=img.get_pixel(b*n+r,g);values.push([p[0] as f32/255.,p[1] as f32/255.,p[2] as f32/255.]);
 }}}
 Ok(Cube{size:n,values})
}
fn read_strip(path:&Path)->Result<Cube,String>{
 if fs::metadata(path).map_err(|e|e.to_string())?.len()>4*1024*1024{return Err("PNG LUT exceeds 4 MiB".into());}
 let mut reader=image::ImageReader::open(path).map_err(|e|e.to_string())?.with_guessed_format().map_err(|e|e.to_string())?;
 if reader.format()!=Some(image::ImageFormat::Png){return Err("LUT strip must be PNG".into());}
 let mut limits=image::Limits::default();limits.max_image_width=Some(1089);limits.max_image_height=Some(33);limits.max_alloc=Some(4*1024*1024);reader.limits(limits);
 from_strip(&reader.decode().map_err(|e|e.to_string())?.to_rgba8())
}
pub fn import(path:&Path)->Result<String,String>{
 let cube=match path.extension().and_then(|v|v.to_str()).map(str::to_ascii_lowercase).as_deref(){
 Some("cube")=>read_cube(path)?,Some("png")=>read_strip(path)?,_=>return Err("Choose .cube or PNG LUT strip".into())};
 let mut text=format!("LUT_3D_SIZE {}\n",cube.size);
 for v in cube.values{text.push_str(&format!("{:.8} {:.8} {:.8}\n",v[0],v[1],v[2]));}
 Cube::parse(&text)?;
 let dir=folder();fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
 let id=format!("lut-{}.cube",crate::media_tools::stamp());
 crate::settings::atomic_json_replace(&dir.join(&id),text.as_bytes())?;Ok(id)
}
pub fn list()->Vec<String>{
 let mut v=vec!["identity".into(),"inverse".into()];
 if let Ok(entries)=fs::read_dir(folder()){for e in entries.flatten().take(300){if let Some(n)=e.file_name().to_str(){if valid_id(n)&&e.file_type().is_ok_and(|m|m.is_file()){v.push(n.into());}}}}
 v.sort();v.truncate(100);v
}
pub struct Pass{shader:ID3D11ComputeShader,lut:ID3D11ShaderResourceView,constants:ID3D11Buffer}
impl Pass{
 pub fn new(device:&ID3D11Device,cube:&Cube)->Result<Self,String>{
  let n=cube.size;let data=cube.packed();
  let d=D3D11_TEXTURE2D_DESC{Width:n*n,Height:n,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R32G32B32A32_FLOAT,SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},
   Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,CPUAccessFlags:0,MiscFlags:0};
  let init=D3D11_SUBRESOURCE_DATA{pSysMem:data.as_ptr().cast(),SysMemPitch:n*n*16,SysMemSlicePitch:n*n*n*16};
  let mut texture=None;unsafe{device.CreateTexture2D(&d,Some(&init),Some(&mut texture))}.map_err(|e|e.to_string())?;
  let mut lut=None;unsafe{device.CreateShaderResourceView(&texture.ok_or("LUT texture missing")?,None,Some(&mut lut))}.map_err(|e|e.to_string())?;
  let bytes=crate::shadercheck::compile_native_hlsl(&format!("#define LIVE_LUT 1\n{}",include_str!("../shaders/native/HB_CubePreview.hlsl")))?;
  let mut shader=None;unsafe{device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|e.to_string())?;
  let mut constants=None;unsafe{device.CreateBuffer(&D3D11_BUFFER_DESC{ByteWidth:16,Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_CONSTANT_BUFFER.0 as u32,CPUAccessFlags:0,MiscFlags:0,StructureByteStride:0},None,Some(&mut constants))}.map_err(|e|e.to_string())?;
  Ok(Self{shader:shader.ok_or("LUT shader missing")?,lut:lut.ok_or("LUT SRV missing")?,constants:constants.ok_or("LUT cbuffer missing")?})
 }
 pub fn run(&self,ctx:&ID3D11DeviceContext,input:&ID3D11ShaderResourceView,output:&ID3D11UnorderedAccessView,w:u32,h:u32,strength:f32)->Result<(),String>{
  let v=[strength,0.,0.,0.];let resource:ID3D11Resource=self.constants.cast().map_err(|e|e.to_string())?;
  unsafe{
   ctx.UpdateSubresource(&resource,0,None,v.as_ptr().cast(),0,0);ctx.CSSetConstantBuffers(0,Some(&[Some(self.constants.clone())]));
   ctx.CSSetShader(&self.shader,None);ctx.CSSetShaderResources(0,Some(&[Some(input.clone()),Some(self.lut.clone())]));
   let u=[Some(output.clone())];ctx.CSSetUnorderedAccessViews(0,1,Some(u.as_ptr()),None);ctx.Dispatch(w.div_ceil(8),h.div_ceil(8),1);
   ctx.CSSetShaderResources(0,Some(&[None,None]));ctx.CSSetConstantBuffers(0,Some(&[None]));
   let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);
  }Ok(())
 }
}
#[cfg(test)]mod tests{
 #[test]fn strip_order_and_id_validation(){
  let im=image::RgbaImage::from_fn(4,2,|x,y|image::Rgba([((x%2)*255) as u8,(y*255) as u8,((x/2)*255) as u8,255]));
  let c=super::from_strip(&im).unwrap();assert_eq!(c.values,crate::cube_lut::fixture(false).values);
  assert!(!super::valid_id("../test.cube"));assert!(!super::valid_id("lut-foo.cube"));
  assert!(super::valid_id("lut-123-4-0.cube"));assert!(super::from_strip(&image::RgbaImage::new(3,2)).is_err());
 }
}
