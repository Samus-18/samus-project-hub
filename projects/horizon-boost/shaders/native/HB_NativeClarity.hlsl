Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 xy=int2(id.xy),mx=int2(w-1,h-1);
 float4 v=Source.Load(int3(xy,0));
 float3 a=Source.Load(int3(clamp(xy+int2(-1,0),int2(0,0),mx),0)).rgb;
 float3 b=Source.Load(int3(clamp(xy+int2(1,0),int2(0,0),mx),0)).rgb;
 float3 c=Source.Load(int3(clamp(xy+int2(0,-1),int2(0,0),mx),0)).rgb;
 float3 d=Source.Load(int3(clamp(xy+int2(0,1),int2(0,0),mx),0)).rgb;
 float3 blur=(a+b+c+d+v.rgb)*0.2;
 Target[id.xy]=float4(saturate(v.rgb+(v.rgb-blur)*0.86),v.a);
}
