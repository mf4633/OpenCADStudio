// Thin adapter: implements the generic `BuiltinPlugin` trait for Storm Sewer.
// Domain logic is in `dispatch.rs`; identity in `manifest.rs`; hook in `register.rs`.

use crate::plugin::host::{BuiltinPlugin, HostSession};
use crate::plugin::manifest::PluginManifest;

use super::dispatch;
use super::manifest;

pub struct StormSewerPlugin;

impl BuiltinPlugin for StormSewerPlugin {
    fn manifest(&self) -> &'static PluginManifest {
        &manifest::MANIFEST
    }

    fn ribbon(&self) -> Box<dyn crate::modules::CadModule> {
        Box::new(super::StormSewerModule)
    }

    fn dispatch(&self, host: &mut HostSession<'_>, cmd: &str) -> bool {
        dispatch::handle(host, cmd)
    }
}