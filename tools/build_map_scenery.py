"""Convert authored tree placements and spline corridors into small runtime buffers.

Scenery remains visual data, not a 2026 transport or forestry simulation. Binary
outputs use a four-byte magic, little-endian version/count and float32 records.
"""
from pathlib import Path
import json
import re
import struct
import numpy as np

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets"
OUTPUT = ASSETS / "gfx/map/derived"


class SplineReader:
    """Strict reader for the scalar/container subset used by this spline v4 file.

    Scalar token values follow the Jomini binary format (rakaly/jomini lexer).
    Unknown field IDs stay numeric; unexpected scalar encodings fail the build.
    """
    def __init__(self, data):
        self.data, self.offset = data, 0

    def read(self, fmt):
        value = struct.unpack_from(fmt, self.data, self.offset)[0]
        self.offset += struct.calcsize(fmt)
        return value

    def value(self):
        token = self.read("<H")
        if token == 3:
            return self.block(True)
        formats = {12: "<i", 13: "<f", 20: "<I", 0x29c: "<Q"}
        if token in formats:
            return self.read(formats[token])
        if token not in (0xee, 0x45a, 0x5f4, 0x5f5, 0x5f6, 0x5f7, 0xb, 0x4c):
            raise ValueError(f"Unsupported spline token {token:#x} at {self.offset-2}")
        return token

    def block(self, nested=False):
        result = []
        while self.offset < len(self.data):
            if self.data[self.offset:self.offset+2] == b"\x04\x00":
                if not nested:
                    raise ValueError("Unmatched spline container")
                self.offset += 2
                return result
            value = self.value()
            if self.data[self.offset:self.offset+2] == b"\x01\x00":
                self.offset += 2
                value = (value, self.value())
            result.append(value)
        if nested:
            raise ValueError("Unclosed spline container")
        return result


def write_records(name, magic, records):
    records = np.asarray(records, dtype="<f4").reshape(-1, 4)
    if not np.isfinite(records).all():
        raise ValueError(f"Non-finite scenery in {name}")
    (OUTPUT / name).write_bytes(struct.pack("<4sII", magic, 1, len(records)) + records.tobytes())
    print(f"{name}: {len(records):,} records")


def build_forests():
    records = []
    directory = ASSETS / "gfx/map/map_object_data/generated"
    for family, kind in (("pine_dense", 0), ("pine_sparse", 0), ("oak_dense", 1),
                         ("oak_sparse", 1), ("rainforest", 1), ("rainforest_sparse", 1)):
        path = directory / f"{family}_generator_2.ron"
        if not path.is_file():
            continue
        source = path.read_text()
        transforms = []
        for name in re.findall(r'key: "transform_bin_file".*?Text\("([^"]+)"\)', source):
            transforms.extend(np.fromfile(ASSETS / name, dtype="<f4").reshape(-1, 10))
        for escaped in re.findall(r'key: "transform".*?Text\("((?:[^"\\]|\\.)*)"\)', source):
            values = np.fromstring(json.loads('"' + escaped + '"'), sep=" ")
            transforms.extend(values.reshape(-1, 10))
        for row in transforms:
            records.append((row[0]/8192, 1-row[2]/3616, max(.08, min(.25, row[7]*.26)), kind))
    write_records("forest_instances.bin", b"AGTF", records)


def build_roads():
    source = SplineReader((ASSETS / "gfx/map/spline_network/spline_network.splnet").read_bytes())
    document = dict(source.block())
    if document[0xee] != 4:
        raise ValueError("Unsupported spline version")
    nodes = {item[0xb]: item[0x4c] for item in map(dict, document[0x5f4])}
    if len(nodes) != document[0x45a][0] or len(document[0x5f5]) != document[0x45a][1]:
        raise ValueError("Spline count mismatch")
    segments = []
    for item in map(dict, document[0x5f5]):
        points = [nodes[index] for index in item[0x5f7]]
        for start, end in zip(points, points[1:]):
            segments.append((start[0]/8192, 1-start[1]/3616, end[0]/8192, 1-end[1]/3616))
    write_records("road_segments.bin", b"AGRD", segments)


if __name__ == "__main__":
    OUTPUT.mkdir(parents=True, exist_ok=True)
    build_forests()
    build_roads()
