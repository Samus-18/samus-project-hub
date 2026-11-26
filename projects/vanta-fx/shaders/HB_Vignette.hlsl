// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Lens Vignette
//!DEFAULT 0.38
//!MIN 0
//!MAX 1
//!STEP 0.01
float strength;

//!PARAMETER
//!LABEL Lens Radius
//!DEFAULT 0.62
//!MIN 0.1
//!MAX 1
//!STEP 0.01
float radius;

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
    float2 d=(pos-0.5)*float2(1.0,0.7);
    float dist=length(d);
    float v=1.0-strength*smoothstep(radius*0.45,max(radius,0.2),dist);
    return float4(src.rgb*v,src.a);
}
