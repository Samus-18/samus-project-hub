#ifndef FLOW_TILE
#define FLOW_TILE 16
#endif
// GPU-only scene discontinuity estimate for 2D external interpolation.
// Samples 8x8 distributed locations, motion compensates using forward flow.
// A high global mismatch -> discard synthetic blends and retain real frame.
// This does NOT change engine FPS. Works entirely in D3D11 compute.
Texture2D<float4> Previous:register(t0);
Texture2D<float4> Current:register(t1);
Texture2D<float4> FlowField:register(t2);
RWTexture2D<float> SceneCut:register(u0);
[numthreads(1,1,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
    uint w,h;Current.GetDimensions(w,h);
    int2 lim=int2(w-1,h-1);
    float total=0.0;
    [loop]for(uint y=0;y<8;y++){
        [loop]for(uint x=0;x<8;x++){
            int2 p=int2((x*2+1)*w/16,(y*2+1)*h/16);
            p=clamp(p,int2(0,0),lim);
            float2 motion=FlowField.Load(int3(p/FLOW_TILE,0)).xy;
            int2 shifted=clamp(p+int2(round(motion)),int2(0,0),lim);
            float3 a=Previous.Load(int3(p,0)).rgb;
            float3 b=Current.Load(int3(shifted,0)).rgb;
            total+=max(abs(a.r-b.r),max(abs(a.g-b.g),abs(a.b-b.b)));
        }
    }
    // 64 independent tiles, RGB normalized [0,1].
    // Conservative threshold: only dramatic changes skip synthesis.
    SceneCut[int2(0,0)]=(total/64.0>0.35)?1.0:0.0;
}
