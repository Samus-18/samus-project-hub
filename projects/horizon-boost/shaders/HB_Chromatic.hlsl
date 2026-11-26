// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Chromatic Offset
//!DEFAULT 0.38
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
    float2 centered=(pos-0.5);
    float2 offset=centered*strength*0.006;
    float r=INPUT.SampleLevel(sam,saturate(pos+offset),0).r;
    float b=INPUT.SampleLevel(sam,saturate(pos-offset),0).b;
    return float4(r,src.g,b,src.a);
}
