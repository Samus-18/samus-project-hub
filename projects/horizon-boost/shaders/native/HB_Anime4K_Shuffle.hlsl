// MIT License

// Copyright (c) 2019-2021 bloc97
// All rights reserved.

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// Exact 2x depth-to-space residual. Preserve sampled input alpha instead of adding residual to alpha.
Texture2D<float4> src:register(t0);
Texture2D<float4> residual:register(t1);
RWTexture2D<float4> dst:register(u0);
float4 loadpx(int2 p,int2 n){return src.Load(int3(clamp(p,int2(0,0),n-1),0));}
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h,sw,sh;dst.GetDimensions(w,h);src.GetDimensions(sw,sh);
 if(id.x>=w||id.y>=h)return;
 float2 p=(float2(id.xy)+0.5)*float2(sw,sh)/float2(w,h)-0.5;
 int2 b=int2(floor(p)),n=int2(sw,sh);float2 f=frac(p);
 float4 c=lerp(lerp(loadpx(b,n),loadpx(b+int2(1,0),n),f.x),lerp(loadpx(b+int2(0,1),n),loadpx(b+int2(1,1),n),f.x),f.y);
 float4 r=residual.Load(int3(id.xy/2,0));
 dst[id.xy]=float4(saturate(c.rgb+r[(id.y%2)*2+id.x%2]),c.a);
}
