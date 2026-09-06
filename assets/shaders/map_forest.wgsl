#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> tint: vec4<f32>;
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let distance = length(view.world_position-in.world_position.xyz);
    if distance > 320.0 { discard; }
    let normal = normalize(in.world_normal);
    let lighting = 0.30+0.82*max(dot(normal,normalize(vec3<f32>(-0.5,0.85,-0.35))),0.0);
    return vec4<f32>(in.color.rgb*tint.rgb*lighting,1.0);
}
