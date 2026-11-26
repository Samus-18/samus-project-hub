Texture2D<float4> Source:register(t0);
Texture2D<float4> Lut:register(t1);
RWTexture2D<float4> Target:register(u0);
#ifdef LIVE_LUT
cbuffer Mix:register(b0){float4 Params;};
#endif
float3 at(uint3 p,uint n){return Lut.Load(int3(p.x+p.z*n,p.y,0)).rgb;}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h,lw,n;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;Lut.GetDimensions(lw,n);
 float3 q=saturate(Source.Load(int3(id.xy,0)).rgb)*(n-1);
 uint3 a=(uint3)floor(q),b=min(a+1,n-1);float3 f=frac(q);
 float3 z0=lerp(lerp(at(uint3(a.x,a.y,a.z),n),at(uint3(b.x,a.y,a.z),n),f.x),lerp(at(uint3(a.x,b.y,a.z),n),at(uint3(b.x,b.y,a.z),n),f.x),f.y);
 float3 z1=lerp(lerp(at(uint3(a.x,a.y,b.z),n),at(uint3(b.x,a.y,b.z),n),f.x),lerp(at(uint3(a.x,b.y,b.z),n),at(uint3(b.x,b.y,b.z),n),f.x),f.y);
 float3 mapped=saturate(lerp(z0,z1,f.z));
#ifdef LIVE_LUT
 mapped=lerp(Source.Load(int3(id.xy,0)).rgb,mapped,saturate(Params.x));
#endif
 Target[id.xy]=float4(mapped,1);
}