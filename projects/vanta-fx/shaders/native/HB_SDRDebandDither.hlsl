// Original Horizon Boost optional ordered blue-leaning SDR debanding.
// Output is 8-bit RGBA UNORM. Does NOT claim native HDR / 10-bit display.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
static const float Bayer[16]={
 0.0, 8.0, 2.0,10.0,
12.0, 4.0,14.0, 6.0,
 3.0,11.0, 1.0, 9.0,
15.0, 7.0,13.0, 5.0
};
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
    uint w,h;Target.GetDimensions(w,h);
    if(id.x>=w||id.y>=h)return;
    float4 base=Source.Load(int3(id.xy,0));
    uint idx=(id.x&3)+((id.y&3)<<2);
    float amplitude=(Bayer[idx]-7.5)/16.0*1.55/255.0;
    // Use low-amplitude channel decorrelation to suppress visible grids.
    float3 dither=float3(amplitude,
      -amplitude*0.73,
      amplitude*0.61);
    Target[id.xy]=float4(saturate(base.rgb+dither),base.a);
}