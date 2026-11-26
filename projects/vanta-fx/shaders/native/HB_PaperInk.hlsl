Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 px(int2 p,int2 s){return Source.Load(int3(clamp(p,int2(0,0),s-1),0)).rgb;}
float luma(float3 c){return dot(c,float3(.2126,.7152,.0722));}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),s=int2(w,h);float3 c=px(p,s);
float gx=luma(px(p+int2(1,0),s))-luma(px(p-int2(1,0),s));float gy=luma(px(p+int2(0,1),s))-luma(px(p-int2(0,1),s));
 float edge=smoothstep(.06,.26,length(float2(gx,gy)));float3 flat=lerp(c,floor(c*8+.5)/8,.45);Target[id.xy]=float4(lerp(flat,flat*.28,edge*.75),1);}
