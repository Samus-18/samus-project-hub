Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
cbuffer Options:register(b0){float4 Params;};
float3 at(int2 p,int2 lim){return Source.Load(int3(clamp(p,0,lim),0)).rgb;}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;int2 p=int2(id.xy),lim=int2(w-1,h-1);
 float3 c=at(p,lim),a=at(p+int2(-1,0),lim),b=at(p+int2(1,0),lim),d=at(p+int2(0,-1),lim),e=at(p+int2(0,1),lim);
 float3 lo=min(c,min(min(a,b),min(d,e))),hi=max(c,max(max(a,b),max(d,e)));
 float3 detail=c-(a+b+d+e)*.25;
 float3 sharpened=clamp(c+detail*.9*Params.x,lo,hi);
 Target[id.xy]=float4(saturate(sharpened),1);
}