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

 float base=Y(c),outer=0;
 int2 q[8]={int2(4,0),int2(-4,0),int2(0,4),int2(0,-4),
 int2(10,6),int2(-10,6),int2(10,-6),int2(-10,-6)};
 [unroll]for(int i=0;i<8;i++)outer+=Y(s(p+q[i],mx));
 outer/=8.0;
 float crevice=saturate((outer-base-0.025)*3.4);
 c*=1.0-0.29*crevice;
 Target[id.xy]=float4(saturate(c),v.a);
}
