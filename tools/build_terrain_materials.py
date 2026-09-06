"""Pack imported terrain layers for the native renderer. Requires Pillow and NumPy.

The source masks and materials remain intact. Output textures are reproducible
runtime inputs. Material array order follows materials.ron; four-channel detail
indices and coverage preserve the authored geographic placement.
"""
from pathlib import Path
import struct
import json
import re
import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets/gfx/map/terrain"
DESTINATION = ROOT / "assets/gfx/map/compiled"
SIZE = 512


def materials():
    # Read the converted manifest's flat material records, preserving source order.
    source = (SOURCE / "materials.ron").read_text(encoding="utf-8")
    records = []
    for block in source.split("Item(Block([")[2:]:
        fields = dict(re.findall(r'Property\(key: "([^"]+)", operator: Assign, value: Text\("([^"]+)"\)\)', block))
        if "diffuse" in fields:
            records.append(fields)
    if not records or any(not (SOURCE / record[k]).is_file() for record in records for k in ("diffuse", "normal")):
        raise ValueError("Invalid terrain material manifest")
    return records


def write_array(kind, records):
    # Standard DDS/DX10 array layout, layer-major with a complete mip chain.
    # https://learn.microsoft.com/en-us/windows/win32/direct3ddds/dds-header-dxt10
    levels = SIZE.bit_length()
    header = [124, 0x2100F, SIZE, SIZE, SIZE * 4, 0, levels] + [0] * 11
    header += [32, 4, int.from_bytes(b"DX10", "little"), 0, 0, 0, 0, 0]
    header += [0x401008, 0, 0, 0, 0]
    with (DESTINATION / f"terrain_{kind}.dds").open("wb") as output:
        output.write(b"DDS " + struct.pack("<31I", *header))
        output.write(struct.pack("<5I", 29 if kind == "diffuse" else 28, 3, 0, len(records), 4))
        for record in records:
            with Image.open(SOURCE / record[kind]) as source:
                level = source.convert("RGBA").resize((SIZE, SIZE), Image.Resampling.BOX)
            for _ in range(levels):
                output.write(level.tobytes())
                if level.width > 1:
                    level = level.resize((level.width // 2, level.height // 2), Image.Resampling.BOX)


def main():
    DESTINATION.mkdir(parents=True, exist_ok=True)
    records = materials()
    with Image.open(SOURCE / "detail_index.tga") as image:
        indices = np.asarray(image.convert("RGBA"))
    with Image.open(SOURCE / "detail_intensity.tga") as image:
        weights = np.asarray(image.convert("RGBA"))
    if indices.shape != weights.shape or np.any(indices[weights > 0] >= len(records)):
        raise ValueError("Detail indices do not match the material manifest")
    # Unused source channels use 255 sentinels; canonicalize before GPU lookup.
    indices = np.where(weights > 0, indices, 0).astype(np.uint8)
    Image.fromarray(indices).save(DESTINATION / "terrain_indices.png", optimize=True)
    Image.fromarray(weights).save(DESTINATION / "terrain_weights.png", optimize=True)
    for kind in ("diffuse", "normal"):
        write_array(kind, records)
    (DESTINATION / "materials.json").write_text(json.dumps(records, indent=2) + "\n", encoding="utf-8")
    print(f"Packed {len(records)} material layers and original four-channel detail placement.")


if __name__ == "__main__":
    main()
