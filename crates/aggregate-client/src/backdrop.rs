//! Original procedural cartographic study, for interface composition only.
//! This is not province geometry or simulated world data.
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

#[derive(Component)]
pub struct CartographicBackdrop;

pub fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let width = 1600u32;
    let height = 1000u32;
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    let seeds = [
        (0.28, 0.25),
        (0.39, 0.4),
        (0.58, 0.26),
        (0.67, 0.42),
        (0.72, 0.61),
        (0.54, 0.64),
        (0.42, 0.76),
        (0.87, 0.35),
        (0.89, 0.71),
    ];
    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32;
            let v = y as f32 / height as f32;
            let noise = (u * 29. + (v * 15.).sin()).sin() * 0.04
                + (v * 47. + u * 21.).sin() * 0.023
                + (u * 117. + v * 73.).sin() * 0.008;
            let continent = 1. - ((u - 0.65) / 0.34).powi(2) - ((v - 0.46) / 0.68).powi(2);
            let inlet = (-((u - 0.47) / 0.12).powi(2) - ((v - 0.52) / 0.18).powi(2)).exp() * 0.85;
            let coast = continent * 0.12 + noise - inlet * 0.17;
            let grain = (((x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263))
                .wrapping_mul(1274126177)
                >> 24) as f32
                / 255.
                - 0.5)
                * 0.025;
            let mut color = if coast > 0. {
                let mut nearest = (f32::MAX, 0usize);
                let mut second = f32::MAX;
                for (index, (sx, sy)) in seeds.iter().enumerate() {
                    let distance = (u - sx).powi(2) + (v - sy).powi(2);
                    if distance < nearest.0 {
                        second = nearest.0;
                        nearest = (distance, index);
                    } else {
                        second = second.min(distance);
                    }
                }
                let hue = nearest.1 as f32 / seeds.len() as f32;
                let ridge = ((u * 45. + v * 31. + (v * 12.).sin() * 4.).sin().abs()).powi(18);
                let contour = ((coast * 750.).sin().abs() < 0.12) as u8 as f32;
                let border = if second - nearest.0 < 0.00065 {
                    0.13
                } else {
                    0.
                };
                let light = border - ridge * 0.045 - contour * 0.026;
                [
                    0.27 + hue * 0.065 + light,
                    0.32 + hue * 0.025 + light,
                    0.25 + light,
                ]
            } else {
                let coastal_line = if coast > -0.018 && ((coast * 1250.).sin().abs() < 0.16) {
                    0.035
                } else {
                    0.
                };
                [
                    0.09 + coastal_line,
                    0.18 + coastal_line,
                    0.19 + coastal_line,
                ]
            };
            if coast.abs() < 0.0009 {
                color = [0.52, 0.51, 0.37];
            }
            let grid = if x % 100 < 1 || y % 100 < 1 {
                0.022
            } else {
                0.
            };
            let vignette = (1. - 0.45 * ((u - 0.5).powi(2) + (v - 0.5).powi(2))).max(0.);
            for channel in color {
                pixels.push(((channel + grain + grid) * vignette * 255.).clamp(0., 255.) as u8);
            }
            pixels.push(255);
        }
    }
    let image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    commands.spawn((
        Sprite::from_image(images.add(image)),
        Transform::from_xyz(0., 0., -10.),
        CartographicBackdrop,
    ));
}

pub fn resize(
    window: Single<&Window>,
    mut backdrop: Single<&mut Sprite, With<CartographicBackdrop>>,
) {
    let size = Vec2::new(window.width(), window.height());
    if backdrop.custom_size != Some(size) {
        backdrop.custom_size = Some(size);
    }
}
