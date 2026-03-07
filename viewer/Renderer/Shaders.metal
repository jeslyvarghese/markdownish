#include <metal_stdlib>
using namespace metal;

struct VertexIn {
    float2 position [[attribute(0)]];
    float2 uv       [[attribute(1)]];
};

struct VertexOut {
    float4 position [[position]];
    float2 uv;
};

struct ScrollUniforms {
    float scrollY;         // scroll offset in document pixels
    float viewportHeight;  // viewport height in pixels
    float documentHeight;  // total document height in pixels
    float scale;           // display scale factor (retina)
};

vertex VertexOut vertex_main(VertexIn in [[stage_in]]) {
    VertexOut out;
    out.position = float4(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

fragment float4 fragment_main(
    VertexOut in [[stage_in]],
    texture2d<float> tex [[texture(0)]],
    constant ScrollUniforms& uniforms [[buffer(0)]]
) {
    constexpr sampler s(filter::linear, address::clamp_to_edge);

    // Offset the UV vertically by scroll position
    float uvScrollOffset = uniforms.scrollY / (uniforms.documentHeight * uniforms.scale);
    float2 scrolledUV = float2(in.uv.x, in.uv.y + uvScrollOffset);

    return tex.sample(s, scrolledUV);
}
