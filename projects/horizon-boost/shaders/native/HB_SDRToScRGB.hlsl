// SDR capture to scRGB linear-light FP16 for user-opt-in HDR-capable
// Horizon-owned swapchain. Never invents HDR detail or game luminance.
Texture2D<float4> Input:register(t0);
RWTexture2D<float4> Output:register(u0);
cbuffer HdrCalibration:register(b0){float WhiteScale;float Curve;float PeakScale;float Exposure;};
float linearize(float c){
 return c<=.04045?c/12.92:pow((c+.055)/1.055,2.4);
}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Input.GetDimensions(w,h);
 if(id.x>=w||id.y>=h)return;
 float4 c=Input.Load(int3(id.xy,0));
 // scRGB SDR reference white 80 nits, typical SDR content 100 nits;
 // converting known SDR EOTF yields a correctly encoded FP16 transport.
 float3 rgb=float3(linearize(saturate(c.r)),linearize(saturate(c.g)),linearize(saturate(c.b)));
 if(Curve>0.5){
  float y=dot(rgb,float3(.2126,.7152,.0722));
  float v=y*Exposure;
  float mapped=Curve<1.5 ? v/(1+v) : saturate((v*(2.51*v+.03))/(v*(2.43*v+.59)+.14));
  float norm=Curve<1.5 ? Exposure/(1+Exposure) : saturate((Exposure*(2.51*Exposure+.03))/(Exposure*(2.43*Exposure+.59)+.14));
  rgb*=mapped/max(1e-6,norm*y);
 }
 rgb*=WhiteScale;
 if(Curve>0.5){float high=max(rgb.r,max(rgb.g,rgb.b));rgb*=min(1.,PeakScale/max(high,1e-6));}
 Output[id.xy]=float4(rgb,c.a);
}
