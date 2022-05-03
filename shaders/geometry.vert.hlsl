#include "geometry_common_hlsl"

struct SVertexData {
	_location(0) float3 Position: POSITION;
	_location(1) float3 Color: COLOR;
	_location(2) float3 Normal: NORMAL;
	_location(3) float2 TexCoord_0: TEXCOORD_0;
};

struct SViewProjection {
	matrix mt;
};

_binding(0)  ConstantBuffer<SViewProjection> view_projection : register(b0);

SVertexOutput main(SVertexData IN) {
	SVertexOutput OUT;

	OUT.Position = mul(view_projection.mt, float4(IN.Position, 1.0f));

	if (has_color) {
		OUT.Color = float4(IN.Color, 1.0f);
	}

	if (has_normal) {
		OUT.Normal = IN.Normal;
	}

	if (has_tex_coord_0) {
		OUT.TexCoord_0 = IN.TexCoord_0;
	}

	return OUT;
}