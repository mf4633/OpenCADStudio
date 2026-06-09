// Compile-time plugin registry via `inventory`.

use super::host::{BuiltinPlugin, HostSession};
use crate::app::OpenCADStudio;
use crate::modules::{registry as core_registry, CadModule};

pub struct PluginRegistration {
    pub construct: fn() -> Box<dyn BuiltinPlugin>,
}

inventory::collect!(PluginRegistration);

/// Construct every registered built-in plugin (once per process).
pub fn all_plugins() -> Vec<Box<dyn BuiltinPlugin>> {
    inventory::iter::<PluginRegistration>
        .into_iter()
        .map(|r| (r.construct)())
        .collect()
}

/// Core ribbon tabs plus add-on tabs (sorted by `manifest.ribbon_order`).
pub fn all_ribbon_modules() -> Vec<Box<dyn CadModule>> {
    let mut core = core_registry::all_modules();
    let mut addons: Vec<(i32, Box<dyn CadModule>)> = all_plugins()
        .into_iter()
        .map(|p| (p.manifest().ribbon_order, p.ribbon()))
        .collect();
    addons.sort_by_key(|(order, _)| *order);
    core.extend(addons.into_iter().map(|(_, ribbon)| ribbon));
    core
}

/// Try each plugin until one handles `cmd`. Returns true if handled.
pub(crate) fn try_dispatch(app: &mut OpenCADStudio, tab: usize, cmd: &str) -> bool {
    let mut host = HostSession::new(app, tab);
    for plugin in all_plugins() {
        if plugin.dispatch(&mut host, cmd) {
            return true;
        }
    }
    false
}