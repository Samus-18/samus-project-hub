// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Film Grain
//!DEFAULT 0.17
//!MIN 0
//!MAX 0.7
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
    float2 pixel=pos/GetInputPt();
    float noise=frac(sin(dot(floor(pixel),float2(12.9898,78.233)))*43758.5453)-0.5;
    float y=dot(src.rgb,float3(0.2126,0.7152,0.0722));
    float amount=strength*(0.5+0.5*saturate(1.0-y))*0.16;
    return float4(saturate(src.rgb+noise*amount),src.a);
}
