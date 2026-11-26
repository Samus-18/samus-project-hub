#ifndef FLOW_TILE
#define FLOW_TILE 16
#endif
// External bidirectional 2D optical-flow blend. Independent motion fields
// and GPU-only scene-change flag. Not a game-integrated frame generator.
#ifndef FLOW_ALPHA
#define FLOW_ALPHA .5
#endif
Texture2D<float4> Previous:register(t0);
Texture2D<float4> Current:register(t1);
Texture2D<float4> FlowField:register(t2);
Texture2D<float> CutFlag:register(t3);
RWTexture2D<float4> Result:register(u0);
cbuffer MotionOptions:register(b0){float4 MaskRect;float4 Tuning;float4 Extras;};
float4 sample_flow(int2 p){
 if(Extras.x<0.5)return FlowField.Load(int3(p/FLOW_TILE,0));
 uint tw,th;FlowField.GetDimensions(tw,th);
 float2 q=clamp(float2(p)/float(FLOW_TILE)-0.5,0.0,float2(tw-1,th-1));
 int2 a=int2(floor(q));int2 b=min(a+1,int2(tw-1,th-1));float2 f=frac(q);
 return lerp(lerp(FlowField.Load(int3(a,0)),FlowField.Load(int3(b.x,a.y,0)),f.x),
             lerp(FlowField.Load(int3(a.x,b.y,0)),FlowField.Load(int3(b,0)),f.x),f.y);
}
float4 fetch_bilinear(Texture2D<float4> tex,float2 q,int2 lim){
    q=clamp(q,float2(0,0),float2(lim));
    int2 lo=int2(floor(q));
    int2 hi=min(lo+int2(1,1),lim);
    float2 t=saturate(q-float2(lo));
    float4 tl=tex.Load(int3(lo,0));
    float4 tr=tex.Load(int3(int2(hi.x,lo.y),0));
    float4 bl=tex.Load(int3(int2(lo.x,hi.y),0));
    float4 br=tex.Load(int3(hi,0));
    return lerp(lerp(tl,tr,t.x),lerp(bl,br,t.x),t.y);
}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
    uint w,h;Result.GetDimensions(w,h);
    if(id.x>=w||id.y>=h)return;
    int2 p=int2(id.xy),lim=int2(w-1,h-1);
    float4 currentPixel=Current.Load(int3(p,0));
    // Entire scene replaced; displaying the real current frame is safer.
    bool cut=CutFlag.Load(int3(0,0,0))>=0.5;
    float2 uv=(float2(p)+0.5)/float2(w,h);
    bool protectedArea=Tuning.w>0.5&&all(uv>=MaskRect.xy)&&all(uv<=MaskRect.zw);
    if(Extras.y<0.5&&(cut||protectedArea)){
        Result[p]=float4(currentPixel.rgb,1.0);
        return;
    }
    float4 vectors=sample_flow(p);
    float2 fwd=vectors.xy,rev=vectors.zw;
    // External capture-only temporal mode, using current->previous optical flow.
    // Do not label this game-engine TAA: no jitter, depth or engine motion vectors.
    if(Extras.w>0.5){
        if(Extras.z<=0.0||cut||protectedArea||max(length(fwd),length(rev))>Tuning.z){
            Result[p]=float4(currentPixel.rgb,1);return;
        }
        float2 oldPos=clamp(float2(p)+rev,0.0,float2(lim));
        float3 history=fetch_bilinear(Previous,oldPos,lim).rgb;
        float2 oldFwd=sample_flow(int2(oldPos)).xy;
        float mismatch=length(history-currentPixel.rgb);
        float agreement=length(oldFwd+rev);
        float confidence=saturate(1.0-(max(0.0,mismatch-.035)*4.0+max(0.0,agreement-1.0)*.25)*Tuning.y);
        if(confidence<=0.0){Result[p]=float4(currentPixel.rgb,1);return;}
        float3 lo=currentPixel.rgb,hi=currentPixel.rgb;
        [unroll]for(int dy=-1;dy<=1;dy++){
            [unroll]for(int dx=-1;dx<=1;dx++){
                float3 n=Current.Load(int3(clamp(p+int2(dx,dy),int2(0,0),lim),0)).rgb;
                lo=min(lo,n);hi=max(hi,n);
            }
        }
        float3 result=currentPixel.rgb;
        if(Extras.w<1.5){
            // Reproject exactly one unfiltered prior frame; neighborhood clamp bounds ghosting.
            result=lerp(result,clamp(history,lo,hi),saturate(Extras.z)*confidence);
        }else{
            // Motion-direction shutter approximation using bounded current spatial samples.
            float mag=length(rev);
            if(mag>.25){
                float2 step=rev*Extras.z*.5;
                float3 blur=(fetch_bilinear(Current,float2(p)-step,lim).rgb+
                             fetch_bilinear(Current,float2(p)-step*.5,lim).rgb+
                             currentPixel.rgb+
                             fetch_bilinear(Current,float2(p)+step*.5,lim).rgb+
                             fetch_bilinear(Current,float2(p)+step,lim).rgb)*.2;
                result=lerp(result,blur,confidence);
            }
        }
        float3 prevPixel=Previous.Load(int3(p,0)).rgb;
        float edge=length(hi-lo);
        float still=length(currentPixel.rgb-prevPixel);
        float preserve=saturate((edge-.18)*3.4)*saturate((.10-still)*12.0);
        Result[p]=float4(saturate(lerp(result,currentPixel.rgb,preserve*Tuning.x)),1);return;
    }
    float2 locOld=float2(p)-fwd*FLOW_ALPHA;
    float2 locNew=float2(p)-rev*(1.0-FLOW_ALPHA);
    float4 old=fetch_bilinear(Previous,locOld,lim);
    float4 now=fetch_bilinear(Current,locNew,lim);
    float mismatch=length(old.rgb-now.rgb);
    // Independent direction agreement: trustworthy motion approximately
    // has reverse and forward estimates that cancel each other.
    float disagreement=length(fwd+rev);
    float reject=saturate(((mismatch-0.18)*4.0+max(0.0,disagreement-2.0)*0.19)*Tuning.y);
    bool excessive=max(length(fwd),length(rev))>Tuning.z;
    if(Extras.y>1.5){
        // Red/green encode signed forward XY, blue encodes disagreement.
        Result[p]=float4(saturate(float3(.5+fwd.x/24.0,.5+fwd.y/24.0,disagreement/16.0)),1);
        return;
    }
    if(Extras.y>0.5){
        // Heuristic rejection map, not ground-truth occlusion or confidence.
        float bad=(cut||excessive)?1.0:reject;
        Result[p]=protectedArea?float4(.1,.3,1,1):float4(bad,1-bad,0,1);
        return;
    }
    if(excessive){Result[p]=float4(currentPixel.rgb,1);return;}
    float3 interpolated=lerp(old.rgb,now.rgb,FLOW_ALPHA);
    // Real bidirectional consistency test for color-only disocclusion:
    // Diverging forward/reverse vectors and a discontinuity between adjacent
    // flow tiles indicate a possible revealed region. There is no depth or
    // geometry buffer from the game, so borrow from the actual latest frame,
    // never hallucinate a missing pixel from stale history.
    float2 nextX=sample_flow(min(p+int2(FLOW_TILE,0),lim)).xy;
    float2 nextY=sample_flow(min(p+int2(0,FLOW_TILE),lim)).xy;
    float motionEdge=max(length(nextX-fwd),length(nextY-fwd));
    float unreliable=saturate(max(0.0,disagreement-1.1)*0.24+
                              max(0.0,mismatch-0.12)*2.6+
                              max(0.0,motionEdge-1.7)*0.16);
    float holeCoverage=saturate(unreliable*Extras.z);
    float fallback=max(reject*0.93,holeCoverage);
    float3 safe=lerp(interpolated,currentPixel.rgb,saturate(fallback));
    // Preserve unchanged high-contrast overlays/text based on still edges.
    // Heuristic only, not semantic per-game HUD recognition.
    int2 dx=min(p+int2(1,0),lim);
    int2 dy=min(p+int2(0,1),lim);
    float3 adjx=Current.Load(int3(dx,0)).rgb;
    float3 adjy=Current.Load(int3(dy,0)).rgb;
    float3 prevPixel=Previous.Load(int3(p,0)).rgb;
    float edges=length(currentPixel.rgb-adjx)+length(currentPixel.rgb-adjy);
    float still=length(currentPixel.rgb-prevPixel);
    float preserve=saturate((edges-0.18)*3.4)*
                   saturate((0.10-still)*12.0);
    safe=lerp(safe,currentPixel.rgb,preserve*Tuning.x);
    Result[p]=float4(saturate(safe),1.0);
}
