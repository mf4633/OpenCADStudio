// Module system — CadModule, ToolDef, RibbonGroup.
//
// To add a **core** ribbon tab (Home, View, …):
//   1. Create `src/modules/my_name/` directory (no `plugin.toml`)
//   2. Add `src/modules/my_name/mod.rs` implementing `CadModule` as `MyNameModule`
//   3. Add `pub mod my_name;` below
//   4. `cargo build` — tab appears via build.rs registry
//
// To add an **add-on plugin** (Storm Sewer, …):
//   See `docs/plugin-architecture.md` and copy `docs/plugin-template/`.
//
// Each module folder contains:
//   - mod.rs       : module definition (ribbon groups + tool layout)
//   - <tool>.rs    : one file per tool (ribbon def + future command logic)

// ── Ribbon vocabulary (CadModule, ToolDef, RibbonGroup, …) ─────────────────
//
// These types moved to the dependency-free `ocs_plugin_api` crate so add-ons
// can target a semver-stable contract. Re-exported here to keep the long-used
// `crate::modules::{CadModule, ToolDef, …}` paths stable across the codebase.
pub use ocs_plugin_api::ribbon::{
    CadModule, IconKind, ModuleEvent, RibbonGroup, RibbonItem, StyleKey, ToolDef,
};

// ── Module declarations ───────────────────────────────────────────────────

pub mod annotate;
pub mod home;
pub mod insert;
pub mod model;
pub mod layout;
pub mod demo_plugin;
pub mod manage;
pub mod view;

// ── Auto-generated module registry ───────────────────────────────────────
// build.rs writes this file; it contains only `all_modules()`.
pub mod registry;
