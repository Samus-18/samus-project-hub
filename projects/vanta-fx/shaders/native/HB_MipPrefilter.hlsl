Texture2D<float4> Source:register(t0);
SamplerState LinearClamp:register(s0);
RWTexture2D<float4> Target:register(u0);
cbuffer Options:register(b0){float4 Params;};
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 float2 uv=(float2(id.xy)+.5)/float2(w,h);
 Target[id.xy]=float4(Source.SampleLevel(LinearClamp,uv,Params.x).rgb,1);
}