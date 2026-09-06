use ab_glyph::{Font, FontRef, ScaleFont, point};
use bevy::{asset::RenderAssetUsages, prelude::Image, render::render_resource::{Extent3d, TextureDimension, TextureFormat}};

/// Rasterized once off the render thread, then linear-filtered on the map surface.
pub fn rasterize(font: &FontRef<'_>, text: &str) -> Image {
    let scaled = font.as_scaled(96.);
    let mut pen = 8.;
    let mut previous = None;
    let mut glyphs = Vec::new();
    for character in text.chars() {
        let id = scaled.glyph_id(character);
        if let Some(previous) = previous { pen += scaled.kern(previous, id); }
        let glyph = id.with_scale_and_position(96., point(pen, 8. + scaled.ascent()));
        pen += scaled.h_advance(id) + 6.;
        if let Some(outline) = font.outline_glyph(glyph) { glyphs.push(outline); }
        previous = Some(id);
    }
    let width = (pen + 8.).ceil() as u32;
    let height = 112;
    let mut pixels = vec![255; (width * height * 4) as usize];
    for pixel in pixels.chunks_exact_mut(4) { pixel[3] = 0; }
    for glyph in glyphs {
        let bounds = glyph.px_bounds();
        glyph.draw(|x, y, coverage| {
            let x = x as i32 + bounds.min.x as i32;
            let y = y as i32 + bounds.min.y as i32;
            if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                let alpha = &mut pixels[((y as u32 * width + x as u32) * 4 + 3) as usize];
                *alpha = (*alpha).max((coverage * 255.).round() as u8);
            }
        });
    }
    let mut image = Image::new(Extent3d { width, height, depth_or_array_layers: 1 }, TextureDimension::D2, pixels, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD);
    image.sampler = bevy::image::ImageSampler::linear();
    image
}
