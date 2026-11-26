Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 read_px(int2 p,int2 size){return Source.Load(int3(clamp(p,int2(0,0),size-1),0)).rgb;}
float lum(float3 c){return dot(c,float3(.2126,.7152,.0722));}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h; Target.GetDimensions(w,h); if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),size=int2(w,h);
 float3 c=read_px(p,size);

 float2 q=(float2(p)+.5-float2(size)*.5)*.25+float2(size)*.5-.5;
 int2 base=int2(floor(q));float2 f=frac(q);
 float3 col=lerp(lerp(read_px(base,size),read_px(base+int2(1,0),size),f.x),
                 lerp(read_px(base+int2(0,1),size),read_px(base+int2(1,1),size),f.x),f.y);
 Target[id.xy]=float4(col,1);
}
