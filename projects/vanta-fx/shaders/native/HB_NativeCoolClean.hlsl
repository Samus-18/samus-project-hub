// Native GPU compute shader by Horizon Boost, no game hooks or Magpie runtime.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 transform(float3 c,uint2 xy){
float gray=dot(c,float3(0.2126,0.7152,0.0722));
float3 balanced=lerp(float3(gray,gray,gray),c,1.18);
balanced+=float3(-0.018,0.009,0.022);
return saturate((balanced-0.5)*1.08+0.5);}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint width,height;Target.GetDimensions(width,height);
 if(id.x>=width||id.y>=height)return;
 float4 c=Source.Load(int3(id.xy,0));
 Target[id.xy]=float4(transform(saturate(c.rgb),id.xy),c.a);
}
