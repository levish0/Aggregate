#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var road_diffuse: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var road_sampler: sampler;
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(road_diffuse,road_sampler,in.uv);
    let edge = smoothstep(0.0,0.2,in.uv.y)*(1.0-smoothstep(0.8,1.0,in.uv.y));
    let fade = 1.0-smoothstep(150.0,280.0,length(view.world_position-in.world_position.xyz));
    return vec4<f32>(mix(sample.rgb,vec3<f32>(0.36,0.32,0.24),0.4),sample.a*edge*fade*0.85);
}
