// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Soft Bloom
//!DEFAULT 0.42
//!MIN 0
//!MAX 1
//!STEP 0.01
float strength;

//!PARAMETER
//!LABEL Highlight Threshold
//!DEFAULT 0.68
//!MIN 0.3
//!MAX 1
//!STEP 0.01
float threshold;

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
    float2 p=GetInputPt()*3.0;
    float3 a=INPUT.SampleLevel(sam,pos+float2(p.x,0),0).rgb;
    float3 b=INPUT.SampleLevel(sam,pos-float2(p.x,0),0).rgb;
    float3 c=INPUT.SampleLevel(sam,pos+float2(0,p.y),0).rgb;
    float3 d=INPUT.SampleLevel(sam,pos-float2(0,p.y),0).rgb;
    float3 avg=(a+b+c+d)*0.25;
    float l=dot(avg,float3(0.2126,0.7152,0.0722));
    float m=smoothstep(threshold,1.0,l);
    return float4(saturate(src.rgb+avg*m*strength*0.35),src.a);
}
