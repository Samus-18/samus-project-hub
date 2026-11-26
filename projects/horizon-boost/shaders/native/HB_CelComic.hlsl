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
    float3 sx=tap(p+int2(1,0),lim)-tap(p+int2(-1,0),lim);
    float3 sy=tap(p+int2(0,1),lim)-tap(p+int2(0,-1),lim);
    float ink=saturate((length(sx)+length(sy)-0.13)*2.9);
    float lighting=luma(col);
    float tones=floor(lighting*4.0+0.50)/4.0;
    float3 cartoon=pow(saturate(col),0.90)*(0.76+0.38*tones);
    col=lerp(col,cartoon,0.82)*(1.0-0.72*ink);
    Target[id.xy]=float4(saturate(col),base.a);
}
