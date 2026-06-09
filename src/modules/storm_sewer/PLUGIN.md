# Storm Sewer (`opencad.storm_sewer`)

Add-on package for gravity storm-drain network design and analysis.

- **Engine:** `crates/stormsewer` (headless hydraulics)
- **Host integration:** `plugin.rs`, `dispatch.rs`, `register.rs`
- **Architecture:** `docs/plugin-architecture.md`

## Commands

| Command | Description |
|---------|-------------|
| `SS_INLET` | Place inlet structure |
| `SS_JUNCTION` | Place junction structure |
| `SS_OUTFALL` | Place outfall structure |
| `SS_PIPE` | Draw pipe between two structures |
| `SS_CATCHMENT` | Draw catchment boundary polyline |
| `SS_APPLYTC` | Apply time-of-concentration from catchments |
| `SS_LANDXML` / `SS_IMPORTXML` | Import LandXML storm network |
| `SS_ANALYZE` | Run hydraulic analysis on drawn network |
| `SS_SIZE` | Size pipes from analysis |
| `SS_PARAMS` | Set rainfall / analysis parameters |
| `SS_MULTIRP` | Multi return-period analysis |
| `SS_REPORT` | Print analysis report |
| `SS_PROFILE` | Draw HGL profile |

## XDATA schemas

All records use application names registered in `manifest.rs`. Values are stored as XDATA on DWG entities so networks round-trip through save/load.

### `STORMSEWER_STRUCT` (on `CIRCLE`)

| Index | Field | Type | Notes |
|-------|-------|------|-------|
| 0 | kind | int | 0=inlet, 1=junction, 2=outfall |
| 1 | invert | real | Structure invert elevation |
| 2 | rim | real | Rim elevation |
| 3 | area | real | Contributing area (acres) |
| 4 | C | real | Runoff coefficient |

### `STORMSEWER_PIPE` (on `LINE`)

| Index | Field | Type | Notes |
|-------|-------|------|-------|
| 0 | diameter | real | Pipe diameter (inches) |
| 1 | n | real | Manning's n |
| 2 | from_handle | int | Start structure entity handle |
| 3 | to_handle | int | End structure entity handle |

### `STORMSEWER_CATCHMENT` (on `LWPOLYLINE`)

| Index | Field | Type | Notes |
|-------|-------|------|-------|
| 0 | area_acres | real | Catchment area |
| 1 | C | real | Runoff coefficient |
| 2 | length_ft | real | Flow path length |
| 3 | slope | real | Average slope |
| 4 | inlet_handle | int | Optional inlet structure handle (0 = none) |

## Per-document state

Stored under plugin id `opencad.storm_sewer` as `StormTabState` (`state.rs`):

- `StormAnalysisParams` — IDF, tailwater, min Tc, return periods, etc.

## Dependencies

- `stormsewer` workspace crate — must not depend on CAD host crates.
- This package — must not edit `src/app/commands.rs`; use `dispatch.rs`.