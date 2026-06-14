//! Plugin identity and capability declaration.
//!
//! These types now live in the standalone, dependency-free `ocs_plugin_api`
//! crate so external add-ons can target a semver-stable contract. Re-exported
//! here to keep the `crate::plugin::manifest::*` path stable for in-tree use.

pub use ocs_plugin_api::manifest::*;
