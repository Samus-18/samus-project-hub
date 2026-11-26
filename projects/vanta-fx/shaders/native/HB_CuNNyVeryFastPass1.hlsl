// GPLv3 derivative of Magpie CuNNy Veryfast, from user-local Magpie source
// MUST distribute GNU GPLv3 license and corresponding derived source if distributing.
// Native Direct3D11 pass compiled from original neural weights; no Magpie.exe dependency.
#define MF float
#define MF1 float
#define MF2 float2
#define MF3 float3
#define MF4 float4
#define MF3x3 float3x3
#define MF4x4 float4x4
#define MulAdd(x,y,a) (mul(x,y)+(a))
#define MP_BLOCK_WIDTH 8
#define MP_BLOCK_HEIGHT 8
#define MP_NUM_THREADS_X 64
#define MP_NUM_THREADS_Y 1
#define MP_NUM_THREADS_Z 1
cbuffer FrameInfo:register(b0){uint4 SizeInfo;};
uint2 GetInputSize(){return SizeInfo.xy;}
uint2 GetOutputSize(){return SizeInfo.zw;}
float2 GetInputPt(){return 1.0/float2(SizeInfo.xy);}
float2 GetOutputPt(){return 1.0/float2(SizeInfo.zw);}
uint2 Rmp8x8(uint index){return uint2(index%8,index/8);}
SamplerState SP:register(s0);
SamplerState SL:register(s1);
Texture2D<float4> INPUT:register(t0);
RWTexture2D<float4> T0:register(u0);
RWTexture2D<float4> T1:register(u1);
#define O(t, x, y) t.SampleLevel(SP, pos + float2(x, y) * pt, 0)
#define V4 MF4
#define M4 MF4x4

//!DESC in (1x8)
//!BLOCK_SIZE 8
//!NUM_THREADS 64
//!IN INPUT
//!OUT T0, T1

#define L0(x, y) MF(dot(MF3(0.299, 0.587, 0.114), O(INPUT, x, y).rgb))

void Pass1(uint2 blockStart, uint3 tid) {
	float2 pt = float2(GetInputPt());
	uint2 gxy = Rmp8x8(tid.x) + blockStart;
	uint2 sz = GetInputSize();
	if (gxy.x >= sz.x || gxy.y >= sz.y)
		return;
	float2 pos = (gxy + 0.5) * pt;
	MF s0_0_0, s0_0_1, s0_0_2, s0_1_0, s0_1_1, s0_1_2, s0_2_0, s0_2_1, s0_2_2;
	V4 r0 = 0.0, r1 = 0.0;
	r0 = V4(1.026e-03, -2.981e-03, 2.268e-03, -1.057e-03);
	r1 = V4(-1.665e-03, 3.286e-03, -3.161e-03, -9.035e-04);
	s0_0_0 = L0(-1.0, -1.0); s0_0_1 = L0(0.0, -1.0); s0_0_2 = L0(1.0, -1.0);
	s0_1_0 = L0(-1.0, 0.0); s0_1_1 = L0(0.0, 0.0); s0_1_2 = L0(1.0, 0.0);
	s0_2_0 = L0(-1.0, 1.0); s0_2_1 = L0(0.0, 1.0); s0_2_2 = L0(1.0, 1.0);
	r0 = mad(s0_0_0, V4(4.998e-03, -1.996e-02, 2.062e-02, -1.826e-02), r0);
	r1 = mad(s0_0_0, V4(-5.265e-03, 2.075e-03, 2.429e-02, 3.332e-02), r1);
	r0 = mad(s0_0_1, V4(2.804e-02, 4.874e-02, 3.034e-02, 7.068e-03), r0);
	r1 = mad(s0_0_1, V4(2.430e-02, -1.450e-01, 1.032e-02, 4.446e-01), r1);
	r0 = mad(s0_0_2, V4(1.752e-02, -4.398e-02, -1.954e-02, 1.824e-02), r0);
	r1 = mad(s0_0_2, V4(-2.447e-02, 3.411e-02, -3.408e-02, -8.259e-02), r1);
	r0 = mad(s0_1_0, V4(3.185e-02, -3.662e-01, -1.870e-02, 8.200e-01), r0);
	r1 = mad(s0_1_0, V4(-7.897e-03, 1.151e-01, -2.607e-01, -3.053e-02), r1);
	r0 = mad(s0_1_1, V4(-9.682e-02, 4.676e-01, -1.874e-01, -8.066e-01), r0);
	r1 = mad(s0_1_1, V4(-8.105e-01, 4.792e-01, 8.066e-01, 9.627e-02), r1);
	r0 = mad(s0_1_2, V4(4.775e-01, -8.455e-02, 8.943e-02, -2.106e-02), r0);
	r1 = mad(s0_1_2, V4(8.912e-02, -9.258e-02, 3.846e-02, -7.281e-02), r1);
	r0 = mad(s0_2_0, V4(-1.763e-02, -2.789e-01, 4.132e-01, -2.679e-02), r0);
	r1 = mad(s0_2_0, V4(8.231e-03, 8.443e-02, -2.719e-01, 4.610e-04), r1);
	r0 = mad(s0_2_1, V4(3.664e-03, 2.998e-01, -6.781e-02, 2.461e-02), r0);
	r1 = mad(s0_2_1, V4(7.667e-01, -1.057e-02, -2.979e-01, 5.408e-02), r1);
	r0 = mad(s0_2_2, V4(-6.392e-02, -1.812e-02, 1.094e-02, 2.662e-03), r0);
	r1 = mad(s0_2_2, V4(-3.848e-02, 2.277e-02, -1.486e-02, -1.206e-02), r1);
	r0 = max(r0, 0.0);
	T0[gxy] = r0;
	r1 = max(r1, 0.0);
	T1[gxy] = r1;
}


[numthreads(64,1,1)]void CSMain(uint3 group:SV_GroupID,uint3 threadId:SV_GroupThreadID){Pass1(group.xy*uint2(8,8),threadId);}
