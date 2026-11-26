Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
cbuffer GrainParams:register(b0){uint GrainPhase;uint3 GrainPadding;};
float3 px(int2 p,int2 s){return Source.Load(int3(clamp(p,int2(0,0),s-1),0)).rgb;}
float lum(float3 c){return dot(c,float3(.2126,.7152,.0722));}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),s=int2(w,h);float3 c=px(p,s);
uint hash=id.x*1973u+id.y*9277u+89173u+GrainPhase*747796405u;hash=(hash^(hash>>16))*2246822519u;hash^=hash>>13;
 float noise=(float(hash&1023u)/1023.0-.5)*.032;float amount=.4+.6*(1-abs(lum(c)*2-1));
 Target[id.xy]=float4(saturate(c+noise*amount),Source.Load(int3(p,0)).a);}
