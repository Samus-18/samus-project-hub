// Original SDR CRT shadow-mask simulation; no game hooks.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 tap(int2 p,int2 s){return Source.Load(int3(clamp(p,int2(0,0),s-1),0)).rgb;}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),s=int2(w,h);float4 base=Source.Load(int3(p,0));
 float3 bleed=tap(p+int2(-1,0),s)*.16+base.rgb*.68+tap(p+int2(1,0),s)*.16;
 uint triad=(id.x+(id.y/2u)%2u)%3u;
 float3 mask=triad==0u?float3(1.12,.84,.84):(triad==1u?float3(.84,1.12,.84):float3(.84,.84,1.12));
 float scanGain=(id.y%2u==0u)?1.:.76;
 float2 uv=(float2(p)+.5)/float2(s)*2.-1.;
 float edge=1.-.17*saturate(dot(uv,uv)*.5);
 Target[id.xy]=float4(saturate(bleed*mask*scanGain*edge),base.a);
}
