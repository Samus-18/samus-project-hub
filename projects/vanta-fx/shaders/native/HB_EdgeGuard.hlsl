Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 px(int2 p,int2 s){return Source.Load(int3(clamp(p,int2(0,0),s-1),0)).rgb;}
float luma(float3 c){return dot(c,float3(.2126,.7152,.0722));}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),s=int2(w,h);float3 c=px(p,s);
float3 a=px(p+int2(-1,0),s),b=px(p+int2(1,0),s),d=px(p+int2(0,-1),s),e=px(p+int2(0,1),s);
 float edge=max(abs(luma(a)-luma(b)),abs(luma(d)-luma(e)));
 float3 blur=(a+b+d+e)*.25;float weight=.62*(1-smoothstep(.12,.5,edge));
 Target[id.xy]=float4(saturate(c+clamp((c-blur)*weight,-.06,.06)),1);}
