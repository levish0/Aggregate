"""Bake world-space normals and local horizon occlusion from the source heightmap.

This is an offline asset build, not work performed while opening the map. Requires
Pillow and NumPy. RGB stores a world normal; alpha stores ambient visibility.
"""
from pathlib import Path
import re
import struct
import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
WIDTH = 4096


def main():
    settings = (ROOT / "assets/map_data/terrain_settings.ron").read_text()
    values = dict(re.findall(r"(world_width|height_scale|sea_level_raw):\s*([\d.]+)", settings))
    world_width, height_scale, sea_level = (float(values[key]) for key in
                                           ("world_width", "height_scale", "sea_level_raw"))
    # The trusted, local source is 16384 x 7232, above Pillow's default warning limit.
    Image.MAX_IMAGE_PIXELS = 16384 * 16384
    with Image.open(ROOT / "assets/map_data/heightmap.png") as source:
        height = round(WIDTH * source.height / source.width)
        elevation = np.asarray(source.convert("F").resize((WIDTH, height), Image.Resampling.BOX))
    elevation = np.maximum(elevation - sea_level, 0) * (height_scale / 65535)
    spacing = world_width / WIDTH
    dz, dx = np.gradient(elevation, spacing)
    normal = np.stack((-dx, np.ones_like(dx), -dz), axis=-1)
    normal /= np.linalg.norm(normal, axis=-1, keepdims=True)
    horizon = np.zeros_like(elevation)
    # Clamp at the poles and wrap at the date line, matching the world topology.
    for row_offset, column_offset in ((0, 1), (0, -1), (1, 0), (-1, 0)):
        direction = np.zeros_like(elevation)
        for radius in (2, 8, 24):
            rows = np.clip(np.arange(height) + row_offset * radius, 0, height - 1)
            neighbor = np.roll(elevation[rows], column_offset * radius, axis=1)
            direction = np.maximum(direction, (neighbor - elevation) / (radius * spacing))
        horizon += np.arctan(direction) / (np.pi * 2)
    rgba = np.concatenate((normal * .5 + .5, np.clip(1 - horizon, .35, 1)[..., None]), axis=-1)
    level = Image.fromarray(np.round(rgba * 255).astype(np.uint8))
    levels = max(WIDTH, height).bit_length()
    header = [124, 0x2100F, height, WIDTH, WIDTH * 4, 0, levels] + [0] * 11
    header += [32, 0x41, 0, 32, 0xff, 0xff00, 0xff0000, 0xff000000]
    header += [0x401008, 0, 0, 0, 0]
    destination = ROOT / "assets/gfx/map/derived/terrain_relief.dds"
    with destination.open("wb") as output:
        output.write(b"DDS " + struct.pack("<31I", *header))
        for _ in range(levels):
            pixels = np.asarray(level).astype(np.float32) / 255
            vectors = pixels[..., :3] * 2 - 1
            vectors /= np.maximum(np.linalg.norm(vectors, axis=-1, keepdims=True), 1e-6)
            pixels[..., :3] = vectors * .5 + .5
            output.write(np.round(pixels * 255).astype(np.uint8).tobytes())
            level = level.resize((max(1, level.width // 2), max(1, level.height // 2)), Image.Resampling.BOX)
    print(f"Baked {WIDTH} x {height} relief, {levels} mip levels: {destination.stat().st_size:,} bytes")


if __name__ == "__main__":
    main()
