// HORIZON BOOST original, non-injected live GPU filter.
//!MAGPIE EFFECT
//!VERSION 4

//!PARAMETER
//!LABEL Adaptive Vibrance
//!DEFAULT 0.44
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
    float high=max(c.r,max(c.g,c.b));
    float low=min(c.r,min(c.g,c.b));
    float saturation=high-low;
    float y=dot(c,float3(0.2126,0.7152,0.0722));
    float adaptive=saturate(1.0-saturation*1.2)*saturate(1.5-abs(y-0.5)*1.8);
    c=saturate(lerp(float3(y,y,y),c,1.0+strength*adaptive));
    return float4(c,src.a);
}
