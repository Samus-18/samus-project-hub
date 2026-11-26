use serde::{Serialize,Deserialize};
use windows::Win32::Graphics::Direct3D11::*;
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(tag="op",rename_all="snake_case",deny_unknown_fields)]
pub enum Op{Exposure{ev:f32},Contrast{value:f32},Saturation{value:f32},Gamma{value:f32},Tint{rgb:[f32;3]},Invert{mix:f32},Shoulder{value:f32}}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default,deny_unknown_fields)]pub struct Recipe{pub enabled:bool,pub mix:f32,pub ops:Vec<Op>}
impl Default for Recipe{fn default()->Self{Self{enabled:false,mix:1.,ops:vec![Op::Contrast{value:1.1},Op::Saturation{value:1.12}]}}}
impl Recipe{
 pub fn valid(&self)->bool{
  let range=|v:f32,a:f32,b:f32|v.is_finite()&&(a..=b).contains(&v);
  range(self.mix,0.,1.)&&!self.ops.is_empty()&&self.ops.len()<=8&&self.ops.iter().all(|op|match op{
   Op::Exposure{ev}=>range(*ev,-2.,2.),Op::Contrast{value}=>range(*value,0.5,1.5),Op::Saturation{value}=>range(*value,0.,2.),Op::Gamma{value}=>range(*value,0.5,2.),Op::Tint{rgb}=>rgb.iter().all(|&v|range(v,-0.2,0.2)),Op::Invert{mix}=>range(*mix,0.,1.),Op::Shoulder{value}=>range(*value,0.,1.)})
 }
 pub fn sanitize(&mut self){if !self.valid(){*self=Self::default();}}
 pub fn active(&self)->bool{self.valid()&&self.enabled&&self.mix>0.}
 pub fn decode(s:&str)->Result<Self,String>{if s.len()>4096{return Err("Recipe limited to 4 KiB".into());}let r:Self=serde_json::from_str(s).map_err(|e|e.to_string())?;if !r.valid(){return Err("Recipe must have 1-8 bounded numeric operations".into());}Ok(r)}
 pub fn source(&self)->Result<String,String>{
  if !self.valid(){return Err("Invalid color recipe".into());}
  let mut s=String::from("Texture2D<float4> Input:register(t0);RWTexture2D<float4> Output:register(u0);[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){uint w,h;Output.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;float4 o=Input.Load(int3(id.xy,0));float3 c=o.rgb;");
  for op in &self.ops{let code=match op{
   Op::Exposure{ev}=>format!("c*=exp2({ev:.6});"),
   Op::Contrast{value}=>format!("c=(c-.5)*{value:.6}+.5;"),
   Op::Saturation{value}=>format!("c=lerp(dot(c,float3(.2126,.7152,.0722)).xxx,c,{value:.6});"),
   Op::Gamma{value}=>format!("c=pow(saturate(c),1.0/{value:.6});"),
   Op::Tint{rgb}=>format!("c+=float3({:.6},{:.6},{:.6});",rgb[0],rgb[1],rgb[2]),
   Op::Invert{mix}=>format!("c=lerp(c,1-c,{mix:.6});"),
   Op::Shoulder{value}=>format!("c=c/(1+max(c-.55,0)*{value:.6});"),
  };s.push_str(&code);s.push_str("c=saturate(c);");}
  s.push_str(&format!("Output[id.xy]=float4(lerp(o.rgb,c,{:.6}),o.a);}}",self.mix));Ok(s)
 }
}
pub struct Pass{shader:ID3D11ComputeShader}
impl Pass{
 pub fn new(d:&ID3D11Device,r:&Recipe)->Result<Self,String>{
  let bytes=crate::shadercheck::compile_native_hlsl(&r.source()?)?;let mut shader=None;
  unsafe{d.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut shader))}.map_err(|e|e.to_string())?;
  Ok(Self{shader:shader.ok_or("Recipe shader missing")?})
 }
 pub fn run(&self,ctx:&ID3D11DeviceContext,input:&ID3D11ShaderResourceView,target:&ID3D11UnorderedAccessView,w:u32,h:u32){
  unsafe{ctx.CSSetShader(&self.shader,None);ctx.CSSetShaderResources(0,Some(&[Some(input.clone())]));ctx.CSSetUnorderedAccessViews(0,1,Some([Some(target.clone())].as_ptr()),None);ctx.Dispatch(w.div_ceil(8),h.div_ceil(8),1);ctx.CSSetShaderResources(0,Some(&[None]));let clear:[Option<ID3D11UnorderedAccessView>;1]=[None];ctx.CSSetUnorderedAccessViews(0,1,Some(clear.as_ptr()),None);ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);}
 }
}
pub fn proof()->Result<String,String>{
 let g=crate::photo_lab::Gpu::new()?;let im=image::RgbaImage::from_fn(64,32,|x,y|image::Rgba([(x*4)as u8,(y*8)as u8,110,255]));let src=g.upload(&im)?;
 let pipe=crate::native_pipeline::NativeGpuPipeline::new(&g.device,64,32)?;
 let mut view=None;unsafe{g.device.CreateShaderResourceView(&src,None,Some(&mut view))}.map_err(|e|e.to_string())?;let view=view.ok_or("Missing view")?;
 let render=|r:Recipe|->Result<image::RgbaImage,String>{let mut t=crate::lighting::LightTuning::default();t.recipe=r;t.layers.strength.insert(7,0.);g.read(&pipe.dispatch_with_cached_srv_tuned(&g.ctx,&src,&view,&[7],&t)?)};
 let a=render(Recipe{enabled:true,mix:1.,ops:vec![Op::Invert{mix:1.}]})?;
 let b=render(Recipe{enabled:true,mix:1.,ops:vec![Op::Tint{rgb:[0.1,0.,-0.1]}]})?;
 if a==b||a==im||b==im||render(Recipe{enabled:true,mix:0.,..Default::default()})?!=im{return Err("Recipe GPU hot update or zero bypass failed".into());}
 if Recipe::decode(r#"{"enabled":true,"ops":[{"op":"inject","code":"while(true)"}]}"#).is_ok(){return Err("Unsafe op accepted".into());}
 Ok("RECIPE_GPU=PASS SAME_PIPE_HOT_UPDATE=true ZERO_EXACT=true BOUNDED_OPS=true NO_RAW_HLSL=true".into())
}
#[cfg(test)]mod tests{use super::*;#[test]fn recipe_limits_and_templates(){let r=Recipe::default();assert!(r.valid());assert!(r.source().unwrap().len()<2048);assert!(Recipe::decode(r#"{"ops":[{"op":"exposure","ev":999}]}"#).is_err());assert!(Recipe::decode(r#"{"ops":[{"op":"exposure","ev":1,"code":"run"}]}"#).is_err());assert!(!Recipe{ops:vec![Op::Gamma{value:1.};9],..r}.valid());}}
