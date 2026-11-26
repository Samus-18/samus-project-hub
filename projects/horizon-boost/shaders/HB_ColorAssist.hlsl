// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Color Separation
//!DEFAULT 0.48
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
    float3 c=src.rgb;
    float redGreen=c.r-c.g;
    float3 adjust=float3(redGreen*0.055,-redGreen*0.025,redGreen*0.045);
    float3 outlined=saturate(c+adjust*strength*4.0);
    return float4(outlined,src.a);
}
