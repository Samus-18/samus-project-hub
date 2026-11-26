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
 float gx=abs(lum(a)-lum(b)),gy=abs(lum(u)-lum(d));
 float strength=smoothstep(.035,.20,max(gx,gy))*.42;
 float3 along=gx>gy?(u+d)*.5:(a+b)*.5;
 Target[id.xy]=float4(saturate(lerp(c,along,strength)),1);
}
