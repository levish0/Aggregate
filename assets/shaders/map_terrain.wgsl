#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::{view, globals}

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
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var terrain_diffuse_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var terrain_normal: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var terrain_normal_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(9) var water_color: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(10) var water_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(11) var river_distance_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(12) var river_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(13) var terrain_weights: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(14) var terrain_weights_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(16) var<uniform> map_view: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(15) var terrain_indices: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(17) var terrain_properties: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(18) var terrain_properties_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(19) var terrain_relief: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(20) var terrain_relief_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(21) var water_normal: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(22) var water_normal_sampler: sampler;

struct TerrainSurface {
    diffuse: vec3<f32>,
    normal: vec3<f32>,
    properties: vec4<f32>,
};

fn terrain_surface(uv: vec2<f32>, detail_uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> TerrainSurface {
    let size = vec2<i32>(textureDimensions(terrain_indices));
    let coordinate = uv * vec2<f32>(size) - 0.5;
    let origin = vec2<i32>(floor(coordinate));
    let fraction = fract(coordinate);
    var material_ids: array<i32,16>;
    var material_weights: array<f32,16>;
    var count = 0u;
    // Accumulate duplicate categorical IDs before choosing the four dominant layers.
    // Filtering an ID texture directly would sample unrelated terrain materials.
    for (var z = 0; z < 2; z += 1) {
        for (var x = 0; x < 2; x += 1) {
            let point = origin + vec2<i32>(x,z);
            let pixel = vec2<i32>((point.x % size.x + size.x) % size.x, clamp(point.y,0,size.y-1));
            let ids = vec4<i32>(round(textureLoad(terrain_indices,pixel,0)*255.0));
            let weights = textureLoad(terrain_weights,pixel,0);
            let cell_weight = select(1.0-fraction.x,fraction.x,x==1) * select(1.0-fraction.y,fraction.y,z==1);
            for (var slot = 0u; slot < 4u; slot += 1u) {
                let weight = weights[slot] * cell_weight;
                if weight > 0.001 {
                    var found = count;
                    for (var index = 0u; index < count; index += 1u) {
                        if material_ids[index] == ids[slot] { found = index; break; }
                    }
                    if found == count { material_ids[count] = ids[slot]; count += 1u; }
                    material_weights[found] += weight;
                }
            }
        }
    }
    var selected_ids: array<i32,4>;
    var selected_weights = vec4<f32>(0.0);
    var samples: array<vec4<f32>,4>;
    var heights = vec4<f32>(0.0);
    for (var slot = 0u; slot < 4u; slot += 1u) {
        var largest = 0.0;
        var selected = 0u;
        for (var index = 0u; index < count; index += 1u) {
            if material_weights[index] > largest { largest = material_weights[index]; selected = index; }
        }
        selected_ids[slot] = material_ids[selected];
        selected_weights[slot] = largest;
        material_weights[selected] = 0.0;
        samples[slot] = textureSampleGrad(terrain_diffuse,terrain_diffuse_sampler,detail_uv,selected_ids[slot],dx,dy);
        heights[slot] = samples[slot].a;
    }
    selected_weights /= max(dot(selected_weights,vec4<f32>(1.0)),0.001);
    // Diffuse alpha is authored height. Highest surfaces win only in transition zones.
    let height_weights = heights * smoothstep(vec4<f32>(0.0),vec4<f32>(0.1),selected_weights) + selected_weights;
    let blend_start = max(max(height_weights.x,height_weights.y),max(height_weights.z,height_weights.w)) - 0.1;
    var blend = max(height_weights-vec4<f32>(blend_start),vec4<f32>(0.0)) * selected_weights;
    blend /= max(dot(blend,vec4<f32>(1.0)),0.001);
    var diffuse = vec3<f32>(0.0);
    var packed_normal = vec4<f32>(0.0);
    var properties = vec4<f32>(0.0);
    for (var slot = 0u; slot < 4u; slot += 1u) {
        diffuse += samples[slot].rgb * blend[slot];
        packed_normal += textureSampleGrad(terrain_normal,terrain_normal_sampler,detail_uv,selected_ids[slot],dx,dy) * blend[slot];
        properties += textureSampleGrad(terrain_properties,terrain_properties_sampler,detail_uv,selected_ids[slot],dx,dy) * blend[slot];
    }
    // Imported RRxG normal encoding: horizontal in green, inverted vertical in alpha.
    let xy = vec2<f32>(packed_normal.g*2.0-1.0,1.0-packed_normal.a*2.0);
    return TerrainSurface(diffuse,vec3<f32>(xy,sqrt(max(1.0-dot(xy,xy),0.0))),properties);
}

fn terrain_soft_light(detail: vec3<f32>, tint: vec3<f32>) -> vec3<f32> {
    return (1.0-2.0*tint)*detail*detail + 2.0*tint*detail;
}

fn water_surface(uv: vec2<f32>, world: vec3<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec3<f32> {
    let drift = globals.time * vec2<f32>(0.008,-0.004);
    let wave_uv = world.xz * 0.32;
    let first = textureSampleGrad(water_normal,water_normal_sampler,wave_uv+drift,dx*1.28,dy*1.28).rgb*2.0-1.0;
    let second = textureSampleGrad(water_normal,water_normal_sampler,wave_uv*0.63-drift*0.8,dx*0.8064,dy*0.8064).rgb*2.0-1.0;
    let normal = normalize(vec3<f32>((first.x+second.x)*0.45,1.0,(first.y+second.y)*0.45));
    let eye = normalize(view.world_position-world);
    let light = normalize(vec3<f32>(-0.5,0.85,-0.35));
    let halfway = normalize(eye+light);
    let fresnel = 0.04+0.65*pow(1.0-max(dot(eye,normal),0.0),5.0);
    let source = textureSample(water_color,water_sampler,uv);
    let reflection = vec3<f32>(0.24,0.36,0.54);
    let specular = pow(max(dot(normal,halfway),0.0),96.0) * source.a * 0.7;
    let ripple_light = 0.72+0.45*max(dot(normal,light),0.0);
    return mix(source.rgb*0.78,reflection,fresnel)*ripple_light + vec3<f32>(specular);
}

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
    let relief = textureSample(terrain_relief,terrain_relief_sampler,in.uv);
    let normal = normalize(relief.rgb*2.0-1.0);
    let atlas = textureSample(color_map,color_sampler,in.uv).rgb;
    let detail_uv = in.world_position.xz*0.25;
    let detail_dx = dpdx(detail_uv);
    let detail_dy = dpdy(detail_uv);
    var surface = TerrainSurface(atlas,vec3<f32>(0.0,0.0,1.0),vec4<f32>(0.0,0.0,0.1,0.9));
    if !water {
        surface = terrain_surface(in.uv,detail_uv,detail_dx,detail_dy);
    }
    let tangent = normalize(vec3<f32>(normal.y,-normal.x,0.0));
    let bitangent = cross(tangent,normal);
    let shaded_normal = normalize(normal*max(surface.normal.z,0.1)+(tangent*surface.normal.x+bitangent*surface.normal.y)*0.75);
    let sun = normalize(vec3<f32>(-0.5,0.85,-0.35));
    let light = (0.28+0.90*max(dot(shaded_normal,sun),0.0))*relief.a;
    let river = textureSample(river_distance_map,river_sampler,in.uv).rg;
    let river_width = 0.18 + river.g * 0.85;
    let river_footprint = max(length(fwidth(in.uv)*vec2<f32>(textureDimensions(river_distance_map))),0.02);
    let river_coverage = (1.0-smoothstep(max(0.0,river_width-river_footprint*0.5),river_width+river_footprint*0.5,river.r*16.0)) * min(1.0,river_width/max(river_footprint,0.001));
    let albedo = mix(surface.diffuse,terrain_soft_light(surface.diffuse,atlas),1.0-surface.properties.r);
    let eye = normalize(view.world_position-in.world_position.xyz);
    let roughness = clamp(surface.properties.a,0.2,1.0);
    let specular = pow(max(dot(shaded_normal,normalize(sun+eye)),0.0),mix(96.0,4.0,roughness))
        * surface.properties.b * (1.0-roughness) * 0.15;
    var color = albedo*light+vec3<f32>(specular);
    if !water {
        let political = styles[ids.x].political.rgb*weights.x + styles[ids.y].political.rgb*weights.y + styles[ids.z].political.rgb*weights.z + styles[ids.w].political.rgb*weights.w;
        let political_surface = mix(political, vec3<f32>(0.93,0.90,0.81),0.24);
        color = mix(color,political_surface*(0.80+light*0.20),0.025+map_view.x*0.895);
    } else {
        color = water_surface(in.uv,in.world_position.xyz,detail_dx,detail_dy);
    }
    if !water { color *= 1.0-country_outline*0.55; }
    if !water { color *= 1.0-state_outline*(0.16+map_view.x*0.06); }
    color = mix(color,vec3<f32>(0.40,0.55,0.52),coast_outline*0.3);
    if !water { color = mix(color,vec3<f32>(0.025,0.09,0.12),river_coverage*0.85); }
    let hovered_group = styles[selection.y].grouping.w;
    if hovered_group != 0u && !water {
        let hover_coverage = coverage(hovered_group,3u,ids,weights);
        let hover_outline = boundary(hovered_group,3u,ids,weights,fraction,footprint);
        color = mix(color,vec3<f32>(1.0),hover_coverage*0.08);
        color = mix(color,vec3<f32>(1.0),hover_outline*0.8);
    }
    if selected_group != 0u {
        color = mix(color,vec3<f32>(1.0),selected_coverage*0.12);
        color = mix(color,vec3<f32>(1.0),selection_outline*0.92);
    }
    return vec4<f32>(color,1.0);
}
