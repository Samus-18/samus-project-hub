Texture2D<float4> Source:register(t0);
Texture2D<float4> Field:register(t1);
Texture2D<float4> LightField:register(t2);
RWTexture2D<float4> Target:register(u0);
cbuffer Settings:register(b0){float4 Light;float4 Art;float4 Rect;float4 Extra;}
float3 lin(float3 c){return lerp(c/12.92,pow(max((c+.055)/1.055,0),2.4),step(.04045,c));}
float3 srgb(float3 c){c=max(c,0);return lerp(c*12.92,1.055*pow(c,1./2.4)-.055,step(.0031308,c));}
float lum(float3 c){return dot(c,float3(.2126,.7152,.0722));}
float4 fieldSample(float2 p,int2 lim){
 int2 q=int2(floor(p));float2 f=frac(p);
 float4 a=LightField.Load(int3(clamp(q,0,lim),0));
 float4 b=LightField.Load(int3(clamp(q+int2(1,0),0,lim),0));
 float4 c=LightField.Load(int3(clamp(q+int2(0,1),0,lim),0));
 float4 d=LightField.Load(int3(clamp(q+int2(1,1),0,lim),0));
 return lerp(lerp(a,b,f.x),lerp(c,d,f.x),f.y);
}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h,fw,fh;Target.GetDimensions(w,h);Field.GetDimensions(fw,fh);if(id.x>=w||id.y>=h)return;
 float4 o=Source.Load(int3(id.xy,0));float2 uv=(float2(id.xy)+.5)/float2(w,h);
 if(Art.w<=0||(Extra.x>.5&&all(uv>=Rect.xy)&&all(uv<=Rect.zw))){Target[id.xy]=o;return;}
 float3 base=lin(saturate(o.rgb));float y=lum(base);
 float2 p=(float2(id.xy)+.5)/Extra.y-.5;int2 lim=int2(fw-1,fh-1);
 float4 local=Field.Load(int3(clamp(int2(round(p)),0,lim),0));float4 incoming=fieldSample(p,lim);
 float iy=lum(incoming.rgb),ly=lum(local.rgb);
 // Preserve dark detail and avoid making black letterboxing emit light.
 float gate=smoothstep(.003,.045,y);
 float occlusion=incoming.a*saturate((ly-y+.045)*5.0);
 float3 result=base*(1-Light.x*.55*occlusion);
 float bounce=saturate((iy-y)*2.8)*gate;
 result+=max(incoming.rgb-base*.35,0)*bounce*Light.y*.44;
 float glow=smoothstep(.23,.68,iy);
 result+=incoming.rgb*glow*Light.z*.24*(1-.65*y)*gate;
 // Small bright edge sheen derived exclusively from visible contrast, not world-space reflections.
 float sheen=smoothstep(.12,.5,y)*saturate((y-ly)*3)*saturate(iy*2);
 result+=incoming.rgb*sheen*Light.w*.16;
 // Local exposure responds to nearby luminance rather than a global gain. No adaptive history flicker.
 float ev=clamp(log2((.22+.14)/(.22+ly)),-.38,.38)*Art.y;
 result*=exp2(ev);
 float shoulder=max(lum(result)-.68,0);result/=1+shoulder*Art.z*.7;
 float3 col=saturate(srgb(result));
 Target[id.xy]=float4(lerp(o.rgb,col,Art.w),o.a);
}