Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 float4 src=Source.Load(int3(id.xy,0));
 float l=dot(src.rgb,float3(0.2126,0.7152,0.0722));
 float gain=saturate(sqrt(l)*1.2+0.06);
 Target[id.xy]=float4(gain*0.13,gain,gain*0.2,src.a);
}
