Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 read_px(int2 p,int2 size){return Source.Load(int3(clamp(p,int2(0,0),size-1),0)).rgb;}
float lum(float3 c){return dot(c,float3(.2126,.7152,.0722));}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h; Target.GetDimensions(w,h); if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),size=int2(w,h);
 float3 c=read_px(p,size);

 float3 sum=c*4;
 sum+=read_px(p+int2(-3,0),size)+read_px(p+int2(3,0),size);
 sum+=read_px(p+int2(0,-3),size)+read_px(p+int2(0,3),size);
 float average=lum(sum/8);
 float gain=clamp(.48/max(.12,average),.78,1.32);
 float3 col=c*lerp(1.0,gain,.48);
 Target[id.xy]=float4(saturate(col),1);
}
