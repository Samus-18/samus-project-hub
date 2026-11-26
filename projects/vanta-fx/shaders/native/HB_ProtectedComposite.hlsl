Texture2D<float4> Original:register(t0);
Texture2D<float4> Processed:register(t1);
RWTexture2D<float4> Target:register(u0);
cbuffer Settings:register(b0){float4 Rect;float4 P;}
float l(int2 p,int2 lim){return dot(Original.Load(int3(clamp(p,0,lim),0)).rgb,float3(.2126,.7152,.0722));}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),lim=int2(w-1,h-1);float4 a=Original.Load(int3(p,0)),b=Processed.Load(int3(p,0));
 float2 uv=(float2(id.xy)+.5)/float2(w,h);
 if(P.x>.5&&all(uv>=Rect.xy)&&all(uv<=Rect.zw)){Target[id.xy]=a;return;}
 float protect=0.;
 if(P.y>0.){
  float c=l(p,lim);float4 d=float4(l(p+int2(1,0),lim),l(p-int2(1,0),lim),l(p+int2(0,1),lim),l(p-int2(0,1),lim));
  float e=max(max(abs(d.x-c),abs(d.y-c)),max(abs(d.z-c),abs(d.w-c)));
  protect=smoothstep(P.z,min(1.,P.z+.15),e)*P.y;
 }
 Target[id.xy]=lerp(b,a,protect);
}