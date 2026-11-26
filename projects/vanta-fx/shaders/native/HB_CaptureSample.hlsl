Texture2D<float4> Source:register(t0);
RWTexture2D<uint4> Result:register(u0);
[numthreads(1,1,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Source.GetDimensions(w,h);uint maximum=0,minimum=255,total=0,hash=2166136261u;
 [loop]for(uint y=0;y<16;y++){[loop]for(uint x=0;x<16;x++){
  uint2 p=min(uint2((x+.5)*w/16.0,(y+.5)*h/16.0),uint2(w-1,h-1));
  uint3 c=(uint3)round(saturate(Source.Load(int3(p,0)).rgb)*255);
  uint v=max(c.r,max(c.g,c.b));maximum=max(maximum,v);minimum=min(minimum,v);total+=v;
  hash=(hash^((c.r<<16)|(c.g<<8)|c.b))*16777619u;
 }}Result[uint2(0,0)]=uint4(minimum,maximum,total,hash);
}