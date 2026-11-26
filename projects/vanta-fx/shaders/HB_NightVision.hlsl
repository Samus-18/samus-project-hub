// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Night Vision
//!DEFAULT 0.72
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
    float lifted=pow(saturate(y),0.62);
    float3 night=float3(lifted*0.38,lifted*1.03,lifted*0.49);
    float scan=1.0-0.03*(0.5+0.5*sin(pos.y/GetInputPt().y*1.6));
    night*=scan;
    return float4(saturate(lerp(src.rgb,night,strength)),src.a);
}
