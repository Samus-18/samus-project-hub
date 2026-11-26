Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 px(int2 p,int2 s){return Source.Load(int3(clamp(p,int2(0,0),s-1),0)).rgb;}
float lum(float3 c){return dot(c,float3(.2126,.7152,.0722));}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),s=int2(w,h);float3 c=px(p,s);
float2 uv=(float2(p)+.5)/float2(s)*2-1;float2 q=(uv*(1+.055*dot(uv,uv))*.5+.5)*float2(s)-.5;
 int2 b=int2(floor(q));float2 f=frac(q);float3 col=lerp(lerp(px(b,s),px(b+int2(1,0),s),f.x),lerp(px(b+int2(0,1),s),px(b+int2(1,1),s),f.x),f.y);
 Target[id.xy]=float4(col,1);}
