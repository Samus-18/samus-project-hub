// Horizon Boost: bounded 3x3 edge-aware bilateral denoise, D3D11 compute.
// Runs entirely on the owner's captured pixel texture; no game engine data.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 fetch_rgb(int2 p,int2 size){
    return Source.Load(int3(clamp(p,int2(0,0),size-1),0)).rgb;
}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
    uint w,h;Target.GetDimensions(w,h);
    if(id.x>=w||id.y>=h)return;
    int2 xy=int2(id.xy);
    int2 lim=int2(w,h);
    float3 center=fetch_rgb(xy,lim);
    float center_luma=dot(center,float3(0.2126,0.7152,0.0722));
    float3 sum=0;float total=0;
    [unroll] for(int dy=-1;dy<=1;dy++){
        [unroll] for(int dx=-1;dx<=1;dx++){
            float3 sample_rgb=fetch_rgb(xy+int2(dx,dy),lim);
            float lum=dot(sample_rgb,float3(0.2126,0.7152,0.0722));
            float color_distance=abs(center_luma-lum);
            float spatial=(dx==0&&dy==0)?1.0:((dx==0||dy==0)?0.65:0.40);
            float weight=spatial*exp2(-color_distance*43.0);
            sum+=weight*sample_rgb;total+=weight;
        }
    }
    float3 smooth=sum/max(total,0.00001);
    float3 out_color=lerp(center,smooth,0.60);
    Target[id.xy]=float4(saturate(out_color),1.0);
}