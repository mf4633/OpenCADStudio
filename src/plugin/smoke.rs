// Integration smoke tests — plugin host + Storm Sewer consumer.

#[cfg(test)]
mod tests {
    use crate::app::OpenCADStudio;
    use crate::plugin::all_ribbon_modules;
    use crate::plugin::registry::{all_plugins, try_dispatch};

    #[test]
    fn storm_plugin_is_registered() {
        let ids: Vec<_> = all_plugins().iter().map(|p| p.manifest().id).collect();
        assert!(
            ids.contains(&"opencad.storm_sewer"),
            "expected storm plugin; got {ids:?}"
        );
    }

    #[test]
    fn storm_ribbon_tab_is_present() {
        let titles: Vec<&str> = all_ribbon_modules().iter().map(|m| m.title()).collect();
        assert!(titles.contains(&"Storm Sewer"), "ribbon tabs: {titles:?}");
    }

    #[test]
    fn ss_inlet_dispatches_to_structure_command() {
        let mut app = OpenCADStudio::new_for_test();
        assert!(try_dispatch(&mut app, 0, "SS_INLET"));
        assert_eq!(app.active_command_name(0), Some("SS_STRUCTURE"));
    }

    #[test]
    fn ss_pipe_dispatches_to_pipe_command() {
        let mut app = OpenCADStudio::new_for_test();
        assert!(try_dispatch(&mut app, 0, "SS_PIPE"));
        assert_eq!(app.active_command_name(0), Some("SS_PIPE"));
    }

    #[test]
    fn ss_unknown_prefix_not_handled() {
        let mut app = OpenCADStudio::new_for_test();
        assert!(!try_dispatch(&mut app, 0, "SS_NOTACOMMAND"));
    }
}