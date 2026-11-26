// Horizon Boost: native tiled LDS cache for original soft, alpha-safe RGB denoise.
// Workgroup loads shared 10x10 halo for 8x8 pixels. No CPU readback.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Output:register(u0);
groupshared float4 Cached[100];
[numthreads(8,8,1)]
void CSMain(uint3 gid:SV_GroupID,uint3 lid:SV_GroupThreadID,uint3 id:SV_DispatchThreadID){
 uint w,h;Source.GetDimensions(w,h);
 uint lane=lid.y*8+lid.x;
 [loop]for(uint t=lane;t<100;t+=64){
  int2 pixel=int2(gid.xy*8)+int2(t%10,t/10)-int2(1,1);
  pixel=clamp(pixel,int2(0,0),int2(w-1,h-1));
  Cached[t]=Source.Load(int3(pixel,0));
 }
 GroupMemoryBarrierWithGroupSync();
 if(id.x>=w||id.y>=h)return;
 uint base=(lid.y+1)*10+lid.x+1;
 float3 blur=0;
 [unroll]for(int y=-1;y<=1;y++){
  [unroll]for(int x=-1;x<=1;x++){
   float weight=(2-abs(x))*(2-abs(y));
   blur+=Cached[base+y*10+x].rgb*(weight/16.);
  }
 }
 float4 center=Cached[base];
 Output[id.xy]=float4(saturate(lerp(center.rgb,blur,0.65)),center.a);
}
