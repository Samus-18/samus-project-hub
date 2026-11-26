Texture2D<float4> Source : register(t0);
RWTexture2D<float> Exposure : register(u0);
groupshared uint bins[64];
[numthreads(256,1,1)]
void CSMain(uint3 thread:SV_GroupThreadID){
 uint lane=thread.x;
 if(lane<64) bins[lane]=0;
 GroupMemoryBarrierWithGroupSync();
 uint w,h; Source.GetDimensions(w,h);
 for(uint i=lane;i<16384;i+=256){
    uint2 g=uint2(i%128,i/128);
    uint2 p=min(uint2((float2(g)+.5)*float2(w,h)/128.0),uint2(w-1,h-1));
    float3 rgb=saturate(Source.Load(int3(p,0)).rgb);
    float y=dot(rgb,float3(.2126,.7152,.0722));
    uint b=min(63u,(uint)(y*64.0));
    InterlockedAdd(bins[b],1);
 }
 GroupMemoryBarrierWithGroupSync();
 if(lane==0){
    uint n=0;float total=0;uint used=0;
    // Trim bottom and top 5% of the sampled luminance distribution.
    for(uint b=0;b<64;b++){
      uint next=n+bins[b];
      uint lo=max(n,819u),hi=min(next,15565u);
      if(hi>lo){uint count=hi-lo;total+=(float(b)+.5)/64.0*count;used+=count;}
      n=next;
    }
    float mean=total/max(1.0,float(used));
    Exposure[uint2(0,0)]=clamp(.43/max(.10,mean),.72,1.48);
 }
}