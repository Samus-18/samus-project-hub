// Original Horizon Boost GPU screen-space pixel effect. Does not access game
// depth/motion vectors or claim true ray tracing, reflection or global illumination.
Texture2D<float4> Source: register(t0);
RWTexture2D<float4> Target: register(u0);
float3 tap(int2 p,int2 limit){return Source.Load(int3(clamp(p,int2(0,0),limit),0)).rgb;}
float luma(float3 a){return dot(a,float3(0.2126,0.7152,0.0722));}
float hash21(float2 p){return frac(sin(dot(p,float2(127.1,311.7)))*43758.5453);}
[numthreads(8,8,1)]
void CSMain(uint3 id: SV_DispatchThreadID){
    uint w,h;Target.GetDimensions(w,h);
    if(id.x>=w||id.y>=h)return;
    int2 p=int2(id.xy),lim=int2(w-1,h-1);
    float2 uv=(float2(p)+0.5)/float2(w,h);
    float4 base=Source.Load(int3(p,0));
    float3 col=base.rgb;
    float defocus=saturate((abs(uv.y-0.47)-0.15)*3.0);
    float r=2.0+defocus*11.0;
    float3 blurred=col;
    blurred+=tap(p+int2(r,0),lim)+tap(p-int2(r,0),lim);
    blurred+=tap(p+int2(0,r),lim)+tap(p-int2(0,r),lim);
    blurred/=5.0;
    col=lerp(col,blurred,defocus*0.94);
    Target[id.xy]=float4(saturate(col),base.a);
}
