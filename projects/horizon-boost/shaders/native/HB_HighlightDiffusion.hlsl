Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 px(int2 p,int2 s){return Source.Load(int3(clamp(p,int2(0,0),s-1),0)).rgb;}
float luma(float3 c){return dot(c,float3(.2126,.7152,.0722));}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),s=int2(w,h);float3 c=px(p,s);
float3 sum=0;
 [unroll]for(int j=-2;j<=2;j++){[unroll]for(int i=-2;i<=2;i++){
 float3 v=px(p+int2(i,j)*2,s);sum+=v*smoothstep(.55,.95,luma(v));}}
 Target[id.xy]=float4(saturate(c+sum*.012),1);}
