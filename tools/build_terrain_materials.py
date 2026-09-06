"""Pack imported terrain layers for the native renderer. Requires Pillow.

The source masks and materials remain intact. Output textures are reproducible
runtime inputs; mask channels are rock, woodland, desert and snow, respectively.
"""
from pathlib import Path
import struct
from PIL import Image, ImageChops

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets/gfx/map/terrain"
DESTINATION = ROOT / "assets/gfx/map/compiled"
LAYERS = ["grasslands_01", "rocks_01", "woodlands_01", "desert_01", "snow_02"]
SIZE = 1024


def write_array(kind):
    # Standard DDS/DX10 array layout, layer-major with a complete mip chain.
    # https://learn.microsoft.com/en-us/windows/win32/direct3ddds/dds-header-dxt10
    levels = SIZE.bit_length()
    header = [124, 0x2100F, SIZE, SIZE, SIZE * 4, 0, levels] + [0] * 11
    header += [32, 4, int.from_bytes(b"DX10", "little"), 0, 0, 0, 0, 0]
    header += [0x401008, 0, 0, 0, 0]
    with (DESTINATION / f"terrain_{kind}.dds").open("wb") as output:
        output.write(b"DDS " + struct.pack("<31I", *header))
        output.write(struct.pack("<5I", 29 if kind == "diffuse" else 28, 3, 0, len(LAYERS), 4))
        for name in LAYERS:
            with Image.open(SOURCE / f"{name}_{kind}.dds") as source:
                level = source.convert("RGBA").resize((SIZE, SIZE), Image.Resampling.BOX)
            for _ in range(levels):
                output.write(level.tobytes())
                if level.width > 1:
                    level = level.resize((level.width // 2, level.height // 2), Image.Resampling.BOX)


def main():
    DESTINATION.mkdir(parents=True, exist_ok=True)
    for kind in ("diffuse", "normal"):
        write_array(kind)
    channels = []
    for family in ("rocks", "woodlands", "desert", "snow"):
        channel = Image.new("L", (4096, 1808))
        for path in sorted(SOURCE.glob(f"mask_{family}_*.png")):
            with Image.open(path) as source:
                channel = ImageChops.lighter(channel, source.convert("L").resize(channel.size, Image.Resampling.BOX))
        channels.append(channel)
    Image.merge("RGBA", channels).save(DESTINATION / "terrain_weights.png", optimize=True)
    print("Packed 5 diffuse/normal layers and 4 geographic material masks.")


if __name__ == "__main__":
    main()
