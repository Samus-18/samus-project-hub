Texture2D<float4> Source:register(t0);
RWTexture2D<float4> Target:register(u0);
float3 Filmic(float3 x){return saturate((x*(2.51*x+0.03))/(x*(2.43*x+0.59)+0.14));}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Target.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 float4 src=Source.Load(int3(id.xy,0));float3 c=Filmic(max(src.rgb*1.26+0.012,0.0));
 float l=dot(c,float3(0.2126,0.7152,0.0722));
 float shadows=(1.0-smoothstep(0.15,0.6,l))*0.055;
 float highlights=smoothstep(0.45,0.9,l)*0.045;
 c+=float3(-shadows+highlights,0.2*shadows+0.25*highlights,shadows-highlights);
 Target[id.xy]=float4(saturate(c),src.a);
}
