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

 float3 n=s(p+int2(0,-1),mx),d=s(p+int2(0,1),mx);
 float3 l=s(p+int2(-1,0),mx),rt=s(p+int2(1,0),mx);
 float3 blur=(n+d+l+rt+c)*0.2;
 float3 grad=abs(c-blur);
 float flat=saturate(Y(grad)*12.0);
 float limits=saturate((1.0-Y(c))*4.0)*saturate(Y(c)*10.0);
 c+=clamp((c-blur)*0.48,-0.07,0.07)*flat*limits;
 Target[id.xy]=float4(saturate(c),v.a);
}
