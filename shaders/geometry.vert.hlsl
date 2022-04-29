[[vk::constant_id(0)]] const bool has_color = false;
[[vk::constant_id(1)]] const bool has_normal = false;
[[vk::constant_id(2)]] const bool has_tex_coord_0 = false;

struct ViewProjection {
	matrix VP;
};

[[vk::binding(0)]] ConstantBuffer<ViewProjection> view_projection : register(b0);

struct VertexData {
	[[vk::location(0)]] float3 Position: POSITION;
	[[vk::location(1)]] float3 Color: COLOR;
	[[vk::location(2)]] float3 Normal: NORMAL;
	[[vk::location(3)]] float2 TexCoord_0: TEXCOORD_0;
};

struct VertexOutput {
	[[vk::location(0)]] float4 Color : COLOR;
	[[vk::location(1)]] float3 Normal: NORMAL;
	[[vk::location(2)]] float2 TexCoord_0: TEXCOORD_0;
	float4 Position : SV_Position;
};

VertexOutput main(VertexData IN) {
	VertexOutput OUT;

	OUT.Position = mul(view_projection.VP, float4(IN.Position, 1.0f));

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