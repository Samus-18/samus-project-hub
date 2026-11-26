// FSRCNNX_x2_8-0-4-1 real GPLv3 neural weights sourced from user-local Magpie/FSRCNNX effect.
// GPLv3 derivative: redistribution requires source and license; this is an actual port, not mock.
#define MF float
#define MF1 float
#define MF2 float2
#define MF3 float3
#define MF4 float4
#define MF2x3 float2x3
#define MF3x3 float3x3
#define MF4x4 float4x4
#define MulAdd(x,y,a) (mul((x),(y))+(a))
#define MP_BLOCK_WIDTH 32
#define MP_BLOCK_HEIGHT 24
#define MP_NUM_THREADS_X 128
#define MP_NUM_THREADS_Y 1
#define MP_NUM_THREADS_Z 1
cbuffer FrameInfo:register(b0){uint4 SizeInfo;};
uint2 GetInputSize(){return SizeInfo.xy;}
uint2 GetOutputSize(){return SizeInfo.zw;}
float2 GetInputPt(){return 1.0/float2(SizeInfo.xy);}
float2 GetOutputPt(){return 1.0/float2(SizeInfo.zw);}
uint2 Rmp8x8(uint index){return uint2(index%8,index/8);}
SamplerState sam:register(s0);
SamplerState sam1:register(s1);
Texture2D<float4> INPUT:register(t0);
RWTexture2D<float4> featureMap1:register(u0);
RWTexture2D<float4> featureMap2:register(u1);

//!DESC feature map
//!IN INPUT
//!OUT featureMap1, featureMap2
//!BLOCK_SIZE 32, 24
//!NUM_THREADS 128

#define SH_PIXELS_X  (MP_BLOCK_WIDTH + 4)
#define SH_PIXELS_Y  (MP_BLOCK_HEIGHT + 4)

groupshared MF shPixelsY[SH_PIXELS_Y][SH_PIXELS_X];

MF GetLuma(MF3 rgb) {
	return dot(MF3(0.299, 0.587, 0.114), rgb);
}

void Pass1(uint2 blockStart, uint3 threadId) {
	float2 inputPt = GetInputPt();
	uint2 inputSize = GetInputSize();
	uint i;

	for (i = threadId.x * 2; i < SH_PIXELS_X * SH_PIXELS_Y / 2; i += MP_NUM_THREADS_X * 2) {
		uint2 pos = uint2(i % SH_PIXELS_X, i / SH_PIXELS_X * 2);
		const float2 tpos = (blockStart + pos - 1.5f) * inputPt;

		const MF4 sr = INPUT.GatherRed(sam, tpos);
		const MF4 sg = INPUT.GatherGreen(sam, tpos);
		const MF4 sb = INPUT.GatherBlue(sam, tpos);

		shPixelsY[pos.y][pos.x] = GetLuma(MF3(sr.w, sg.w, sb.w));
		shPixelsY[pos.y][pos.x + 1] = GetLuma(MF3(sr.z, sg.z, sb.z));
		shPixelsY[pos.y + 1][pos.x] = GetLuma(MF3(sr.x, sg.x, sb.x));
		shPixelsY[pos.y + 1][pos.x + 1] = GetLuma(MF3(sr.y, sg.y, sb.y));
	}

	GroupMemoryBarrierWithGroupSync();

	for (i = threadId.x; i < MP_BLOCK_WIDTH * MP_BLOCK_HEIGHT; i += MP_NUM_THREADS_X) {
		const uint2 pos = uint2(i % MP_BLOCK_WIDTH, i / MP_BLOCK_WIDTH);
		const uint2 destPos = blockStart + pos;

		if (destPos.x >= inputSize.x || destPos.y >= inputSize.y) {
			continue;
		}

		MF src[5][5];
		[unroll]
		for (int i = 0; i < 5; ++i) {
			[unroll]
			for (int j = 0; j < 5; ++j) {
				src[j][i] = shPixelsY[pos.y + i][pos.x + j];
			}
		}

		MF4 target1 = MF4(-0.1572492271661758, -0.0120896836742759, 0.0061487639322877, -0.2852848768234253);
		target1 = mad(src[0][0], MF4(-0.0047900392673910, 0.0537447109818459, -0.0000247144635068, 0.0066653941757977), target1);
		target1 = mad(src[0][1], MF4(0.0073144687339664, -0.0309004038572311, -0.0109181385487318, -0.0092840325087309), target1);
		target1 = mad(src[0][2], MF4(0.0591700896620750, 0.1974907070398331, -0.0197357516735792, -0.0546554848551750), target1);
		target1 = mad(src[0][3], MF4(-0.0011764382943511, -0.0299451071768999, 0.0229587312787771, 0.0021908886265010), target1);
		target1 = mad(src[0][4], MF4(0.0098101310431957, 0.0080995410680771, -0.0030452020000666, -0.0132035519927740), target1);
		target1 = mad(src[1][0], MF4(-0.0168330334126949, -0.0743711441755295, -0.0259261634200811, 0.0234480481594801), target1);
		target1 = mad(src[1][1], MF4(0.0239933785051107, 0.1896541714668274, 0.0207756329327822, -0.0370332375168800), target1);
		target1 = mad(src[1][2], MF4(0.0094799501821399, -0.0652511194348335, -0.0004292793164495, -0.0726212188601494), target1);
		target1 = mad(src[1][3], MF4(0.0297284796833992, -0.1210186630487442, -0.0202929321676493, -0.0574462898075581), target1);
		target1 = mad(src[1][4], MF4(-0.0318185277283192, 0.0840775370597839, 0.0110451309010386, 0.0415569432079792), target1);
		target1 = mad(src[2][0], MF4(-0.0253141783177853, 0.1168256178498268, 0.1159729585051537, 0.0963164269924164), target1);
		target1 = mad(src[2][1], MF4(-0.1103615835309029, -0.0276833958923817, -0.4999594092369080, 0.1053867191076279), target1);
		target1 = mad(src[2][2], MF4(1.1100435256958008, 0.0646764487028122, 0.0154005717486143, 0.8891586661338806), target1);
		target1 = mad(src[2][3], MF4(0.1229330673813820, 0.1719468832015991, 0.5730338096618652, -0.1645544171333313), target1);
		target1 = mad(src[2][4], MF4(-0.0090442728251219, -0.3023961782455444, -0.1589493155479431, 0.0418574027717113), target1);
		target1 = mad(src[3][0], MF4(0.0031942036002874, -0.1310926079750061, 0.0075543406419456, -0.0016449346439913), target1);
		target1 = mad(src[3][1], MF4(-0.0995150282979012, -0.0701921209692955, -0.0130895879119635, 0.1344170123338699), target1);
		target1 = mad(src[3][2], MF4(0.0060519003309309, -0.1533465683460236, 0.0114194005727768, 0.0264683905988932), target1);
		target1 = mad(src[3][3], MF4(0.0244008023291826, 0.1881769001483917, -0.0206351149827242, -0.0628309547901154), target1);
		target1 = mad(src[3][4], MF4(0.0075713125988841, 0.0508594363927841, 0.0430423170328140, -0.0124188791960478), target1);
		target1 = mad(src[4][0], MF4(-0.0166875869035721, -0.0047865519300103, 0.0006719123339280, 0.0316803231835365), target1);
		target1 = mad(src[4][1], MF4(-0.0058461269363761, 0.0990798473358154, -0.0177743826061487, -0.0066122291609645), target1);
		target1 = mad(src[4][2], MF4(-0.0972401946783066, -0.0225446373224258, -0.0037693574558944, 0.1953062713146210), target1);
		target1 = mad(src[4][3], MF4(-0.0216837190091610, -0.1824268400669098, 0.0069816261529922, 0.0283037684857845), target1);
		target1 = mad(src[4][4], MF4(-0.0025767991319299, 0.0459827110171318, -0.0080216089263558, 0.0084134787321091), target1);

		MF4 target2 = MF4(0.0541447550058365, 0.0088306749239564, -0.0112389577552676, -0.0127860950306058);
		target2 = mad(src[0][0], MF4(0.0142660010606050, 0.0137931071221828, 0.0061188107356429, -0.0104134222492576), target2);
		target2 = mad(src[0][1], MF4(0.0147292809560895, -0.0289912857115269, 0.0266769435256720, 0.0933856964111328), target2);
		target2 = mad(src[0][2], MF4(-0.1734338253736496, 0.1116316691040993, -0.1973157376050949, -0.0581855811178684), target2);
		target2 = mad(src[0][3], MF4(0.0347507223486900, -0.0341566652059555, 0.0061667622067034, 0.0075258882716298), target2);
		target2 = mad(src[0][4], MF4(0.0069884369149804, -0.0194250214844942, 0.0080830128863454, -0.0036874092184007), target2);
		target2 = mad(src[1][0], MF4(0.0233764201402664, 0.0344744995236397, 0.0162145942449570, 0.0979529991745949), target2);
		target2 = mad(src[1][1], MF4(0.1280796974897385, -0.1018339172005653, -0.0132977198809385, -0.0019474622095004), target2);
		target2 = mad(src[1][2], MF4(0.4286882579326630, 0.1222677752375603, 0.7046694159507751, 0.0945475697517395), target2);
		target2 = mad(src[1][3], MF4(0.1107441782951355, -0.0134433070197701, -0.0174900908023119, -0.1686445474624634), target2);
		target2 = mad(src[1][4], MF4(0.0321478620171547, 0.0065357843413949, 0.0300805997103453, 0.0420113280415535), target2);
		target2 = mad(src[2][0], MF4(-0.1240341588854790, 0.0950303301215172, -0.0129648456349969, -0.2681856453418732), target2);
		target2 = mad(src[2][1], MF4(0.4846960902214050, 0.0351924635469913, 0.0223043337464333, -0.1273630708456039), target2);
		target2 = mad(src[2][2], MF4(-1.9379507303237915, -0.2444442063570023, 0.0291962660849094, -0.3835578560829163), target2);
		target2 = mad(src[2][3], MF4(0.6396278142929077, -0.0765938311815262, -0.0552659817039967, 0.4393545985221863), target2);
		target2 = mad(src[2][4], MF4(-0.1969728022813797, -0.0607173256576061, 0.0131113547831774, 0.0542017817497253), target2);
		target2 = mad(src[3][0], MF4(0.0091696009039879, -0.0031533432193100, -0.0368777588009834, -0.0459998287260532), target2);
		target2 = mad(src[3][1], MF4(0.1096992492675781, 0.2597902715206146, 0.0304869692772627, -0.0195200722664595), target2);
		target2 = mad(src[3][2], MF4(0.2889648377895355, -0.4275591969490051, -0.7414156794548035, 0.2695442438125610), target2);
		target2 = mad(src[3][3], MF4(0.0892018377780914, -0.0229137558490038, 0.0244414471089840, -0.1926898956298828), target2);
		target2 = mad(src[3][4], MF4(0.0576358586549759, 0.0027846973389387, -0.0036861505359411, -0.0253547113388777), target2);
		target2 = mad(src[4][0], MF4(0.0159624069929123, 0.0319602824747562, 0.0019470085389912, 0.0089780492708087), target2);
		target2 = mad(src[4][1], MF4(0.0552792511880398, 0.0543054342269897, 0.0134062822908163, 0.0545728243887424), target2);
		target2 = mad(src[4][2], MF4(-0.1170092225074768, 0.1963327825069427, 0.1503890156745911, 0.1891828328371048), target2);
		target2 = mad(src[4][3], MF4(-0.0084421783685684, 0.1297017931938171, -0.0330600887537003, -0.0942063704133034), target2);
		target2 = mad(src[4][4], MF4(0.0118440408259630, -0.0337875857949257, 0.0055063469335437, 0.0254479162395000), target2);

		featureMap1[destPos] = target1;
		featureMap2[destPos] = target2;
	}
}



[numthreads(128,1,1)]void CSMain(uint3 group:SV_GroupID,uint3 threadId:SV_GroupThreadID){Pass1(group.xy*uint2(32,24),threadId);}
