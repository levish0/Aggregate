#import bevy_pbr::forward_io::VertexOutput

struct ProvinceStyle {
    terrain: vec4<f32>,
    political: vec4<f32>,
    grouping: vec4<u32>,
};
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> selection: vec4<u32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var province_indices: texture_2d<u32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<storage, read> styles: array<ProvinceStyle>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var color_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var color_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var terrain_diffuse: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var grass_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var terrain_normal: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var rock_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(9) var water_color: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(10) var water_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(11) var river_distance_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(12) var river_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(13) var terrain_weights: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(14) var terrain_weights_sampler: sampler;

fn province_at(pixel: vec2<i32>) -> u32 {
    let dimensions=vec2<i32>(textureDimensions(province_indices));
    let wrapped=vec2<i32>(((pixel.x%dimensions.x)+dimensions.x)%dimensions.x,clamp(pixel.y,0,dimensions.y-1));
    return textureLoad(province_indices, wrapped, 0).r;
}

// Reconstruct categorical coverage between texel centers. The CPU picking raster
// stays exact; display outlines follow continuous coverage instead of pixel edges.
fn coverage(group: u32, channel: u32, ids: vec4<u32>, weights: vec4<f32>) -> f32 {
    let matches = vec4<bool>(styles[ids.x].grouping[channel] == group, styles[ids.y].grouping[channel] == group, styles[ids.z].grouping[channel] == group, styles[ids.w].grouping[channel] == group);
    return dot(select(vec4<f32>(0.0), vec4<f32>(1.0), matches), weights);
}

fn province_coverage(province: u32, ids: vec4<u32>, weights: vec4<f32>, fraction: vec2<f32>, footprint: f32) -> f32 {
    if province == 0u { return 0.0; }
    let values = select(vec4<f32>(0.0), vec4<f32>(1.0), ids == vec4<u32>(province));
    let gradient = vec2<f32>(mix(values.y-values.x, values.w-values.z, fraction.y), mix(values.z-values.x, values.w-values.y, fraction.x));
    let signed_distance = (dot(values, weights)-0.5) / max(length(gradient),0.001);
    return smoothstep(-footprint*0.6,footprint*0.6,signed_distance);
}

fn boundary(group: u32, channel: u32, ids: vec4<u32>, weights: vec4<f32>, fraction: vec2<f32>, footprint: f32) -> f32 {
    let values = select(vec4<f32>(0.0), vec4<f32>(1.0), vec4<bool>(styles[ids.x].grouping[channel] == group, styles[ids.y].grouping[channel] == group, styles[ids.z].grouping[channel] == group, styles[ids.w].grouping[channel] == group));
    let gradient = vec2<f32>(mix(values.y-values.x, values.w-values.z, fraction.y), mix(values.z-values.x, values.w-values.y, fraction.x));
    let distance = abs(dot(values, weights)-0.5) / max(length(gradient),0.001);
    return 1.0-smoothstep(footprint*0.4,footprint*1.4,distance);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dimensions = vec2<f32>(textureDimensions(province_indices));
    let pixel = vec2<i32>(in.uv * dimensions);
    let id = province_at(pixel);
    let style = styles[id];
    let center = in.uv * dimensions - 0.5;
    let cell = vec2<i32>(floor(center));
    let fraction = fract(center);
    let ids = vec4<u32>(province_at(cell),province_at(cell+vec2<i32>(1,0)),province_at(cell+vec2<i32>(0,1)),province_at(cell+vec2<i32>(1,1)));
    let weights = vec4<f32>((1.0-fraction.x)*(1.0-fraction.y), fraction.x*(1.0-fraction.y), (1.0-fraction.x)*fraction.y, fraction.x*fraction.y);
    let footprint = max(length(fwidth(in.uv)*dimensions),0.02);
    let country_outline = boundary(style.grouping.y,1u,ids,weights,fraction,footprint);
    let state_outline = boundary(style.grouping.w,3u,ids,weights,fraction,footprint);
    let coast_outline = boundary(style.grouping.z,2u,ids,weights,fraction,footprint);
    let selected_group = select(styles[selection.x].grouping.w, styles[selection.x].grouping.y, selection.w != 0u);
    let selected_channel = select(3u,1u,selection.w != 0u);
    let selected_coverage = coverage(selected_group,selected_channel,ids,weights);
    let selection_outline = boundary(selected_group,selected_channel,ids,weights,fraction,footprint);
    let water = style.grouping.z != 0u;
    let normal = normalize(in.world_normal);
    let atlas = textureSample(color_map,color_sampler,in.uv).rgb;
    // Geographic masks select real material families independently of province IDs.
    var mask = textureSample(terrain_weights,terrain_weights_sampler,in.uv);
    mask.x = max(mask.x,smoothstep(0.12,0.45,1.0-normal.y));
    let base_weight = max(1.0-max(max(mask.x,mask.y),max(mask.z,mask.w)),0.0);
    let total_weight = max(base_weight+dot(mask,vec4<f32>(1.0)),0.001);
    let weights_material = array<f32,5>(base_weight,mask.x,mask.y,mask.z,mask.w);
    let detail_uv = in.world_position.xz*0.08;
    var diffuse = vec3<f32>(0.0);
    var surface_normal = vec3<f32>(0.0);
    for (var layer = 0u; layer < 5u; layer += 1u) {
        let weight = weights_material[layer]/total_weight;
        diffuse += textureSample(terrain_diffuse,grass_sampler,detail_uv,i32(layer)).rgb*weight;
        surface_normal += (textureSample(terrain_normal,rock_sampler,detail_uv,i32(layer)).rgb*2.0-1.0)*weight;
    }
    // Array textures carry mipmaps; preserve material coverage at every zoom.
    let detail_visibility = 1.0;
    let tangent = normalize(vec3<f32>(normal.y,-normal.x,0.0));
    let bitangent = cross(tangent,normal);
    let shaded_normal = normalize(normal*max(surface_normal.z,0.35)+(tangent*surface_normal.x+bitangent*surface_normal.y)*0.55*detail_visibility);
    let light = 0.38+0.62*max(dot(shaded_normal,normalize(vec3<f32>(-0.5,0.85,-0.35))),0.0);
    let sea = textureSample(water_color,water_sampler,in.uv).rgb;
    let river = textureSample(river_distance_map,river_sampler,in.uv).rg;
    let river_width = 0.25 + river.g * 1.35;
    let river_coverage = 1.0-smoothstep(river_width-footprint*0.6,river_width+footprint*0.6,river.r*16.0);
    var color = atlas * (0.68+light*0.45);
    let material_color = mix(atlas,diffuse,0.32)*light;
    color = mix(color,material_color,detail_visibility*0.65);
    if !water {
        let rock = smoothstep(5.0,18.0,in.world_position.y);
        color = mix(color,vec3<f32>(0.48,0.47,0.40)*light,rock*0.55);
        let snow = smoothstep(16.0,27.0,in.world_position.y);
        color = mix(color,vec3<f32>(0.84,0.87,0.85)*light,snow);
        let political = styles[ids.x].political.rgb*weights.x + styles[ids.y].political.rgb*weights.y + styles[ids.z].political.rgb*weights.z + styles[ids.w].political.rgb*weights.w;
        if selection.z != 0u { color = mix(color,political*light,0.58); }
    } else {
        let waves = sin(in.world_position.x*0.7)*sin(in.world_position.z*0.9)*0.0006;
        color = mix(vec3<f32>(0.025,0.10,0.18),sea,0.6)+waves;
    }
    if !water { color *= 1.0-country_outline*0.55; }
    if !water && selection.z != 0u { color *= 1.0-state_outline*0.22; }
    color = mix(color,vec3<f32>(0.40,0.55,0.52),coast_outline*0.3);
    if !water { color = mix(color,vec3<f32>(0.045,0.19,0.26),river_coverage*0.80); }
    let hovered_coverage = province_coverage(selection.y,ids,weights,fraction,footprint);
    color = mix(color,vec3<f32>(1.0),hovered_coverage*0.12);
    if selected_group != 0u {
        color = mix(color,vec3<f32>(1.0),selected_coverage*0.12);
        color = mix(color,vec3<f32>(1.0),selection_outline*0.92);
    } else {
        let selected_province_coverage = province_coverage(selection.x,ids,weights,fraction,footprint);
        color = mix(color,vec3<f32>(1.0),selected_province_coverage*0.20);
    }
    return vec4<f32>(color,1.0);
}
