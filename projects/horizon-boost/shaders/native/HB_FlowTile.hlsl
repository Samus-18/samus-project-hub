#ifndef FLOW_TILE
#define FLOW_TILE 16
#endif
// Horizon Boost: independent forward and backward 2D color motion estimation.
// Experimental external interpolation, never game motion vectors / FSR3.
// Coarse 6px then 2px local refinement; one 16x16 spatial field sample.
// Memory: 4 floats per tile = prev->cur XY and cur->prev XY.
Texture2D<float4> Previous:register(t0);
Texture2D<float4> Current:register(t1);
RWTexture2D<float4> FlowField:register(u0);
float luminance(float3 rgb){return dot(rgb,float3(.2126,.7152,.0722));}
float3 loadclamped(Texture2D<float4> tex,int2 p,int2 cap){
    return tex.Load(int3(clamp(p,int2(0,0),cap),0)).rgb;
}
float cost(Texture2D<float4> source,Texture2D<float4> target,
           int2 at,int2 delta,int2 cap){
    int2 taps[5]={int2(0,0),int2(-4,0),int2(4,0),int2(0,-4),int2(0,4)};
    float score=0.0;
    [unroll] for(int k=0;k<5;k++){
        float3 a=loadclamped(source,at+taps[k],cap);
        float3 b=loadclamped(target,at+taps[k]+delta,cap);
        score+=abs(luminance(a)-luminance(b))+
             0.08*length(a-b);
    }
    return score+0.004*length(float2(delta));
}
float2 estimate(Texture2D<float4> source,Texture2D<float4> target,
                int2 at,int2 cap){
    int2 choices[9]={int2(0,0),int2(6,0),int2(-6,0),
       int2(0,6),int2(0,-6),int2(6,6),int2(6,-6),
       int2(-6,6),int2(-6,-6)};
    float best=1e20;
    int2 motion=int2(0,0);
    [unroll]for(int i=0;i<9;i++){
        float c=cost(source,target,at,choices[i],cap);
        if(c<best){best=c;motion=choices[i];}
    }
    int2 coarse=motion;
    [unroll]for(int i=0;i<9;i++){
        // Refine around the fixed coarse solution; do not drift the search
        // origin as successive candidates are evaluated.
        int2 candidate=coarse+choices[i]/3;
        float c=cost(source,target,at,candidate,cap);
        if(c<best){best=c;motion=candidate;}
    }
    return float2(motion);
}

#if defined(FLOW_PYRAMID)
Texture2D<float4> Coarse:register(t2);
float2 estimate_pyramid(Texture2D<float4> source,Texture2D<float4> target,int2 at,int2 cap,int2 seed){
 int2 options[13]={int2(0,0),int2(2,0),int2(-2,0),int2(0,2),int2(0,-2),
 int2(4,0),int2(-4,0),int2(0,4),int2(0,-4),
 int2(2,2),int2(2,-2),int2(-2,2),int2(-2,-2)};
 float best=1e20;int2 motion=seed;
 [unroll]for(int i=0;i<13;i++){
  int2 candidate=clamp(seed+options[i],int2(-36,-36),int2(36,36));
  float c=cost(source,target,at,candidate,cap);
  if(c<best){best=c;motion=candidate;}
 }
 float stationary=cost(source,target,at,int2(0,0),cap);
 if(stationary<best){motion=int2(0,0);}
 return float2(motion);
}
#endif
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
    uint tw,th;FlowField.GetDimensions(tw,th);
    if(id.x>=tw||id.y>=th)return;
    uint w,h;Previous.GetDimensions(w,h);
    int2 cap=int2(w-1,h-1);
    int2 center=int2(id.xy)*FLOW_TILE+int2(FLOW_TILE/2,FLOW_TILE/2);
    #if defined(FLOW_PYRAMID)
    float4 seed=Coarse.Load(int3(id.xy,0));
    float2 forward=estimate_pyramid(Previous,Current,center,cap,int2(round(seed.xy)));
#else
    float2 forward=estimate(Previous,Current,center,cap);
#endif
    #if defined(FLOW_PYRAMID)
    float2 backward=estimate_pyramid(Current,Previous,center,cap,int2(round(seed.zw)));
#else
    float2 backward=estimate(Current,Previous,center,cap);
#endif
    FlowField[id.xy]=float4(forward,backward);
}
