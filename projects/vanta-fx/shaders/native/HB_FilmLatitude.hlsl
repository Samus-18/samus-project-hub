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

 float3 low=c/(c+0.21)*1.14;
 float3 shoulder=c/(1.0+0.22*c);
 float mid=saturate((Y(c)-0.18)*2.3);
 float3 toned=lerp(low,shoulder,mid);
 c=lerp(c,toned,0.31);
 Target[id.xy]=float4(saturate(c),v.a);
}
