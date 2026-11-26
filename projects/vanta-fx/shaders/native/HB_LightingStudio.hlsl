// Original external SDR lighting studio - safe color-only approximation.
// Six independent user-controlled GPU weights; NO geometric rays or depth.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
cbuffer LightSettings:register(b0){
 float4 LightingA; // shadow, glow, color bounce, clarity
 float4 LightingB; // film tone, warmth, overall mix, reserved
}
float3 px(int2 p,int2 lim){return Source.Load(int3(clamp(p,int2(0,0),lim),0)).rgb;}
float y(float3 c){return dot(c,float3(0.2126,0.7152,0.0722));}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),lim=int2(w-1,h-1);
 float4 raw=Source.Load(int3(p,0));
 float3 color=raw.rgb;
 float3 nearby=0,light=0;float lum=0;
 int2 o[8]={int2(3,0),int2(-3,0),int2(0,3),int2(0,-3),
  int2(10,8),int2(-10,8),int2(10,-8),int2(-10,-8)};
 [unroll]for(int i=0;i<8;i++){
  float3 v=px(p+o[i],lim);float yy=y(v);
  float weight=(i<4?1.0:0.55);
  nearby+=v*weight;
  lum+=yy*weight;
  light+=v*saturate((yy-0.68)*3.5)*weight;
 }
 nearby/=6.2;lum/=6.2;light/=6.2;
 float center=y(color);
 float shadow=saturate((lum-center-0.02)*3.8);
 float3 tuned=color*(1.0-LightingA.x*shadow*0.36);
 tuned+=(1.0-tuned)*light*LightingA.y*0.44;
 float gain=saturate((0.63-center)*1.7)*LightingA.z*0.24;
 tuned+=max(nearby-tuned,0.0)*gain;
 float3 fine=(px(p+int2(-1,0),lim)+px(p+int2(1,0),lim)
      +px(p+int2(0,-1),lim)+px(p+int2(0,1),lim))*0.25;
 float3 detail=clamp(tuned-fine,-0.08,0.08);
 float detailWeight=saturate(center*7.0)*saturate((1.0-center)*7.0);
 tuned+=detail*LightingA.w*0.68*detailWeight;
 float3 shoulder=tuned/(tuned+0.14)*1.05;
 tuned=lerp(tuned,shoulder,LightingB.x*0.25);
 float highlight=saturate((center-0.38)*2.2);
 tuned+=float3(0.045,-0.004,-0.026)*highlight*LightingB.y;
 float3 result=lerp(color,saturate(tuned),LightingB.z);
 Target[id.xy]=float4(saturate(result),raw.a);
}
