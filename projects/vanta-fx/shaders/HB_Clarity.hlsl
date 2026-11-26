// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Local Clarity
//!DEFAULT 0.52
//!MIN 0
//!MAX 1.5
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
    float3 blur=(INPUT.SampleLevel(sam,pos+float2(p.x,0),0).rgb
        +INPUT.SampleLevel(sam,pos-float2(p.x,0),0).rgb
        +INPUT.SampleLevel(sam,pos+float2(0,p.y),0).rgb
        +INPUT.SampleLevel(sam,pos-float2(0,p.y),0).rgb)*0.25;
    float3 high=src.rgb-blur;
    float detail=abs(dot(high,float3(0.2126,0.7152,0.0722)));
    float mask=saturate(detail*14.0);
    float3 outc=saturate(src.rgb+clamp(high,-0.15,0.15)*strength*mask);
    return float4(outc,src.a);
}
