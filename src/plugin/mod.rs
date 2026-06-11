// Open CAD Studio plugin runtime (phase 1: built-in, in-process).
//
// Generic host only — no domain logic. See `docs/plugin-architecture.md`.
// Domain plugins (e.g. storm_sewer) live under `src/modules/<name>/` and
// register here via `inventory::submit!(PluginRegistration { … })`.

pub mod host;
pub mod manifest;
pub mod registry;

#[cfg(test)]
mod smoke;

pub use registry::all_ribbon_modules;
pub(crate) use registry::try_dispatch;