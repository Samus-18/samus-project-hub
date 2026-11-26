// SDR display-referred filmic look, ACES-style fitted shoulder. Not HDR output.
Texture2D<float4> Input:register(t0);RWTexture2D<float4> Output:register(u0);
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Input.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 float4 c=Input.Load(int3(id.xy,0));float3 l=pow(saturate(c.rgb),2.2);
 l=saturate(l*(2.51*l+0.03)/(l*(2.43*l+0.59)+0.14));
 Output[id.xy]=float4(pow(l,1.0/2.2),c.a);
}
