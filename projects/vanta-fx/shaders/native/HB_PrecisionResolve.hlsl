Texture2D<float4> InTex:register(t0);
Texture2D<float4> Original:register(t1);
RWTexture2D<float4> OutTex:register(u0);
cbuffer Params:register(b0){float exposure;float shoulder;float contrast;float saturation;float mixStrength;float quant10;float dither;float pad;}
float3 srgb(float3 x){x=max(x,0);return lerp(x*12.92,1.055*pow(x,1/2.4)-.055,step(.0031308,x));}
float bayer(uint2 p){uint x=p.x&3,y=p.y&3;uint a=((x&1)^(y&1))*2+(y&1);uint b=(((x>>1)^(y>>1))&1)*2+(y>>1);return (4*a+b+.5)/16.-.5;}
[numthreads(8,8,1)] void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;OutTex.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 float4 o=Original.Load(int3(id.xy,0));float3 q=srgb(InTex.Load(int3(id.xy,0)).rgb);
 q=saturate(lerp(o.rgb,q,mixStrength));
 if(quant10>.5){q=round(q*1023)/1023;}
 q=saturate(q+bayer(id.xy)*dither/255.);
 OutTex[id.xy]=float4(q,o.a);
}
