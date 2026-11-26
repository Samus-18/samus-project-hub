//! Exact AMD MIT CAS 1.20190610 scaling kernel; independent D3D11 SDR output.
const A:&str=include_str!("../shaders/third_party/cas/ffx_a.h");
const CAS:&str=include_str!("../shaders/third_party/cas/ffx_cas.h");
pub fn shader_source()->String{
 format!("#define A_GPU 1\\n#define A_HLSL 1\\n{}\\n{}\\n{}\\n{}",A,r#"
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
AF3 CasLoad(ASU2 p){uint w,h;Source.GetDimensions(w,h);float3 v=Source.Load(int3(clamp(int2(p),int2(0,0),int2(w-1,h-1)),0)).rgb;return v*v;}
void CasInput(inout AF1 r,inout AF1 g,inout AF1 b){}
"#,CAS,r#"
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint iw,ih,ow,oh;Source.GetDimensions(iw,ih);Target.GetDimensions(ow,oh);
 if(id.x>=ow||id.y>=oh)return;
 AU4 c0,c1;CasSetup(c0,c1,0.35,float(iw),float(ih),float(ow),float(oh));
 float r,g,b;
 if(iw==ow&&ih==oh)CasFilter(r,g,b,id.xy,c0,c1,true);
 else CasFilter(r,g,b,id.xy,c0,c1,false);
 float2 p=(float2(id.xy)+.5)*float2(iw,ih)/float2(ow,oh)-.5;
 float alpha=Source.Load(int3(clamp(int2(round(p)),int2(0,0),int2(iw-1,ih-1)),0)).a;
 Target[id.xy]=float4(sqrt(saturate(float3(r,g,b))),alpha);
}
"#).replace("\\n","\n")
}
pub fn supported(src:(u32,u32),dst:(u32,u32))->bool{
 src.0>0&&src.1>0&&dst.0>=src.0&&dst.1>=src.1&&u64::from(dst.0)*u64::from(dst.1)<=4*u64::from(src.0)*u64::from(src.1)
}
#[cfg(test)]mod tests{
 #[test]fn amd_reference_and_scale_bounds(){
  assert!(super::A.len()>100000);assert!(super::CAS.contains("void CasFilter("));
  assert!(super::supported((640,360),(1280,720)));assert!(!super::supported((640,360),(1920,1080)));assert!(!super::supported((640,360),(320,180)));
  crate::shadercheck::compile_native_hlsl(&super::shader_source()).unwrap();
 }
}
