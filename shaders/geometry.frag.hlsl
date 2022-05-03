// #include "geometry_common_hlsl"

// float4 main( SFragmentInput IN ): SV_Target0
// {
// 	return IN.Color;
// }

struct PixelShaderInput
{
    [[vk::location(0)]] float4 Color: COLOR;
};

float4 main( PixelShaderInput IN ): SV_Target0
{
    return IN.Color;
}