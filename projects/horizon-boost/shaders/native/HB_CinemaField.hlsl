Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Field:register(u0);
cbuffer Settings:register(b0){float4 Light;float4 Art;float4 Rect;float4 Extra;}
float3 lin(float3 c){return lerp(c/12.92,pow(max((c+.055)/1.055,0),2.4),step(.04045,c));}
float lum(float3 c){return dot(c,float3(.2126,.7152,.0722));}
bool protectedPixel(float2 uv){return Extra.x>.5&&all(uv>=Rect.xy)&&all(uv<=Rect.zw);}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h,fw,fh;Source.GetDimensions(w,h);Field.GetDimensions(fw,fh);if(id.x>=fw||id.y>=fh)return;
 uint d=(uint)Extra.y;float3 sum=0;float n=0;
 [loop]for(uint y=0;y<d;y++){[loop]for(uint x=0;x<d;x++){
  uint2 p=id.xy*d+uint2(x,y);if(p.x>=w||p.y>=h)continue;
  if(protectedPixel((float2(p)+.5)/float2(w,h)))continue;
  sum+=lin(saturate(Source.Load(int3(p,0)).rgb));n+=1;
 }}
 float3 c=sum/max(n,1.);Field[id.xy]=float4(c,n>0?1.:0.);
}