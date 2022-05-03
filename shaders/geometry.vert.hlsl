// #include "geometry_common_hlsl"

// struct SVertexData {
// 	_location(0) float3 Position: POSITION;
// 	_location(1) float3 Color: COLOR;
// 	_location(2) float3 Normal: NORMAL;
// 	_location(3) float2 TexCoord_0: TEXCOORD_0;
// };

// struct SViewProjection {
// 	matrix mt;
// };

// _binding(0)  ConstantBuffer<SViewProjection> view_projection : register(b0);

// SVertexOutput main(SVertexData IN) {
// 	SVertexOutput OUT;

// 	OUT.Position = mul(view_projection.mt, float4(IN.Position, 1.0f));

// 	if (has_color) {
// 		OUT.Color = float4(IN.Color, 1.0f);
// 	}

// 	if (has_normal) {
// 		OUT.Normal = IN.Normal;
// 	}

// 	if (has_tex_coord_0) {
// 		OUT.TexCoord_0 = IN.TexCoord_0;
// 	}

// 	return OUT;
// }

struct ViewProjection
{
    matrix VP;
};

[[vk::binding(0)]] ConstantBuffer<ViewProjection> ViewProjectionCB : register(b0);

struct VertexPosColor
{
    uint id: SV_VertexID;
};
 
struct VertexShaderOutput
{
    [[vk::location(0)]] float4 Color    : COLOR;
    float4 Position : SV_Position;
};

static float offset = 0.3;
static float depth = 0.1;
static float3 color0 = float3(0.0, 0.0, 1.0);
static float3 color1 = float3(0.0, 1.0, 0.0);

static float3 positions[] = {
    float3(0.0, 0.0, 0.0),
    float3(1.0, 0.0, 0.0),
    float3(0.0, 1.0, 0.0),
    float3(0.0 + 0.3, 1.0 + offset, depth),
    float3(1.0 + 0.3, 0.0 + offset, depth),
    float3(0.0 + 0.3, 0.0 + offset, depth)
};

static float3 colors[] = {
    color0,
    color0,
    color0,
    color1,
    color1,
    color1
};

VertexShaderOutput main(VertexPosColor IN)
{
    VertexShaderOutput OUT;

    float3 position = positions[IN.id];
    float3 color = colors[IN.id];
    OUT.Position = mul(ViewProjectionCB.VP, float4(position, 1.0f));
    OUT.Color = float4(color, 1.0f);
 
    return OUT;
}