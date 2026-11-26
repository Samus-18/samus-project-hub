// Coarse bidirectional flow on 1/4 images, output motion in full pixels.
#ifndef FLOW_TILE
#define FLOW_TILE 16
#endif
Texture2D<float4> Previous:register(t0);
Texture2D<float4> Current:register(t1);
RWTexture2D<float4> Seed:register(u0);
float3 pix(Texture2D<float4> tex,int2 p,int2 cap){return tex.Load(int3(clamp(p,int2(0,0),cap),0)).rgb;}
float score(Texture2D<float4> a,Texture2D<float4> b,int2 at,int2 shift,int2 cap){
 int2 taps[5]={int2(0,0),int2(-2,0),int2(2,0),int2(0,-2),int2(0,2)};
 float v=0;[unroll]for(int i=0;i<5;i++){
  float3 x=pix(a,at+taps[i],cap);float3 y=pix(b,at+taps[i]+shift,cap);
  v+=abs(dot(x-y,float3(.2126,.7152,.0722)))+.07*length(x-y);
 }return v+0.0015*length(float2(shift));
}
float2 choose(Texture2D<float4> a,Texture2D<float4> b,int2 center,int2 cap){
 int2 opts[13]={int2(0,0),int2(4,0),int2(-4,0),int2(0,4),int2(0,-4),
 int2(8,0),int2(-8,0),int2(0,8),int2(0,-8),
 int2(4,4),int2(4,-4),int2(-4,4),int2(-4,-4)};
 float best=1e30;int2 motion=0;[unroll]for(int i=0;i<13;i++){
  float c=score(a,b,center,opts[i],cap);
  if(c<best){best=c;motion=opts[i];}
 }return float2(motion)*4.0;
}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint fw,fh;Seed.GetDimensions(fw,fh);if(id.x>=fw||id.y>=fh)return;
 uint sw,sh;Previous.GetDimensions(sw,sh);
 int2 cap=int2(sw-1,sh-1);
 int2 center=int2(id.xy)*(FLOW_TILE/4)+int2(FLOW_TILE/8,FLOW_TILE/8);
 Seed[id.xy]=float4(choose(Previous,Current,center,cap),choose(Current,Previous,center,cap));
}
