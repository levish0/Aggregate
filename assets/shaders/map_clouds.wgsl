#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::{view, globals}
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var cloud_density: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var cloud_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var cloud_normal: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var normal_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.world_position.xz/180.0+globals.time*vec2<f32>(0.0003,0.00008);
    let density = textureSample(cloud_density,cloud_sampler,uv).g;
    let normal = textureSample(cloud_normal,normal_sampler,uv).rgb*2.0-1.0;
    let light = 0.72+0.28*max(dot(normalize(normal.xzy),normalize(vec3<f32>(-0.5,0.85,-0.35))),0.0);
    let distance = length(view.world_position-in.world_position.xyz);
    let visibility = (1.0-smoothstep(180.0,380.0,distance))*smoothstep(8.0,25.0,distance);
    let alpha = smoothstep(0.1,0.7,density)*0.66*visibility;
    return vec4<f32>(vec3<f32>(0.86,0.91,1.0)*light,alpha);
}
