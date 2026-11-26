// Screen-space estimated relief / horizon-style ambient shading.
// The external capture provides NO depth buffer. This is color-only edge
// inference, not actual game-engine SSAO/HBAO, occluded geometry or GI.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Output:register(u0);
float luma(float3 c){return dot(c,float3(0.2126,0.7152,0.0722));}
float3 at(int2 p,int2 lim){return Source.Load(int3(clamp(p,int2(0,0),lim),0)).rgb;}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Output.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),limit=int2(w-1,h-1);
 float4 base=Source.Load(int3(p,0));
 float3 c=base.rgb;
 float center=luma(c);
 // Estimated "height": local contrast and luminance, never geometry depth.
 // Multi-direction horizon search, conservative low-frequency distances.
 int2 direction[8]={int2(1,0),int2(-1,0),int2(0,1),int2(0,-1),
                    int2(1,1),int2(-1,1),int2(1,-1),int2(-1,-1)};
 float occlusion=0.0;
 [unroll]for(int axis=0;axis<8;axis++){
  float horizon=0.0;
  [unroll]for(int step=1;step<=4;step++){
   int reach=step*step+1;
   float3 n=at(p+direction[axis]*reach,limit);
   float contrast=luma(n)-center;
   float chroma=length(n-c);
   // Visible high-contrast ridge only. Untextured sky gets no fake depth.
   float raised=max(0.0,contrast*0.60+chroma*0.15-0.045);
   horizon=max(horizon,saturate(raised*(3.0/float(step+1))));
  }
  occlusion+=horizon;
 }
 float shade=1.0-saturate(occlusion/8.0)*0.40;
 Output[id.xy]=float4(saturate(c*shade),base.a);
}
