// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Sharpen Plus
//!DEFAULT 0.43
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
    float3 a=INPUT.SampleLevel(sam,pos+float2(p.x,0),0).rgb;
    float3 b=INPUT.SampleLevel(sam,pos-float2(p.x,0),0).rgb;
    float3 c=INPUT.SampleLevel(sam,pos+float2(0,p.y),0).rgb;
    float3 d=INPUT.SampleLevel(sam,pos-float2(0,p.y),0).rgb;
    float3 avg=(a+b+c+d)*0.25;
    float localContrast=max(max(length(src.rgb-a),length(src.rgb-b)),max(length(src.rgb-c),length(src.rgb-d)));
    float suppress=saturate(localContrast*4.0);
    float3 delta=clamp(src.rgb-avg,-0.12,0.12);
    return float4(saturate(src.rgb+delta*strength*(0.4+0.6*suppress)*2.0),src.a);
}
