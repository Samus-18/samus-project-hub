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
    float2 focus=float2(0.72,0.18);
    float2 delta=(focus-uv)*float2(w,h)/13.0;
    float3 rays=0;
    [unroll]for(int j=1;j<=13;j++){
        float3 sample=tap(p+int2(round(delta*j)),lim);
        rays+=max(sample-0.72,0.0)*(1.0-(j-1)*0.045);
    }
    col=saturate(col+rays*0.14*float3(1.08,0.95,0.77));
    Target[id.xy]=float4(saturate(col),base.a);
}
