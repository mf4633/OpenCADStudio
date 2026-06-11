//! Headless tests for storm-sewer UX: structure picking, C3D-style acquisition,
//! catchment/pipe command flows (no GUI, no file I/O).

#[cfg(test)]
mod tests {
    use acadrust::entities::LwVertex;
    use acadrust::types::{Vector2, Vector3};
    use acadrust::xdata::XDataValue;
    use acadrust::{Circle, EntityType, Handle, Line, LwPolyline};
    use glam::Vec3;

    use stormsewer::network::NodeKind;

    use crate::command::{CadCommand, CmdResult};
    use crate::modules::storm_sewer::catchment::TagCatchment;
    use crate::modules::storm_sewer::data::{
        self, catchment_xdata, structure_at_point, structure_xdata, APP_CATCHMENT, APP_PIPE,
    };
    use crate::modules::storm_sewer::preview;
    use crate::modules::storm_sewer::structures::PlacePipe;
    use crate::scene::{Scene, WireModel};
    use crate::snap::{SnapType, ALL_SNAP_MODES};

    fn structure(h: u64, kind: NodeKind, x: f64, y: f64, radius: f64) -> EntityType {
        let mut e = EntityType::Circle(Circle {
            center: Vector3::new(x, y, 0.0),
            radius,
            ..Default::default()
        });
        e.common_mut().handle = Handle::new(h);
        e.common_mut()
            .extended_data
            .add_record(structure_xdata(kind, 100.0, 106.0, 1.0, 0.7));
        e
    }

    fn closed_poly(h: u64) -> EntityType {
        let mut pl = LwPolyline::default();
        pl.is_closed = true;
        pl.vertices = vec![
            LwVertex::new(Vector2::new(0.0, 0.0)),
            LwVertex::new(Vector2::new(100.0, 0.0)),
            LwVertex::new(Vector2::new(100.0, 100.0)),
            LwVertex::new(Vector2::new(0.0, 100.0)),
        ];
        let mut e = EntityType::LwPolyline(pl);
        e.common_mut().handle = Handle::new(h);
        e
    }

    fn advance_catchment_to_inlet(cmd: &mut TagCatchment, poly: EntityType) {
        cmd.inject_picked_entity(poly);
        let _ = cmd.on_entity_pick(Handle::new(1), Vec3::ZERO);
        let _ = cmd.on_enter(); // RunoffC -> FlowLength
        let _ = cmd.on_enter(); // FlowLength -> Slope
        let _ = cmd.on_enter(); // Slope -> PickInlet
    }

    // ── Structure resolution (point pick engine) ────────────────────────────

    #[test]
    fn structure_at_point_returns_kind_label_and_center() {
        let ents = vec![structure(1, NodeKind::Junction, 50.0, 25.0, 4.0)];
        let pick = structure_at_point(ents.iter(), 50.0, 25.0, 5.0, true).unwrap();
        assert_eq!(pick.handle, Handle::new(1));
        assert_eq!(pick.label(), "Junction");
        assert!((pick.x - 50.0).abs() < 1e-9);
        assert!((pick.y - 25.0).abs() < 1e-9);
    }

    #[test]
    fn structure_at_point_prefers_nearest_marker() {
        let ents = vec![
            structure(1, NodeKind::Inlet, 0.0, 0.0, 3.0),
            structure(2, NodeKind::Inlet, 40.0, 0.0, 3.0),
        ];
        let pick = structure_at_point(ents.iter(), 38.0, 0.0, 5.0, true).unwrap();
        assert_eq!(pick.handle, Handle::new(2));
    }

    #[test]
    fn catchment_pick_excludes_outfall() {
        let ents = vec![structure(9, NodeKind::Outfall, 0.0, 0.0, 6.0)];
        assert!(structure_at_point(ents.iter(), 0.0, 0.0, 20.0, false).is_none());
        assert!(data::nearest_drainage_structure_at_point(ents.iter(), 0.0, 0.0, 20.0).is_none());
    }

    #[test]
    fn pipe_pick_includes_outfall() {
        let ents = vec![structure(9, NodeKind::Outfall, 0.0, 0.0, 6.0)];
        let pick = structure_at_point(ents.iter(), 0.0, 0.0, 20.0, true).unwrap();
        assert_eq!(pick.label(), "Outfall");
    }

    // ── C3D acquisition constants ───────────────────────────────────────────

    #[test]
    fn object_pick_not_in_osnap_palette() {
        assert!(!ALL_SNAP_MODES.iter().any(|(t, _, _)| *t == SnapType::ObjectPick));
    }

    #[test]
    fn pick_highlight_color_is_orange() {
        assert!(WireModel::PICK_HIGHLIGHT[0] > 0.9);
        assert!(WireModel::PICK_HIGHLIGHT[1] > 0.4);
        assert!(WireModel::PICK_HIGHLIGHT[2] < 0.2);
    }

    #[test]
    fn pipe_rubber_band_connects_start_to_cursor() {
        let w = preview::pipe_run_rubber_band(10.0, 20.0, Vec3::new(50.0, 60.0, 0.0));
        assert_eq!(w.points.len(), 2);
        assert!((w.points[0][0] - 10.0).abs() < 1e-6);
        assert!((w.points[1][0] - 50.0).abs() < 1e-6);
        assert_eq!(w.color, WireModel::CYAN);
    }

    // ── Scene-integrated structure under cursor ─────────────────────────────

    #[test]
    fn scene_structure_under_cursor_at_marker_center() {
        let mut scene = Scene::new();
        let ent = structure(1, NodeKind::Inlet, 200.0, 150.0, 3.0);
        scene.add_entity(ent);
        let pick = preview::structure_under_cursor(&scene, 200.0, 150.0, true).unwrap();
        assert_eq!(pick.label(), "Inlet");
        let wires = preview::structure_acquire_previews(&scene, Vec3::new(200.0, 150.0, 0.0), true);
        assert!(!wires.is_empty());
        assert_eq!(wires[0].color, WireModel::PICK_HIGHLIGHT);
        assert!(wires[0].line_weight_px >= 3.0);
    }

    // ── SS_CATCHMENT command flow ───────────────────────────────────────────

    #[test]
    fn catchment_uses_structure_pick_only_on_inlet_step() {
        let mut cmd = TagCatchment::new();
        assert!(cmd.needs_entity_pick());
        assert!(!cmd.needs_structure_point_pick());

        advance_catchment_to_inlet(&mut cmd, closed_poly(10));
        assert!(!cmd.needs_entity_pick());
        assert!(cmd.needs_structure_point_pick());
        assert!(cmd.prompt().contains("orange snap"));
    }

    #[test]
    fn catchment_prompt_includes_acquisition_hint() {
        let mut cmd = TagCatchment::new();
        advance_catchment_to_inlet(&mut cmd, closed_poly(10));
        cmd.set_acquisition_hint(Some("Inlet"));
        assert!(cmd.prompt().contains("[Inlet]"));
    }

    #[test]
    fn catchment_explicit_inlet_writes_xdata_and_ends_command() {
        let mut cmd = TagCatchment::new();
        let poly = closed_poly(10);
        advance_catchment_to_inlet(&mut cmd, poly);

        match cmd.on_structure_pick(Handle::new(42), Vec3::new(10.0, 10.0, 0.0)) {
            CmdResult::ReplaceMany(replacements, additions) => {
                assert!(additions.is_empty());
                assert_eq!(replacements.len(), 1);
                let (_, ents) = &replacements[0];
                let ent = &ents[0];
                let rec = ent.common().extended_data.get_record(APP_CATCHMENT).unwrap();
                assert!(matches!(&rec.values[3], XDataValue::Handle(h) if *h == Handle::new(42)));
            }
            other => panic!("expected ReplaceMany, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn catchment_enter_at_inlet_auto_assigns_nearest() {
        let mut cmd = TagCatchment::new();
        advance_catchment_to_inlet(&mut cmd, closed_poly(10));

        match cmd.on_enter() {
            CmdResult::ReplaceMany(replacements, _) => {
                let ent = &replacements[0].1[0];
                let rec = ent.common().extended_data.get_record(APP_CATCHMENT).unwrap();
                assert!(matches!(&rec.values[3], XDataValue::Handle(h) if h.is_null()));
            }
            other => panic!("expected ReplaceMany, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn catchment_structure_pick_sets_flow_length_when_zero() {
        let mut cmd = TagCatchment::new();
        advance_catchment_to_inlet(&mut cmd, closed_poly(10));
        match cmd.on_structure_pick(Handle::new(1), Vec3::new(0.0, 0.0, 0.0)) {
            CmdResult::ReplaceMany(replacements, _) => {
                let rec = replacements[0].1[0]
                    .common()
                    .extended_data
                    .get_record(APP_CATCHMENT)
                    .unwrap();
                let flow = match &rec.values[1] {
                    XDataValue::Real(v) => *v,
                    _ => 0.0,
                };
                // Centroid (50,50) → inlet at (0,0) ≈ 70.7 ft
                assert!(flow > 50.0, "expected auto flow length, got {flow}");
            }
            other => panic!("expected ReplaceMany, got {:?}", std::mem::discriminant(&other)),
        }
    }

    // ── SS_PIPE command flow ────────────────────────────────────────────────

    #[test]
    fn pipe_uses_structure_point_pick() {
        let cmd = PlacePipe::new();
        assert!(!cmd.needs_entity_pick());
        assert!(cmd.needs_structure_point_pick());
        assert!(cmd.prompt().contains("orange snap"));
    }

    #[test]
    fn pipe_end_prompt_references_start_structure_label() {
        let mut cmd = PlacePipe::new();
        cmd.set_acquisition_hint(Some("Inlet"));
        cmd.on_structure_pick(Handle::new(1), Vec3::new(0.0, 0.0, 0.0));
        assert!(cmd.prompt().contains("from Inlet"));
    }

    #[test]
    fn pipe_commit_links_structures_in_xdata() {
        let mut cmd = PlacePipe::new();
        cmd.on_structure_pick(Handle::new(1), Vec3::new(0.0, 0.0, 0.0));
        cmd.on_structure_pick(Handle::new(2), Vec3::new(100.0, 0.0, 0.0));
        let mut cmd = PlacePipe::new();
        cmd.on_structure_pick(Handle::new(1), Vec3::new(0.0, 0.0, 0.0));
        if let CmdResult::CommitAndExit(e) = cmd.on_structure_pick(Handle::new(2), Vec3::new(100.0, 0.0, 0.0))
        {
            let rec = e.common().extended_data.get_record(APP_PIPE).unwrap();
            assert!(matches!(&rec.values[2], XDataValue::Handle(h) if *h == Handle::new(1)));
            assert!(matches!(&rec.values[3], XDataValue::Handle(h) if *h == Handle::new(2)));
        } else {
            panic!("second pick should commit pipe");
        }
    }

    #[test]
    fn pipe_preview_only_on_second_pick() {
        let mut cmd = PlacePipe::new();
        assert!(cmd.on_preview_wires(Vec3::new(10.0, 0.0, 0.0)).is_empty());
        cmd.on_structure_pick(Handle::new(1), Vec3::ZERO);
        assert_eq!(cmd.on_preview_wires(Vec3::new(10.0, 0.0, 0.0)).len(), 1);
    }

    // ── End-to-end: catchment + network assembly ────────────────────────────

    #[test]
    fn explicit_catchment_inlet_feeds_network_analysis() {
        let mut ents = vec![
            structure(1, NodeKind::Inlet, 0.0, 0.0, 3.0),
            structure(2, NodeKind::Outfall, 100.0, 0.0, 6.0),
            EntityType::Line(Line::from_points(
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(100.0, 0.0, 0.0),
            )),
        ];
        ents[2].common_mut().handle = Handle::new(3);
        ents[2]
            .common_mut()
            .extended_data
            .add_record(data::pipe_xdata(1.5, 0.013, Handle::new(1), Handle::new(2)));

        let mut poly = closed_poly(10);
        poly.common_mut()
            .extended_data
            .add_record(catchment_xdata(0.8, 2500.0, 0.02, Handle::new(1)));
        ents.push(poly);

        let net = data::network_from_entities(ents.iter()).unwrap();
        assert!(net.nodes[0].area_ac > 1.0, "catchment area should merge onto inlet");
        assert!(
            net.nodes[0].tc_inlet > 10.0,
            "Kirpich tc should exceed default, got {}",
            net.nodes[0].tc_inlet
        );
    }

    // Integration: XDATA roundtrip + headless analyze consistency (review Issue 11).
    // In-mem ents + XDATA; exercises network_from + analyze_doc + report parity.
    #[test]
    fn xdata_roundtrip_and_analyze_consistency() {
        use crate::modules::storm_sewer::analysis;
        use crate::modules::storm_sewer::data::{pipe_xdata, structure_xdata};
        use acadrust::types::Vector3;
        use acadrust::{Circle, Line};
        use stormsewer::params::StormAnalysisParams;
        let mut s1 = EntityType::Circle(Circle { center: Vector3::new(0.0, 0.0, 0.0), radius: 3.0, ..Default::default() });
        s1.common_mut().handle = Handle::new(1);
        s1.common_mut().extended_data.add_record(structure_xdata(NodeKind::Inlet, 100.0, 106.0, 1.0, 0.7));
        let mut s2 = EntityType::Circle(Circle { center: Vector3::new(100.0, 0.0, 0.0), radius: 3.0, ..Default::default() });
        s2.common_mut().handle = Handle::new(2);
        s2.common_mut().extended_data.add_record(structure_xdata(NodeKind::Outfall, 99.0, 104.0, 0.0, 0.0));
        let mut p = EntityType::Line(Line::from_points(Vector3::new(0.0, 0.0, 0.0), Vector3::new(100.0, 0.0, 0.0)));
        p.common_mut().handle = Handle::new(3);
        p.common_mut().extended_data.add_record(pipe_xdata(1.5, 0.013, Handle::new(1), Handle::new(2)));
        let ents = vec![s1, s2, p];
        let params = StormAnalysisParams::municipal();
        let (annots, report, _a) = analysis::analyze_doc(ents.iter(), &params).expect("analyze from XDATA ents");
        assert!(!report.is_empty(), "report should be produced");
        assert!(report.contains("Q") || report.contains("flow"), "report should mention flow/Q");
        let net2 = data::network_from_entities(ents.iter()).expect("re-parse net from XDATA");
        assert_eq!(net2.nodes.len(), 2);
        assert_eq!(net2.pipes.len(), 1);
    }
}