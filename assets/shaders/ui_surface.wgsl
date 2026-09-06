#import bevy_ui::ui_vertex_output::UiVertexOutput
@group(1) @binding(0) var<uniform> parameters: vec4<f32>;
@group(1) @binding(1) var background: texture_2d<f32>;
@group(1) @binding(2) var background_sampler: sampler;
@group(1) @binding(3) var detail: texture_2d<f32>;
@group(1) @binding(4) var detail_sampler: sampler;
@group(1) @binding(5) var shading: texture_2d<f32>;
@group(1) @binding(6) var shading_sampler: sampler;
@group(1) @binding(7) var pattern: texture_2d<f32>;
@group(1) @binding(8) var pattern_sampler: sampler;

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    return select(c * 12.92, 1.055 * pow(max(c, vec3(0.)), vec3(1. / 2.4)) - 0.055, c > vec3(0.0031308));
}
fn to_linear(c: vec3<f32>) -> vec3<f32> {
    return select(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), c > vec3(0.04045));
}
fn overlay(base: vec3<f32>, top: vec3<f32>) -> vec3<f32> {
    return select(2. * base * top, 1. - 2. * (1. - base) * (1. - top), base > vec3(0.5));
}
fn corner_axis(position: f32, extent: f32, source: f32, border: f32) -> f32 {
    let edge = min(border * 0.5, extent * 0.5);
    if position < edge { return position / max(edge, 0.001) * border; }
    if position > extent - edge { return source - (extent - position) / max(edge, 0.001) * border; }
    return border + (position - edge) / max(extent - 2. * edge, 0.001) * (source - 2. * border);
}
@fragment
fn fragment(input: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;
    let pixel = uv * input.size;
    let button = parameters.x > 0.5;
    var base_uv = fract(pixel * 2. / vec2<f32>(textureDimensions(background)));
    if button {
        base_uv = (vec2(corner_axis(pixel.x, input.size.x, 38., 14.) + parameters.y * 38., corner_axis(pixel.y, input.size.y, 38., 14.)) + 0.5) / vec2<f32>(textureDimensions(background));
    }
    let base = textureSample(background, background_sampler, base_uv);
    let grain_uv = fract(pixel * 2. / vec2<f32>(textureDimensions(detail)));
    let grain = to_srgb(textureSample(detail, detail_sampler, grain_uv).rgb);
    let fade = to_srgb(textureSample(shading, shading_sampler, uv).rgb);
    var color = to_srgb(base.rgb);
    color = mix(color, overlay(color, fade), select(0.8, 0.5, button));
    color = mix(color, overlay(color, grain), select(0.25, 0.7, button));
    let pattern_uv = fract(pixel * 2. / vec2<f32>(textureDimensions(pattern)));
    let decoration = to_srgb(textureSample(pattern, pattern_sampler, pattern_uv).rgb);
    color = mix(color, overlay(color, decoration), select(0.065, 0.2, button));
    color *= 1. + parameters.z * 0.25 - parameters.w * 0.16;
    return vec4(to_linear(clamp(color, vec3(0.), vec3(1.))), base.a);
}
