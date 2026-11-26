// Native GPU compute shader by Horizon Boost, no game hooks or Magpie runtime.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 transform(float3 c,uint2 xy){
float l=dot(c,float3(0.2126,0.7152,0.0722));
float shadow=(1.0-smoothstep(0.06,0.58,l));
return saturate(c+shadow*0.30*float3(0.82,0.94,1.08));}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint width,height;Target.GetDimensions(width,height);
 if(id.x>=width||id.y>=height)return;
 float4 c=Source.Load(int3(id.xy,0));
 Target[id.xy]=float4(transform(saturate(c.rgb),id.xy),c.a);
}
