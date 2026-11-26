//! Native SMAA 1x High, original MIT reference algorithm and lookup textures.
//! Three D3D11 compute passes, explicit luma edges, weight and neighborhood blend.
//! Only Horizon's independent GPU output; no game depth or injection.
use windows::Win32::Graphics::{
 Direct3D11::*,
 Dxgi::Common::*,
};
const REFERENCE:&str=include_str!("../assets/smaa/SMAA.hlsli");
const AREA:&[u8]=include_bytes!("../assets/smaa/AreaTex.dds");
const SEARCH:&[u8]=include_bytes!("../assets/smaa/SearchTex.dds");

pub fn shader_source(stage:usize,w:u32,h:u32)->String{
 let (textures,body)=match stage{
 0=>(r#"Texture2D<float4> Source:register(t0);"#,
    "float2 e=SMAALumaEdgeDetectionPS(uv,Source);Output[gid.xy]=float4(e,0,0);"),
 1=>(r#"Texture2D<float2> Edges:register(t0);
Texture2D<float4> Area:register(t1);
Texture2D<float> Search:register(t2);"#,
    "Output[gid.xy]=SMAABlendingWeightCalculationPS(uv,Edges,Area,Search,float4(0,0,0,0));"),
 2=>(r#"Texture2D<float4> Source:register(t0);
Texture2D<float4> Weights:register(t1);"#,
    "float4 c=SMAANeighborhoodBlendingPS(uv,Source,Weights);float4 orig=Source.Load(int3(gid.xy,0));Output[gid.xy]=float4(saturate(c.rgb),orig.a);"),
 _=>unreachable!()
 };
 format!(r#"
#define SMAA_PRESET_HIGH 1
#define SMAA_LINEAR_SAMPLER LinearSampler
#define SMAA_POINT_SAMPLER PointSampler
static const float4 SMAA_RT_METRICS=float4(1.0/{w}.0,1.0/{h}.0,{w}.0,{h}.0);
SamplerState LinearSampler:register(s0);
SamplerState PointSampler:register(s1);
{textures}
RWTexture2D<float4> Output:register(u0);
{REFERENCE}
[numthreads(8,8,1)]
void CSMain(uint3 gid:SV_DispatchThreadID){{
 if(gid.x>={w}||gid.y>={h})return;
 float2 uv=(float2(gid.xy)+0.5)*float2(1.0/{w}.0,1.0/{h}.0);
 {body}
}}
"#)
}
fn compiled(device:&ID3D11Device,src:&str)->Result<ID3D11ComputeShader,String>{
 let bytes=crate::shadercheck::compile_native_hlsl(src)?;
 let mut out=None;
 unsafe{device.CreateComputeShader(&bytes,None::<&ID3D11ClassLinkage>,Some(&mut out))}.map_err(|e|e.to_string())?;
 out.ok_or("SMAA compute shader missing".into())
}
pub fn compile_all()->Result<(),String>{
 for i in 0..3{crate::shadercheck::compile_native_hlsl(&shader_source(i,73,51)).map_err(|e|format!("SMAA pass {i}: {e}"))?;}
 Ok(())
}
struct Surface{
 _tex:ID3D11Texture2D,
 srv:ID3D11ShaderResourceView,
 uav:Option<ID3D11UnorderedAccessView>,
}
fn surface(device:&ID3D11Device,w:u32,h:u32,format:DXGI_FORMAT,bytes:Option<(&[u8],u32)>)->Result<Surface,String>{
 let desc=D3D11_TEXTURE2D_DESC{
  Width:w,Height:h,MipLevels:1,ArraySize:1,Format:format,
  SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,
  BindFlags:if bytes.is_some(){D3D11_BIND_SHADER_RESOURCE.0 as u32}
    else{(D3D11_BIND_SHADER_RESOURCE.0|D3D11_BIND_UNORDERED_ACCESS.0) as u32},
  CPUAccessFlags:0,MiscFlags:0,
 };
 let mut tex=None;
 let initial=bytes.map(|(raw,bpp)|D3D11_SUBRESOURCE_DATA{
  pSysMem:raw.as_ptr().cast(),SysMemPitch:w*bpp,SysMemSlicePitch:0
 });
 match initial.as_ref(){
  Some(data)=>unsafe{device.CreateTexture2D(&desc,Some(data as *const _),Some(&mut tex))},
  None=>unsafe{device.CreateTexture2D(&desc,None,Some(&mut tex))}
 }.map_err(|e|format!("SMAA texture allocation: {e}"))?;
 let tex=tex.ok_or("SMAA texture missing")?;
 let mut srv=None;unsafe{device.CreateShaderResourceView(&tex,None,Some(&mut srv))}.map_err(|e|e.to_string())?;
 let mut uav=None;
 if bytes.is_none(){unsafe{device.CreateUnorderedAccessView(&tex,None,Some(&mut uav))}.map_err(|e|e.to_string())?;}
 Ok(Surface{_tex:tex,srv:srv.ok_or("SMAA shader view missing")?,uav})
}
fn dds(device:&ID3D11Device,blob:&[u8],width:u32,height:u32,format:DXGI_FORMAT,bpp:u32)->Result<Surface,String>{
 if blob.len()<128||blob.get(0..4)!=Some(b"DDS ")||
    u32::from_le_bytes(blob[12..16].try_into().unwrap())!=height||
    u32::from_le_bytes(blob[16..20].try_into().unwrap())!=width||
    blob.len()!=128+width as usize*height as usize*bpp as usize{
    return Err("SMAA reference lookup DDS corrupt or mismatched".into());
 }
 surface(device,width,height,format,Some((&blob[128..],bpp)))
}
fn sampler(device:&ID3D11Device,filter:D3D11_FILTER)->Result<ID3D11SamplerState,String>{
 let mut desc=D3D11_SAMPLER_DESC::default();
 desc.Filter=filter;desc.AddressU=D3D11_TEXTURE_ADDRESS_CLAMP;desc.AddressV=D3D11_TEXTURE_ADDRESS_CLAMP;
 desc.AddressW=D3D11_TEXTURE_ADDRESS_CLAMP;desc.MaxLOD=f32::MAX;desc.ComparisonFunc=D3D11_COMPARISON_NEVER;desc.MaxAnisotropy=1;
 let mut out=None;unsafe{device.CreateSamplerState(&desc,Some(&mut out))}.map_err(|e|e.to_string())?;
 out.ok_or("SMAA sampler missing".into())
}
pub struct Pass{
 width:u32,height:u32,
 shaders:[ID3D11ComputeShader;3],
 linear:ID3D11SamplerState,point:ID3D11SamplerState,
 edges:Surface,blend:Surface,area:Surface,search:Surface,
}
impl Pass{
 pub fn new(device:&ID3D11Device,w:u32,h:u32)->Result<Self,String>{
  if w<2||h<2||w>4096||h>4096||u64::from(w)*u64::from(h)>8_294_400{
   return Err("SMAA 1x exceeds GPU processing size budget".into());
  }
  let mut programs=Vec::with_capacity(3);for i in 0..3{programs.push(compiled(device,&shader_source(i,w,h))?);}let shaders=programs.try_into().map_err(|_|"SMAA shader count invalid")?;
  Ok(Self{
   width:w,height:h,shaders,linear:sampler(device,D3D11_FILTER_MIN_MAG_MIP_LINEAR)?,
   point:sampler(device,D3D11_FILTER_MIN_MAG_MIP_POINT)?,
   edges:surface(device,w,h,DXGI_FORMAT_R16G16B16A16_FLOAT,None)?,
   blend:surface(device,w,h,DXGI_FORMAT_R16G16B16A16_FLOAT,None)?,
   area:dds(device,AREA,160,560,DXGI_FORMAT_R8G8B8A8_UNORM,4)?,
   search:dds(device,SEARCH,64,16,DXGI_FORMAT_R8_UNORM,1)?,
  })
 }
 fn run_pass(&self,ctx:&ID3D11DeviceContext,n:usize,
  inputs:&[Option<ID3D11ShaderResourceView>],out:&ID3D11UnorderedAccessView){
  unsafe{
   ctx.CSSetShader(&self.shaders[n],None);
   ctx.CSSetShaderResources(0,Some(inputs));
   let target=[Some(out.clone())];ctx.CSSetUnorderedAccessViews(0,1,Some(target.as_ptr()),None);
   ctx.Dispatch(self.width.div_ceil(8),self.height.div_ceil(8),1);
   ctx.CSSetShaderResources(0,Some(&[None,None,None,None]));
   let none:[Option<ID3D11UnorderedAccessView>;1]=[None];
   ctx.CSSetUnorderedAccessViews(0,1,Some(none.as_ptr()),None);
  }
 }
 pub fn render(&self,ctx:&ID3D11DeviceContext,input:&ID3D11ShaderResourceView,out:&ID3D11UnorderedAccessView){
  unsafe{ctx.CSSetSamplers(0,Some(&[Some(self.linear.clone()),Some(self.point.clone())]));}
  self.run_pass(ctx,0,&[Some(input.clone())],self.edges.uav.as_ref().unwrap());
  self.run_pass(ctx,1,&[Some(self.edges.srv.clone()),Some(self.area.srv.clone()),Some(self.search.srv.clone())],self.blend.uav.as_ref().unwrap());
  self.run_pass(ctx,2,&[Some(input.clone()),Some(self.blend.srv.clone())],out);
  unsafe{ctx.CSSetSamplers(0,Some(&[None,None]));ctx.CSSetShader(None::<&ID3D11ComputeShader>,None);}
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn original_high_quality_three_pass_reference_compiles(){compile_all().unwrap();}
 #[test]fn native_gpu_smaa_changes_edges_without_touching_alpha(){
  let g=crate::photo_lab::Gpu::new().unwrap();
  let im=image::RgbaImage::from_fn(73,51,|x,y|{
   let v=if x as i32-(y as i32*9)/7>10{235}else{30};
   image::Rgba([v,v,v,163])
  });
  let source=g.upload(&im).unwrap();let pipeline=crate::native_pipeline::NativeGpuPipeline::new(&g.device,73,51).unwrap();
  let filtered=g.read(&pipeline.dispatch(&g.device,&g.ctx,&source,&[crate::native_filter::SMAA_STAGE]).unwrap()).unwrap();
  assert_eq!(filtered.dimensions(),(73,51));
  assert!(filtered.pixels().all(|p|p[3]==163));
  assert_eq!(filtered.get_pixel(0,50),im.get_pixel(0,50));
  assert_eq!(filtered.get_pixel(72,0),im.get_pixel(72,0));
  assert!(filtered.pixels().zip(im.pixels()).filter(|(a,b)|a!=b).count()>10);
  let twice=g.read(&pipeline.dispatch(&g.device,&g.ctx,&source,&[crate::native_filter::SMAA_STAGE]).unwrap()).unwrap();
  assert_eq!(filtered,twice);
  eprintln!("SMAA_1X_GPU=PASS EDGE_DIFF=true ALPHA_EXACT=true DETERMINISTIC=true");
 }
}
