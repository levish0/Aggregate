// Shared by cloud color and terrain shadow so their shapes and motion stay aligned.
struct CloudField {
    density: f32,
    primary_uv: vec2<f32>,
    secondary_uv: vec2<f32>,
};

fn weather_hash(cell: vec2<f32>) -> f32 {
    // Eight weather cells around the world; exact horizontal wrap across map copies.
    let wrapped = vec2<f32>((cell.x%8.0+8.0)%8.0,cell.y);
    return fract(sin(dot(wrapped,vec2<f32>(127.1,311.7)))*43758.5453);
}

fn weather_coverage(point: vec2<f32>) -> f32 {
    let cell = floor(point);
    let fraction = fract(point);
    let blend = fraction*fraction*(3.0-2.0*fraction);
    return mix(mix(weather_hash(cell),weather_hash(cell+vec2<f32>(1.0,0.0)),blend.x),
        mix(weather_hash(cell+vec2<f32>(0.0,1.0)),weather_hash(cell+vec2<f32>(1.0,1.0)),blend.x),blend.y);
}

fn cloud_field(world: vec2<f32>, width: f32, time: f32, density_map: texture_2d<f32>, density_sampler: sampler) -> CloudField {
    let normalized = world/width;
    let primary_uv = normalized*18.0+time*vec2<f32>(0.0003,0.00008);
    // Different scale, rotation and wind. Integer horizontal periods retain the map seam.
    let rotated = vec2<f32>(normalized.x*0.6+normalized.y*0.8,-normalized.x*0.8+normalized.y*0.6);
    let secondary_uv = rotated*25.0+vec2<f32>(0.37,0.61)+time*vec2<f32>(-0.00013,0.00018);
    let primary = textureSample(density_map,density_sampler,primary_uv).g;
    let secondary = textureSample(density_map,density_sampler,secondary_uv).g;
    let coverage = weather_coverage(normalized*8.0+time*vec2<f32>(0.00005,0.00002));
    return CloudField((primary*0.8+secondary*0.45)*(0.55+coverage*0.9),primary_uv,secondary_uv);
}
