// HORIZON BOOST ORIGINAL SPATIAL RECONSTRUCTION. No Magpie dependency.
// The source is a finished SDR frame from Windows Graphics Capture.
// Scales OUTSIDE the game process and never changes game render resolution.
// SCALE_MODE 0 bilinear; 1 Catmull-Rom; 2 Lanczos2; 3 edge-aware;
 // 4 nearest; 5 Mitchell-Netravali (soft); 6 bounded edge clarity (sharp).
 // Original algorithms; not branded as AMD FSR1 or NVIDIA NIS.
#ifndef SCALE_MODE
#define SCALE_MODE 0
#endif
Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float4 texel(int2 i,int2 m){return Source.Load(int3(clamp(i,int2(0,0),m),0));}
float catmull(float x){
 x=abs(x);
 if(x<1.0) return 1.0-2.5*x*x+1.5*x*x*x;
 if(x<2.0) return 2.0-4.0*x+2.5*x*x-0.5*x*x*x;
 return 0.0;
}
float mitchell(float v){
 const float B=1.0/3.0,C=1.0/3.0;
 float x=abs(v),x2=x*x,x3=x2*x;
 if(x<1.0){
    return ((12.0-9.0*B-6.0*C)*x3+(-18.0+12.0*B+6.0*C)*x2+
        (6.0-2.0*B))/6.0;
 }
 if(x<2.0){
    return ((-B-6.0*C)*x3+(6.0*B+30.0*C)*x2+
        (-12.0*B-48.0*C)*x+(8.0*B+24.0*C))/6.0;
 }
 return 0.0;
}
float sinc(float x){
 const float pi=3.14159265359;
 float v=pi*x;
 return abs(v)<0.0001?1.0:sin(v)/v;
}
float lanczos(float x){
 x=abs(x);
 return x<2.0?sinc(x)*sinc(x*0.5):0.0;
}
float lanczos3(float x){
 x=abs(x);
 return x<3.0?sinc(x)*sinc(x/3.0):0.0;
}
float4 bilinear_sample(float2 p,int2 lim){
 int2 a=int2(floor(p));float2 f=saturate(p-float2(a));
 float4 v00=texel(a,lim),v10=texel(a+int2(1,0),lim);
 float4 v01=texel(a+int2(0,1),lim),v11=texel(a+int2(1,1),lim);
 return lerp(lerp(v00,v10,f.x),lerp(v01,v11,f.x),f.y);
}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint iw,ih,ow,oh;
 Source.GetDimensions(iw,ih);
 Target.GetDimensions(ow,oh);
 if(id.x>=ow||id.y>=oh) return;
 int2 lim=int2(iw-1,ih-1);
 float2 uv=(float2(id.xy)+0.5)/float2(ow,oh);
 float2 xy=uv*float2(iw,ih)-0.5;
 float4 result=float4(0.0,0.0,0.0,1.0);
#if SCALE_MODE == 9
 uint factor=min(ow/iw,oh/ih);
 if(factor==0){result=texel(int2(round(xy)),lim);}
 else{
  uint2 dims=uint2(iw,ih)*factor;
  uint2 offset=(uint2(ow,oh)-dims)/2;
  if(all(id.xy>=offset)&&all(id.xy-offset<dims)){result=Source.Load(int3((id.xy-offset)/factor,0));}
 }
#elif SCALE_MODE == 4
 result=texel(int2(round(xy)),lim);
#elif SCALE_MODE == 8
 // Area integral for a source footprint of at most 2x2 pixels.
 // Fractional offsets may overlap three texels on either axis.
 float2 ratio=float2(iw,ih)/float2(ow,oh);
 if(any(ratio>2.001)||any(ratio<1.0)){
  result=bilinear_sample(xy,lim);
 }else{
  float2 lo=float2(id.xy)*ratio;
  float2 hi=lo+ratio;
  int2 start=int2(floor(lo));
  float4 sum=0;float total=0;
  [unroll]for(int yy=0;yy<3;yy++){
   [unroll]for(int xx=0;xx<3;xx++){
    int2 p=start+int2(xx,yy);
    float2 overlap=max(0.0,min(hi,float2(p)+1.0)-max(lo,float2(p)));
    float w=overlap.x*overlap.y;
    sum+=texel(p,lim)*w;total+=w;
   }
  }
  result=sum/max(total,0.000001);
 }
#elif SCALE_MODE == 0
 result=bilinear_sample(xy,lim);
#elif SCALE_MODE == 1 || SCALE_MODE == 2 || SCALE_MODE == 5
 float4 sum=0.0;float total=0.0;
 int2 origin=int2(floor(xy));
 [unroll]for(int y=-1;y<=2;y++){
  [unroll]for(int x=-1;x<=2;x++){
#if SCALE_MODE == 1
    float weight=catmull(xy.x-(origin.x+x))*catmull(xy.y-(origin.y+y));
#elif SCALE_MODE == 5
    float weight=mitchell(xy.x-(origin.x+x))*mitchell(xy.y-(origin.y+y));
#else
    float weight=lanczos(xy.x-(origin.x+x))*lanczos(xy.y-(origin.y+y));
#endif
    sum+=texel(origin+int2(x,y),lim)*weight;
    total+=weight;
  }
 }
 result=sum/max(total,0.00001);
#elif SCALE_MODE == 7
 // Premium quality-only Lanczos3; 6x6 pixel neighborhood, higher GPU cost.
 float4 sum=0.0;float total=0.0;
 int2 origin=int2(floor(xy));
 [unroll]for(int y=-2;y<=3;y++){
  [unroll]for(int x=-2;x<=3;x++){
    float weight=lanczos3(xy.x-(origin.x+x))*
                 lanczos3(xy.y-(origin.y+y));
    sum+=texel(origin+int2(x,y),lim)*weight;
    total+=weight;
  }
 }
 float4 normalized=sum/max(abs(total),0.00001);
 // Limit near-neighbor overshoot to avoid offensive ringing on UI text.
 float3 a=texel(origin,lim).rgb;
 float3 b=texel(origin+int2(1,0),lim).rgb;
 float3 c=texel(origin+int2(0,1),lim).rgb;
 float3 d=texel(origin+int2(1,1),lim).rgb;
 float3 low=min(min(a,b),min(c,d));
 float3 high=max(max(a,b),max(c,d));
 result=float4(clamp(normalized.rgb,low-0.035,high+0.035),normalized.a);
#elif SCALE_MODE == 3
 float4 normal=bilinear_sample(xy,lim);
 float4 xx=bilinear_sample(xy+float2(-1,0),lim);
 float4 yy=bilinear_sample(xy+float2( 1,0),lim);
 float4 up=bilinear_sample(xy+float2(0,-1),lim);
 float4 dn=bilinear_sample(xy+float2(0, 1),lim);
 float3 edge=normal.rgb-(xx.rgb+yy.rgb+up.rgb+dn.rgb)*0.25;
 float energy=dot(abs(edge),float3(0.2126,0.7152,0.0722));
 float limit=saturate((energy-0.018)*7.0);
 float3 cleaned=normal.rgb+clamp(edge,-0.035,0.035)*(0.16+0.50*limit);
 result=float4(cleaned,normal.a);
#elif SCALE_MODE == 6
 float4 center=bilinear_sample(xy,lim);
 float3 n=bilinear_sample(xy+float2(0,-1),lim).rgb;
 float3 so=bilinear_sample(xy+float2(0,1),lim).rgb;
 float3 e=bilinear_sample(xy+float2(1,0),lim).rgb;
 float3 w=bilinear_sample(xy+float2(-1,0),lim).rgb;
 float3 localMin=min(center.rgb,min(min(n,so),min(e,w)));
 float3 localMax=max(center.rgb,max(max(n,so),max(e,w)));
 float3 blur=(n+so+e+w)*0.25;
 float3 edge=center.rgb-blur;
 float contrast=dot(localMax-localMin,float3(0.2126,0.7152,0.0722));
 float strength=saturate((contrast-0.025)*3.8);
 float3 sharpen=center.rgb+clamp(edge,-0.058,0.058)*(0.16+0.58*strength);
 // Never create new bright and dark halos outside local neighborhood.
 result=float4(clamp(sharpen,localMin-0.018,localMax+0.018),center.a);
#endif
 Target[id.xy]=float4(saturate(result.rgb),saturate(result.a));
}
