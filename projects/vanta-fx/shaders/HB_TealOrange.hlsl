// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Teal and Orange
//!DEFAULT 0.39
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
    float y=dot(src.rgb,float3(0.2126,0.7152,0.0722));
    float shadows=1.0-smoothstep(0.18,0.69,y);
    float highlights=smoothstep(0.34,0.89,y);
    float3 shift=shadows*float3(-0.055,0.03,0.07)+highlights*float3(0.07,0.024,-0.052);
    return float4(saturate(src.rgb+shift*strength*2.0),src.a);
}
