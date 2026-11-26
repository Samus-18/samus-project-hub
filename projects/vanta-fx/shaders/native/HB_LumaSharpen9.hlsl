// Original Horizon implementation of luminance-only Gaussian unsharp mask.
// Normal 3x3 [1 2 1] kernel, gain .65, bounded +/- .035 SDR.
Texture2D<float4> Input:register(t0);RWTexture2D<float4> Output:register(u0);
float3 px(int2 p,int2 n){return Input.Load(int3(clamp(p,int2(0,0),n-1),0)).rgb;}
[numthreads(8,8,1)]void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Input.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 int2 p=int2(id.xy),n=int2(w,h);float4 c=Input.Load(int3(p,0));float3 b=0;
 [unroll]for(int y=-1;y<=1;y++)[unroll]for(int x=-1;x<=1;x++)b+=px(p+int2(x,y),n)*(2-abs(x))*(2-abs(y))/16.0;
 float k=clamp(dot(c.rgb-b,float3(.2126,.7152,.0722))*.65,-.035,.035);
 Output[id.xy]=float4(saturate(c.rgb+k),c.a);
}
