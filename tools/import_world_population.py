"""Extract the 2026-01-01 UN WPP medium projection from its official workbook.

Usage: python tools/import_world_population.py path/to/WPP2024_GEN_F01_DEMOGRAPHIC_INDICATORS_COMPACT.xlsx
Requires openpyxl. No network access is performed by this importer.
"""
import hashlib
import json
from decimal import Decimal, ROUND_HALF_UP
from pathlib import Path
import sys
import openpyxl

SOURCE_URL = "https://population.un.org/wpp/assets/Excel%20Files/1_Indicator%20(Standard)/EXCEL_FILES/1_General/WPP2024_GEN_F01_DEMOGRAPHIC_INDICATORS_COMPACT.xlsx"


def main():
    source = Path(sys.argv[1])
    workbook = openpyxl.load_workbook(source, read_only=True, data_only=True)
    countries = {}
    for row in workbook["Medium variant"].iter_rows(min_row=18, values_only=True):
        if row[10] != 2026 or not row[5]:
            continue
        iso3 = str(row[5]).strip()
        if iso3 in countries:
            raise ValueError(f"Duplicate source country: {iso3}")
        countries[iso3] = {
            "name": row[2],
            "population": int((Decimal(str(row[11])) * 1000).quantize(Decimal(1), rounding=ROUND_HALF_UP)),
        }
    if not 230 <= len(countries) <= 250:
        raise ValueError(f"Unexpected country coverage: {len(countries)}")
    output = {
        "schema_version": 1, "date": "2026-01-01", "source_revision": "UN WPP 2024 / medium variant",
        "source_url": SOURCE_URL, "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "license": "CC BY 3.0 IGO", "unit": "persons", "kind": "projection",
        "countries": dict(sorted(countries.items())),
    }
    destination = Path(__file__).resolve().parents[1] / "assets/world/2026/population.json"
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(output, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Imported {len(countries)} country/area projections; Korea={countries['KOR']['population']}")


if __name__ == "__main__":
    main()
