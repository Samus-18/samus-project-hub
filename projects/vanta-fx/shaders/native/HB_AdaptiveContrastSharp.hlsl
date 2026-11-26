Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 read_px(int2 p,int2 size){return Source.Load(int3(clamp(p,int2(0,0),size-1),0)).rgb;}
float lum(float3 c){return dot(c,float3(.2126,.7152,.0722));}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h; Target.GetDimensions(w,h); if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),size=int2(w,h);
 float3 c=read_px(p,size);

 float3 a=read_px(p+int2(-1,0),size),b=read_px(p+int2(1,0),size);
 float3 u=read_px(p+int2(0,-1),size),d=read_px(p+int2(0,1),size);
 float3 mn=min(c,min(min(a,b),min(u,d))),mx=max(c,max(max(a,b),max(u,d)));
 float3 blur=(a+b+u+d)*.25;
 float edge=lum(mx-mn);
 float gain=.50*(1.0-smoothstep(.12,.55,edge));
 float3 detail=(c-blur)*gain;
 float3 col=clamp(c+detail,mn-.025,mx+.025);
 Target[id.xy]=float4(saturate(col),1);
}
