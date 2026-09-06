# 2026 population source

`population.json` contains 237 country/area population projections for **1 January 2026**, extracted from the medium variant of **World Population Prospects 2024**. These are projections, not observed 2026 census counts.

Source: United Nations, Department of Economic and Social Affairs, Population Division (2024). *World Population Prospects 2024, Online Edition*.

- [Publisher](https://population.un.org/wpp/)
- [Source workbook](https://population.un.org/wpp/assets/Excel%20Files/1_Indicator%20(Standard)/EXCEL_FILES/1_General/WPP2024_GEN_F01_DEMOGRAPHIC_INDICATORS_COMPACT.xlsx)
- © July 2024 United Nations. The workbook identifies its license as [Creative Commons Attribution 3.0 IGO](https://creativecommons.org/licenses/by/3.0/igo/).

Aggregate extracted ISO3 codes, country/area labels and January population, converted thousands of people to integer people with decimal rounding, and sorted records by ISO3. The source workbook SHA-256 and provenance are recorded in the JSON. Reproduce with `python tools/import_world_population.py <workbook.xlsx>` (requires `openpyxl`).

This dataset is staged for world initialization; the current game setup still uses authored per-province sandbox population. It provides neither subnational allocation nor complete coverage of the map's territory identifiers. Runtime integration must explicitly address missing areas and distinguish national projections from any estimated provincial distribution.
