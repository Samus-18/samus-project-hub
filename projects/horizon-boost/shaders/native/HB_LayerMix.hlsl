Texture2D<float4> Original:register(t0);
Texture2D<float4> Processed:register(t1);
RWTexture2D<float4> Target:register(u0);
cbuffer Blend:register(b0){float4 P;};
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 float4 a=Original.Load(int3(id.xy,0)),b=Processed.Load(int3(id.xy,0));
 float t=P.y>.5?step(P.x,(float(id.x)+.5)/float(w)):P.x;
 Target[id.xy]=lerp(a,b,t);
}