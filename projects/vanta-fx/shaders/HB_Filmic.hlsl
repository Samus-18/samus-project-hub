// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Filmic Tone
//!DEFAULT 0.54
//!MIN 0
//!MAX 1
//!STEP 0.01
float strength;

//!PARAMETER
//!LABEL Warm Highlights
//!DEFAULT 0.2
//!MIN -1
//!MAX 1
//!STEP 0.01
float warmth;

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
    float3 c=src.rgb;
    float3 curve=saturate((c*(2.51*c+0.03))/(c*(2.43*c+0.59)+0.14));
    float3 warm=float3(1.0+0.05*warmth,1.0,1.0-0.06*warmth);
    float3 toned=saturate(curve*warm);
    float lifted=dot(c,float3(0.2126,0.7152,0.0722));
    toned=lerp(toned,toned*0.94+0.015,saturate((0.45-lifted)*2.0));
    return float4(lerp(c,toned,strength),src.a);
}
