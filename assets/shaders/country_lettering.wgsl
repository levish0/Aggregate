#import bevy_pbr::forward_io::VertexOutput
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var lettering: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var lettering_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let coverage = textureSample(lettering, lettering_sampler, in.uv).a;
    let height_pixels = 1.0 / max(length(vec2<f32>(dpdx(in.uv.y), dpdy(in.uv.y))), 0.00001);
    let visibility = smoothstep(6.0, 11.0, height_pixels);
    if coverage < 0.002 { discard; }
    return vec4<f32>(0.014, 0.018, 0.014, coverage * visibility * 0.86);
}
