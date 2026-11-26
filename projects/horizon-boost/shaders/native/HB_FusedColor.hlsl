// Compatible original Horizon color equations; all operations are pixel-local.
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
cbuffer Params:register(b0){uint4 Ops0;uint4 Ops1;uint4 Control;};
float3 apply_color(float3 c,uint op){
 float l=dot(c,float3(.2126,.7152,.0722));
 switch(op){
 case 0:{float range=max(c.r,max(c.g,c.b))-min(c.r,min(c.g,c.b));float amount=.75*(1.-saturate(range));return saturate(lerp(float3(l,l,l),c,1.+amount));}
 case 1:{float3 x=max(c*1.26+.012,0.);float3 v=saturate((x*(2.51*x+.03))/(x*(2.43*x+.59)+.14));float lum=dot(v,float3(.2126,.7152,.0722));float s=(1.-smoothstep(.15,.6,lum))*.055;float h=smoothstep(.45,.9,lum)*.045;return saturate(v+float3(-s+h,.2*s+.25*h,s-h));}
 case 3:{float gain=saturate(sqrt(l)*1.2+.06);return float3(gain*.13,gain,gain*.2);}
 case 4:{float shadow=1.-smoothstep(.06,.58,l);return saturate(c+shadow*.30*float3(.82,.94,1.08));}
 case 5:return saturate(float3(c.r*1.12+.035*l,c.g*.98+.012*l,c.b*.83));
 case 6:return saturate(float3(c.r*.79,c.g*.98+.035*l,c.b*1.17+.055*l));
 case 7:{float x=saturate((l-.5)*1.37+.5);return float3(x,x,x);}
 case 8:return saturate(float3(dot(c,float3(.393,.769,.189)),dot(c,float3(.349,.686,.168)),dot(c,float3(.272,.534,.131))));
 case 9:return saturate(float3(c.r*.82+c.g*.18,c.r*.10+c.g*.87+c.b*.03,c.b*.91+c.g*.09));
 case 11:{float3 v=lerp(float3(l,l,l),c,1.18)+float3(-.018,.009,.022);return saturate((v-.5)*1.08+.5);}
 default:return c;
 }
}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 float4 src=Source.Load(int3(id.xy,0));float3 c=src.rgb;
 [loop]for(uint i=0;i<8u;i++){
  if(i>=Control.x)break;
  uint lane=i&3u;
  uint op=Ops0[lane];
  if(i>=4u)op=Ops1[lane];
  // Match each separate RGBA8_UNORM intermediate, not merely the final write.
  c=round(saturate(apply_color(c,op))*255.)/255.;
 }
 Target[id.xy]=float4(c,src.a);
}
