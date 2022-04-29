[[vk::constant_id(0)]] const bool has_color = false;
[[vk::constant_id(1)]] const bool has_normal = false;
[[vk::constant_id(2)]] const bool has_tex_coord_0 = false;

struct FragmentInput {
	[[vk::location(0)]] float4 Color : COLOR;
	[[vk::location(1)]] float3 Normal: NORMAL;
	[[vk::location(2)]] float2 TexCoord_0: TEXCOORD_0;
};

float4 main( FragmentInput IN ): SV_Target0
{
	return IN.Color;
}