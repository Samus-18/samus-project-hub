Texture2D<float4> Src:register(t0);
RWTexture2D<float4> OutTex:register(u0);
float luma(float3 c){return dot(c,float3(.2126,.7152,.0722));}
float l(int2 p,int2 lim){return luma(Src.Load(int3(clamp(p,0,lim),0)).rgb);}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Src.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),lim=int2(w-1,h-1);float4 c=Src.Load(int3(p,0));float y=luma(c.rgb);
 // Multi-scale luminance relief: dark local concavities with support in opposite directions.
 const int2 axes[4]={int2(1,0),int2(0,1),int2(1,1),int2(1,-1)};
 float occ=0;
 [unroll]for(int k=0;k<4;k++){
  int2 d=axes[k];float a=l(p+d,lim)-y,b=l(p-d,lim)-y;
  float near=saturate((min(a,b)-.015)*5);
  float far=saturate((min(l(p+d*4,lim),l(p-d*4,lim))-y-.03)*3);
  occ+=max(near,far*.65);
 }
 float shadow=1-.30*saturate(occ*.25);
 OutTex[id.xy]=float4(c.rgb*shadow,c.a);
}
