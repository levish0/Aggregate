#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::{view, globals}
#import "shaders/map_cloud_density.wgsl"::{cloud_field}
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var cloud_density: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var cloud_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var cloud_normal: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var normal_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> cloud_visibility: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    if cloud_visibility.x < 0.001 { discard; }
    let field = cloud_field(in.world_position.xz,cloud_visibility.y,globals.time,cloud_density,cloud_sampler);
    let density = field.density;
    let first_normal = textureSample(cloud_normal,normal_sampler,field.primary_uv).rgb*2.0-1.0;
    let second_normal = textureSample(cloud_normal,normal_sampler,field.secondary_uv).rgb*2.0-1.0;
    let normal = normalize(first_normal+second_normal*0.35);
    let light = 0.72+0.28*max(dot(normalize(normal.xzy),normalize(vec3<f32>(-0.5,0.85,-0.35))),0.0);
    let alpha = smoothstep(0.28,0.92,density)*0.50*cloud_visibility.x;
    return vec4<f32>(vec3<f32>(0.86,0.91,1.0)*light,alpha);
}
