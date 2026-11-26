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
    float2 distortion=float2(
        sin(uv.y*62.0+cos(uv.x*51.0))*2.1,
        cos(uv.x*74.0+sin(uv.y*47.0))*1.9);
    float3 a=tap(p+int2(distortion),lim);
    float3 b=tap(p-int2(distortion),lim);
    col=lerp(col,(a+b)*0.5,0.79);
    float edge=saturate(length((uv-0.5)*1.5));
    col=lerp(col,float3(0.70,0.84,0.98),edge*edge*0.13);
    Target[id.xy]=float4(saturate(col),base.a);
}
