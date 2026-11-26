// Native GPU compute shader by Horizon Boost, no game hooks or Magpie runtime.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 transform(float3 c,uint2 xy){
float3 v=float3(c.r*0.82+c.g*0.18,c.r*0.10+c.g*0.87+c.b*0.03,c.b*0.91+c.g*0.09);
return saturate(v);}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint width,height;Target.GetDimensions(width,height);
 if(id.x>=width||id.y>=height)return;
 float4 c=Source.Load(int3(id.xy,0));
 Target[id.xy]=float4(transform(saturate(c.rgb),id.xy),c.a);
}
