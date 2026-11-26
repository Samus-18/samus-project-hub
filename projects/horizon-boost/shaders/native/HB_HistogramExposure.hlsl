Texture2D<float4> Source:register(t0);
Texture2D<float> Exposure:register(t1);
RWTexture2D<float4> Target:register(u0);
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 float3 c=Source.Load(int3(id.xy,0)).rgb;
 float gain=Exposure.Load(int3(0,0,0));
 float3 corrected=c*gain;
 // Preserve highlight detail in SDR, rather than clip all bright pixels.
 float3 result=lerp(corrected,c,smoothstep(.62,.98,max(c.r,max(c.g,c.b))));
 Target[id.xy]=float4(saturate(result),1);
}