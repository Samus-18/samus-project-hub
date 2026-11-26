Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
cbuffer Grade:register(b0){float4 A;float4 B;float4 C;float4 D;float4 E;float4 Curves[5];float4 Gradient[3];};
float curve(float x,uint channel){float q=saturate(x)*4;uint i=min(3u,(uint)floor(q));return lerp(Curves[i][channel],Curves[i+1][channel],q-i);}
float3 toLinear(float3 x){return lerp(x/12.92,pow(max((x+.055)/1.055,0),2.4),step(.04045,x));}
float3 toSrgb(float3 x){x=max(x,0);return lerp(x*12.92,1.055*pow(x,1./2.4)-.055,step(.0031308,x));}
float luma(float3 x){return dot(x,float3(.2126,.7152,.0722));}
float3 rgbToHsv(float3 c){
 float4 K=float4(0,-1./3.,2./3.,-1);
 float4 p=lerp(float4(c.bg,K.wz),float4(c.gb,K.xy),step(c.b,c.g));
 float4 q=lerp(float4(p.xyw,c.r),float4(c.r,p.yzx),step(p.x,c.r));
 float d=q.x-min(q.w,q.y);
 return float3(abs(q.z+(q.w-q.y)/(6*d+1e-10)),d/(q.x+1e-10),q.x);
}
float3 hsvToRgb(float3 c){float3 p=abs(frac(c.xxx+float3(0,2./3.,1./3.))*6-3);return c.z*lerp(1.,saturate(p-1),c.y);}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 float3 original=Source.Load(int3(id.xy,0)).rgb;
 if(E.x<=0){Target[id.xy]=float4(original,1);return;}
 float3 c=E.y>.5?toLinear(original):original;
 c=max(0,c*exp2(A.x)+A.y*(1-saturate(c)));
 c=pow(c,1./max(.5,A.z))*A.w;
 c*=float3(1+.18*B.x-.06*B.y,1+.10*B.y,1-.18*B.x-.06*B.y);
 float y=luma(c);c=lerp(y.xxx,c,B.z);
 float pivot=E.y>.5?.18:.5;c=(c-pivot)*B.w+pivot;
 c=E.y>.5?toSrgb(max(c,0)):c;
 // Endpoint-preserving midpoint adjustment, RGB channels and luminance.
 c=saturate(c);c=saturate(c+4*c*(1-c)*C.xyz);
 y=luma(c);c=saturate(c+4*y*(1-y)*C.w);
 c=float3(curve(c.r,0),curve(c.g,1),curve(c.b,2));y=luma(c);c=saturate(c+curve(y,3)-y);
 if(abs(D.z)>1e-5||abs(D.w-1)>1e-5){
  float3 hsv=rgbToHsv(c);float hd=abs(hsv.x-D.x);hd=min(hd,1-hd);
  float mask=(1-smoothstep(D.y*.4,D.y,hd))*smoothstep(.02,.18,hsv.y);
  hsv.x=frac(hsv.x+D.z*mask+1);hsv.y=saturate(hsv.y*lerp(1,D.w,mask));c=hsvToRgb(hsv);
 }
 if(E.z>0){
  float lum=saturate(luma(c));float mid=clamp(E.w,.05,.95);
  float3 mapped=lum<=mid?lerp(Gradient[0].rgb,Gradient[1].rgb,lum/mid):lerp(Gradient[1].rgb,Gradient[2].rgb,(lum-mid)/(1-mid));
  c=lerp(c,mapped,saturate(E.z));
 }
 Target[id.xy]=float4(saturate(lerp(original,c,E.x)),1);
}