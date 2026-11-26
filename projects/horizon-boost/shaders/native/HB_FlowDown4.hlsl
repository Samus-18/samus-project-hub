// Horizon Boost GPU-only quarter-resolution frame pyramid. Off by default.
Texture2D<float4> Previous:register(t0);
Texture2D<float4> Current:register(t1);
RWTexture2D<float4> OldLow:register(u0);
RWTexture2D<float4> NewLow:register(u1);
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint lw,lh,w,h;OldLow.GetDimensions(lw,lh);Previous.GetDimensions(w,h);
 if(id.x>=lw||id.y>=lh)return;
 uint2 base=id.xy*4;float4 a=0,b=0;
 [unroll]for(uint y=0;y<4;y++){[unroll]for(uint x=0;x<4;x++){
  int3 p=int3(min(base+uint2(x,y),uint2(w-1,h-1)),0);
  a+=Previous.Load(p);b+=Current.Load(p);
 }}
 OldLow[id.xy]=a*(1.0/16.0);NewLow[id.xy]=b*(1.0/16.0);
}
