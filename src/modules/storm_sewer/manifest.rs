// Storm Sewer plugin identity (domain-specific; not part of `src/plugin/` runtime).

use crate::plugin::manifest::{ApiVersion, PluginManifest};

pub const PLUGIN_ID: &str = "opencad.storm_sewer";

pub static MANIFEST: PluginManifest = PluginManifest {
    id: PLUGIN_ID,
    name: "Storm Sewer",
    version: "0.2.0",
    description: "Gravity storm-drain network design and analysis",
    api_version: ApiVersion::CURRENT,
    ribbon_order: 50,
    xdata_apps: &["STORMSEWER_STRUCT", "STORMSEWER_PIPE", "STORMSEWER_CATCHMENT"],
    command_prefixes: &["SS_"],
};