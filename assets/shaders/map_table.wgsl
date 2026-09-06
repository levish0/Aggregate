#import bevy_pbr::forward_io::VertexOutput
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> color:vec4<f32>;
@fragment
fn fragment(in:VertexOutput)->@location(0) vec4<f32> {
    let p=in.world_position.xz;
    let grain=sin(p.y*0.30+sin(p.x*0.007)*2.8+sin(p.y*0.028))*0.09;
    let boards=1.0-smoothstep(0.95,1.0,abs(sin(p.y*0.014)))*0.14;
    return vec4<f32>(color.rgb*(0.92+grain)*boards,1.0);
}
