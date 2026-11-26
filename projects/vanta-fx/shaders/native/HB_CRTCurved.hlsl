// Original curved arcade CRT: barrel geometry, scanlines and phosphor bloom.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 tap(int2 p,int2 s){return Source.Load(int3(clamp(p,int2(0,0),s-1),0)).rgb;}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),s=int2(w,h);float4 base=Source.Load(int3(p,0));
 float2 uv=(float2(p)+.5)/float2(s)*2.-1.;
 float2 curved=uv*(1.+.075*dot(uv,uv));
 if(any(abs(curved)>1.)){Target[id.xy]=float4(0.,0.,0.,base.a);return;}
 float2 at=(curved*.5+.5)*float2(s)-.5;int2 a=int2(floor(at));float2 f=frac(at);
 float3 c=lerp(lerp(tap(a,s),tap(a+int2(1,0),s),f.x),lerp(tap(a+int2(0,1),s),tap(a+int2(1,1),s),f.x),f.y);
 float3 halo=(tap(a+int2(-2,0),s)+tap(a+int2(2,0),s)+tap(a+int2(0,-2),s)+tap(a+int2(0,2),s))*.25;
 float scanGain=0.82+0.18*cos(at.y*3.14159265);
 c=c*scanGain+max(halo-.65,0.)*.19;
 Target[id.xy]=float4(saturate(c*(1.-.12*saturate(dot(uv,uv)))),base.a);
}
