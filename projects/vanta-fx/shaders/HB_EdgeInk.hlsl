// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Ink Outlines
//!DEFAULT 0.45
//!MIN 0
//!MAX 1
//!STEP 0.01
float strength;

//!TEXTURE
Texture2D INPUT;
//!TEXTURE
//!WIDTH INPUT_WIDTH
//!HEIGHT INPUT_HEIGHT
Texture2D OUTPUT;
//!SAMPLER
//!FILTER LINEAR
//!ADDRESS CLAMP
SamplerState sam;

//!PASS 1
//!STYLE PS
//!IN INPUT
//!OUT OUTPUT
float4 Pass1(float2 pos) {
    float4 src=INPUT.SampleLevel(sam,pos,0);
    float2 p=GetInputPt();
    float3 dx=INPUT.SampleLevel(sam,pos+float2(p.x,0),0).rgb-INPUT.SampleLevel(sam,pos-float2(p.x,0),0).rgb;
    float3 dy=INPUT.SampleLevel(sam,pos+float2(0,p.y),0).rgb-INPUT.SampleLevel(sam,pos-float2(0,p.y),0).rgb;
    float edge=saturate((length(dx)+length(dy))*2.2);
    float3 poster=floor(src.rgb*8.0+0.5)/8.0;
    float3 ink=saturate(lerp(src.rgb,poster,strength*0.5)*(1.0-strength*edge*0.7));
    return float4(ink,src.a);
}
