// Compile-time registration with the generic plugin host.
// Keep this file free of storm-sewer logic — only the hook.

use super::plugin::StormSewerPlugin;

inventory::submit! {
    crate::plugin::registry::PluginRegistration {
        construct: || Box::new(StormSewerPlugin),
    }
}