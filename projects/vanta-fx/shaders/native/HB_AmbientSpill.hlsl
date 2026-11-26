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

 int2 o[12]={int2(6,0),int2(-6,0),int2(0,6),int2(0,-6),
 int2(8,8),int2(-8,8),int2(8,-8),int2(-8,-8),
 int2(18,0),int2(-18,0),int2(0,18),int2(0,-18)};
 float3 bleed=0;float energy=0;
 [unroll]for(int i=0;i<12;i++){
  float3 v1=s(p+o[i],mx);
  float y1=Y(v1);
  float light=saturate((y1-0.48)*2.7);
  float weight=(i<8?1.0:0.52);
  bleed+=v1*light*weight;
  energy+=light*weight;
 }
 float3 spill=bleed/max(energy,0.001);
 float darkness=saturate((0.64-Y(c))*1.6);
 c=c+max(0.0,spill-c)*darkness*saturate(energy/8.0)*0.22;
 Target[id.xy]=float4(saturate(c),v.a);
}
