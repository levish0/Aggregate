#import bevy_pbr::forward_io::VertexOutput

struct ProvinceStyle {
    terrain: vec4<f32>,
    political: vec4<f32>,
    grouping: vec4<u32>,
};
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> selection: vec4<u32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var province_indices: texture_2d<u32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<storage, read> styles: array<ProvinceStyle>;

fn province_at(pixel: vec2<i32>) -> u32 {
    let dimensions=vec2<i32>(textureDimensions(province_indices));
    let wrapped=vec2<i32>(((pixel.x%dimensions.x)+dimensions.x)%dimensions.x,clamp(pixel.y,0,dimensions.y-1));
    return textureLoad(province_indices, wrapped, 0).r;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dimensions = vec2<f32>(textureDimensions(province_indices));
    let pixel = vec2<i32>(in.uv * dimensions);
    let id = province_at(pixel);
    let style = styles[id];
    let water = style.grouping.z != 0u;
    let normal = normalize(in.world_normal);
    let light = 0.48 + 0.52 * max(dot(normal, normalize(vec3<f32>(-0.5,0.85,-0.35))),0.0);
    let grain = sin(in.world_position.x*1.8)*sin(in.world_position.z*1.3)*0.01;
    let near_x = styles[province_at(pixel+vec2<i32>(8,0))].terrain.rgb;
    let near_z = styles[province_at(pixel+vec2<i32>(0,8))].terrain.rgb;
    let far_x = styles[province_at(pixel-vec2<i32>(8,0))].terrain.rgb;
    let far_z = styles[province_at(pixel-vec2<i32>(0,8))].terrain.rgb;
    var color = mix(style.terrain.rgb,(near_x+near_z+far_x+far_z)*0.25,0.55) * (light+grain);
    if !water {
        let rock = smoothstep(5.0,18.0,in.world_position.y);
        color = mix(color,vec3<f32>(0.48,0.47,0.40)*light,rock*0.55);
        let snow = smoothstep(16.0,27.0,in.world_position.y);
        color = mix(color,vec3<f32>(0.84,0.87,0.85)*light,snow);
        if selection.z != 0u { color = mix(color,style.political.rgb*light,0.66); }
    } else {
        let waves = sin(in.world_position.x*0.7)*sin(in.world_position.z*0.9)*0.0006;
        color = vec3<f32>(0.022,0.085,0.14)+waves;
    }
    let offset = max(vec2<i32>(1),vec2<i32>(ceil(fwidth(in.uv)*dimensions*0.8)));
    let right = province_at(pixel+vec2<i32>(offset.x,0));
    let below = province_at(pixel+vec2<i32>(0,offset.y));
    let border = styles[right].grouping.y != style.grouping.y || styles[below].grouping.y != style.grouping.y;
    let coast = styles[right].grouping.z != style.grouping.z || styles[below].grouping.z != style.grouping.z;
    if border && !water { color *= 0.55; }
    if coast { color = mix(color,vec3<f32>(0.40,0.55,0.52),0.5); }
    if id == selection.y && id != 0u { color = mix(color,vec3<f32>(0.78,0.69,0.38),0.24); }
    if id == selection.x && id != 0u {
        color = mix(color,vec3<f32>(0.85,0.60,0.20),0.35);
        if right != id || below != id { color = vec3<f32>(1.0,0.8,0.35); }
    }
    return vec4<f32>(color,1.0);
}
