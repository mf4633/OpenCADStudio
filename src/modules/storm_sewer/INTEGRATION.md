# Storm Sewer module — integration (see authoritative docs)

**This file is superseded / archived per review Issue 4 (2026-06-11).**

See:
- `docs/plugin-architecture.md` (current spec: plugin.toml exclusion in build.rs, `plugin::try_dispatch` in commands.rs:193, BuiltinPlugin + inventory, HostSession, XDATA on entities, no edits to core commands.rs).
- `src/modules/storm_sewer/PLUGIN.md` (XDATA schemas, command reference).
- `src/modules/storm_sewer/dispatch.rs` (all SS_* routing + interactive CadCommand impls).
- `src/modules/storm_sewer/data.rs` (XDATA read/write + network_from_entities + replace_xdata_record helper).
- `src/modules/storm_sewer/analysis.rs` + `headless.rs` (engine bridge + headless tests).

Old direct `commands.rs::dispatch_command` wiring for SS_* is **gone**; plugins route first via `if crate::plugin::try_dispatch(...) { return; }`.

`build.rs` auto-discovers only dirs **without** `plugin.toml` (storm_sewer excluded; registers via its BuiltinPlugin::ribbon).

For adding similar plugins: copy `docs/plugin-template/`, implement dispatch + manifest + register.rs, update `src/modules/mod.rs`.

Storm Sewer demonstrates: placement/snapping/analyze (Rational+Manning+HGL), XDATA roundtrip, apply sizing/Tc, LandXML, headless smoke, WASM engine export (see crates/stormsewer).

(Original content archived in git history.)
  and free outfall; add an `SS_PARAMS` command to set IDF / tailwater / min-Tc.
- **Edit command** — `SS_EDIT` to change a placed structure/pipe's values.
- **Surcharge styling** — recolor surcharged pipes / flag flooded structures.
- **Persistence check** — verify the StormSewer XDATA round-trips through
  DWG/DXF save+reload (acadrust supports XDATA; confirm end-to-end).

## Build

`build.rs` auto-discovers this directory (`storm_sewer/` → `StormSewerModule`)
and regenerates `src/modules/registry.rs`, so the tab appears on `cargo build`.
