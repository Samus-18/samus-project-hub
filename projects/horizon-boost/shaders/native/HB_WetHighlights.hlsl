// Horizon Boost original D3D11 HDR-like SDR screen-space postprocess.
// EXTERNAL COLOR ONLY. Depth, surface normals and real shadow volumes unknown.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 s(int2 p,int2 mx){return Source.Load(int3(clamp(p,int2(0,0),mx),0)).rgb;}
float Y(float3 c){return dot(c,float3(0.2126,0.7152,0.0722));}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),mx=int2(w-1,h-1);
 float4 v=Source.Load(int3(p,0));float3 c=v.rgb;

 float2 uv=(float2(p)+0.5)/float2(w,h);
 float floorMask=smoothstep(0.56,0.87,uv.y);
 int2 o[8]={int2(6,0),int2(-6,0),int2(15,0),int2(-15,0),
 int2(24,1),int2(-24,1),int2(4,-3),int2(-4,-3)};
 float3 sheen=0;
 [unroll]for(int i=0;i<8;i++){
  float3 t=s(p+o[i],mx);
  float glow=saturate((Y(t)-0.74)*4.0);
  sheen+=t*glow*(i<4?0.65:0.35);
 }
 float localGrad=abs(Y(s(p+int2(0,-2),mx))-Y(s(p+int2(0,2),mx)));
 c+=(1.0-c)*sheen*0.15*floorMask*saturate(1.0-localGrad*3.0);
 Target[id.xy]=float4(saturate(c),v.a);
}
