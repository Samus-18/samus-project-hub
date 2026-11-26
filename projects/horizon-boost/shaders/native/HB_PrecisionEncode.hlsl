Texture2D<float4> InTex:register(t0);
RWTexture2D<float4> OutTex:register(u0);
cbuffer Params:register(b0){float exposure;float shoulder;float contrast;float saturation;float mixStrength;float quant10;float dither;float pad;}
float3 linearize(float3 x){return lerp(x/12.92,pow(max((x+.055)/1.055,0),2.4),step(.04045,x));}
[numthreads(8,8,1)] void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;OutTex.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 float4 p=InTex.Load(int3(id.xy,0));float3 q=linearize(saturate(p.rgb));
 q*=exp2(exposure);float y=dot(q,float3(.2126,.7152,.0722));
 q=lerp(y.xxx,q,saturation);q=max(0,(q-.18)*contrast+.18);
 q=lerp(q,q/(1+max(q-0.18,0)*shoulder),saturate(shoulder));
 OutTex[id.xy]=float4(q,p.a);
}
