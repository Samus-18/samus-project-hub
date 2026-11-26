Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 float4 src=Source.Load(int3(id.xy,0));float3 c=src.rgb;
 float lum=dot(c,float3(0.2126,0.7152,0.0722));
 float range=max(c.r,max(c.g,c.b))-min(c.r,min(c.g,c.b));
 float amount=0.75*(1.0-saturate(range));
 Target[id.xy]=float4(saturate(lerp(float3(lum,lum,lum),c,1.0+amount)),src.a);
}
