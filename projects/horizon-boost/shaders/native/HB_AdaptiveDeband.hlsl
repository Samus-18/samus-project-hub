Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 px(int2 p,int2 s){return Source.Load(int3(clamp(p,int2(0,0),s-1),0)).rgb;}
float luma(float3 c){return dot(c,float3(.2126,.7152,.0722));}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),s=int2(w,h);float3 c=px(p,s);
float3 a=px(p+int2(-2,0),s),b=px(p+int2(2,0),s),d=px(p+int2(0,-2),s),e=px(p+int2(0,2),s);
 float3 avg=(a+b+d+e)*.25;float span=max(max(abs(luma(a)-luma(c)),abs(luma(b)-luma(c))),max(abs(luma(d)-luma(c)),abs(luma(e)-luma(c))));
 float smooth=1-smoothstep(.018,.065,span);
 float noise=frac(sin(dot(float2(p),float2(12.9898,78.233)))*43758.5453)-.5;
 Target[id.xy]=float4(saturate(lerp(c,avg,.5*smooth)+noise/255.*smooth),Source.Load(int3(p,0)).a);}
