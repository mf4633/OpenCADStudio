// Storm Sewer command handlers — all domain logic lives here, not in `src/plugin/`.

use crate::command::CadCommand;
use crate::plugin::host::HostSession;

use super::analysis;
use super::catchment::{apply_tc_from_network, TagCatchment};
use super::landxml_import;
use super::manifest::PLUGIN_ID;
use super::params_cmd;
use super::sizing;
use super::state::StormTabState;
use super::structures::{PlacePipe, PlaceStructure};
use super::{data, style};

fn tab_params(host: &mut HostSession<'_>) -> stormsewer::params::StormAnalysisParams {
    host.ensure_plugin_state(PLUGIN_ID, StormTabState::default)
        .params()
        .clone()
}

/// Handle any `SS_*` command. Returns true when consumed.
pub fn handle(host: &mut HostSession<'_>, cmd: &str) -> bool {
    if !cmd.starts_with("SS_") {
        return false;
    }

    match cmd {
        "SS_ANALYZE" => {
            let params = tab_params(host);
            match analysis::analyze_doc(host.entities(), &params) {
                Ok((ents, report, analysis)) => {
                    for e in ents {
                        let _ = host.add_entity(e);
                    }
                    if let Ok(drawn) = data::drawn_network_from_entities(host.entities()) {
                        host.push_undo("SS_STYLE");
                        let (sur, flood) = style::apply_analysis_style(host.entities_mut(), &drawn, &analysis);
                        if sur > 0 || flood > 0 {
                            host.set_dirty();
                            host.push_info(&format!(
                                "Styled {sur} surcharged pipe(s), {flood} flooded structure(s)."
                            ));
                        }
                    }
                    host.bump_geometry();
                    host.push_info(&format!("Storm sewer analyzed ({}).", params.summary()));
                    for line in report.lines() {
                        host.push_output(line);
                    }
                }
                Err(e) => host.push_error(&e),
            }
            true
        }
        "SS_REPORT" => {
            let params = tab_params(host);
            match analysis::report_doc(host.entities(), &params) {
                Ok(report) => {
                    for line in report.lines() {
                        host.push_output(line);
                    }
                }
                Err(e) => host.push_error(&e),
            }
            true
        }
        "SS_MULTIRP" => {
            let params = tab_params(host);
            match analysis::multi_rp_report(host.entities(), &params) {
                Ok(report) => {
                    for line in report.lines() {
                        host.push_output(line);
                    }
                }
                Err(e) => host.push_error(&e),
            }
            true
        }
        "SS_PROFILE" => {
            let params = tab_params(host);
            match analysis::profile_doc(host.entities(), &params) {
                Ok(ents) => {
                    for e in ents {
                        let _ = host.add_entity(e);
                    }
                    host.bump_geometry();
                    host.push_info("Storm sewer HGL profile drawn.");
                }
                Err(e) => host.push_error(&e),
            }
            true
        }
        "SS_SIZE" => {
            let params = tab_params(host);
            match sizing::plan_size_updates(host.entities(), &params) {
                Ok((updates, report, pending)) => {
                    for line in report.lines() {
                        host.push_output(line);
                    }
                    if pending == 0 {
                        host.push_info("Storm sewer: all pipes already meet sizing criteria.");
                    } else {
                        host.push_undo("SS_SIZE");
                        let applied = sizing::apply_updates(host.entities_mut(), &updates);
                        host.bump_geometry();
                        host.set_dirty();
                        host.push_info(&format!("Storm sewer: applied {applied} pipe diameter update(s)."));
                    }
                }
                Err(e) => host.push_error(&e),
            }
            true
        }
        "SS_INLET" | "SS_JUNCTION" | "SS_OUTFALL" => {
            let c = match cmd {
                "SS_INLET" => PlaceStructure::inlet(),
                "SS_JUNCTION" => PlaceStructure::junction(),
                _ => PlaceStructure::outfall(),
            };
            host.push_info(&c.prompt());
            host.set_active_command(Box::new(c));
            true
        }
        "SS_PIPE" => {
            let c = PlacePipe::new();
            host.push_info(&c.prompt());
            host.set_active_command(Box::new(c));
            true
        }
        "SS_CATCHMENT" => {
            let c = TagCatchment::new();
            host.push_info(&c.prompt());
            host.set_active_command(Box::new(c));
            true
        }
        "SS_APPLYTC" => {
            host.push_undo("SS_APPLYTC");
            let snapshot: Vec<_> = host.entities().cloned().collect();
            match apply_tc_from_network(snapshot.iter(), host.entities_mut()) {
                Ok(n) => {
                    host.set_dirty();
                    host.bump_geometry();
                    host.push_info(&format!("Storm sewer: updated inlet Tc on {n} structure(s)."));
                }
                Err(e) => host.push_error(&e),
            }
            true
        }
        "SS_LANDXML" | "SS_IMPORTXML" => {
            match landxml_import::pick_landxml_file() {
                None => host.push_info("LandXML import cancelled."),
                Some(Ok(xml)) => match landxml_import::import_landxml(host, &xml) {
                    Ok(msg) => host.push_info(&msg),
                    Err(e) => host.push_error(&e),
                },
                Some(Err(e)) => host.push_error(&e),
            }
            true
        }
        cmd if cmd == "SS_PARAMS" || cmd.starts_with("SS_PARAMS ") => {
            let rest = cmd.trim_start_matches("SS_PARAMS").trim();
            let state = host.ensure_plugin_state(PLUGIN_ID, StormTabState::default);
            match params_cmd::apply_params(state, rest) {
                Ok(msg) => host.push_info(&msg),
                Err(e) => host.push_error(&e),
            }
            true
        }
        _ => false,
    }
}