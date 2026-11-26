Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 Target[id.xy]=float4(Source.Load(int3(id.xy,0)).rgb,1);
}