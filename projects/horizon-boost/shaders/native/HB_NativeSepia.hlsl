// Native GPU compute shader by Horizon Boost, no game hooks or Magpie runtime.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 transform(float3 c,uint2 xy){
return saturate(float3(dot(c,float3(.393,.769,.189)),dot(c,float3(.349,.686,.168)),dot(c,float3(.272,.534,.131))));}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint width,height;Target.GetDimensions(width,height);
 if(id.x>=width||id.y>=height)return;
 float4 c=Source.Load(int3(id.xy,0));
 Target[id.xy]=float4(transform(saturate(c.rgb),id.xy),c.a);
}
