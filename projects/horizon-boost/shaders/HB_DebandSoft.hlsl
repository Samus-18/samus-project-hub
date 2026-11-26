// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Smart Deband
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
    float2 p=GetInputPt()*2.0;
    float3 a=INPUT.SampleLevel(sam,pos+float2(p.x,p.y),0).rgb;
    float3 b=INPUT.SampleLevel(sam,pos-float2(p.x,p.y),0).rgb;
    float3 avg=(a+b+src.rgb)*0.3333333;
    float diff=max(abs(src.r-avg.r),max(abs(src.g-avg.g),abs(src.b-avg.b)));
    float mask=1.0-smoothstep(0.004,0.065,diff);
    return float4(lerp(src.rgb,avg,strength*mask),src.a);
}
