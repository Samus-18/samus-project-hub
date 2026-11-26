// Real Shader Model 6.0 Wave64 subgroup reduction. No game injection.
// 64 input threads calculate a wave-wide reduction and write actual GPU data.
#define HB_ROOT "UAV(u0)"
RWByteAddressBuffer Output:register(u0);
[RootSignature(HB_ROOT)]
[numthreads(64,1,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
    uint lanes=WaveGetLaneCount();
    uint sum=WaveActiveSum(id.x+1);
    // Both outputs are produced using native subgroup operations.
    Output.Store(id.x*4,(sum<<8)|lanes);
}
