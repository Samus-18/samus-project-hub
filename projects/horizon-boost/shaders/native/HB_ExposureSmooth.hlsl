Texture2D<float> Current:register(t0);
Texture2D<float> Previous:register(t1);
RWTexture2D<float> Output:register(u0);
cbuffer Options:register(b0){float4 Params;};
[numthreads(1,1,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 float now=Current.Load(int3(0,0,0));
 if(Params.z>.5){Output[uint2(0,0)]=now;return;}
 float old=Previous.Load(int3(0,0,0));
 float next=(abs(now-old)>=Params.y)?now:lerp(old,now,saturate(Params.x));
 Output[uint2(0,0)]=clamp(next,.72,1.48);
}
