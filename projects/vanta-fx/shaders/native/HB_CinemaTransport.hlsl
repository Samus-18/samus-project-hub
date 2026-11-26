Texture2D<float4> Field:register(t0);
RWTexture2D<float4> LightField:register(u0);
cbuffer Settings:register(b0){float4 Light;float4 Art;float4 Rect;float4 Extra;}
float lum(float3 c){return dot(c,float3(.2126,.7152,.0722));}
float4 get(int2 p,int2 lim){return Field.Load(int3(clamp(p,int2(0,0),lim),0));}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Field.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),lim=int2(w-1,h-1);float4 c=get(p,lim);float cy=lum(c.rgb);
 const int2 dirs[8]={int2(1,0),int2(1,1),int2(0,1),int2(-1,1),int2(-1,0),int2(-1,-1),int2(0,-1),int2(1,-1)};
 float3 sum=0;float weight=0,occ=0;
 int radius=max(1,(int)round((8.+Art.x*32.)/Extra.y));
 // Opposing bright neighbors create an inferred concavity; directional transport attenuates across dark intervening pixels.
 [unroll]for(int i=0;i<8;i++){
  float4 a=get(p+dirs[i]*radius,lim);float4 b=get(p+dirs[(i+4)%8]*radius,lim);
  float4 far=get(p+dirs[i]*radius*3,lim);
  float ay=lum(a.rgb),by=lum(b.rgb),fy=lum(far.rgb);
  float v=a.a*b.a;
  occ+=v*saturate((min(ay,by)-cy-.018)*4.5);
  float vis=saturate(.30+ay*2.8);
  float nearW=.72*a.a,farW=.28*far.a*vis;
  float3 nearL=a.rgb*(.25+.75*saturate(ay*2.));
  float3 farL=far.rgb*(.25+.75*saturate(fy*2.));
  sum+=nearL*nearW+farL*farW;weight+=nearW+farW;
 }
 LightField[id.xy]=float4(sum/max(weight,.001),saturate(occ/8.)*c.a);
}