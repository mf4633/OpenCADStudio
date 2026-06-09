// Interactive storm-sewer drafting commands.
//
// Robust placement: a canvas click ALWAYS places a structure (with the values
// typed so far, defaults for the rest), so dropping a structure works even if
// command-line text entry is unavailable. If typed entry works, enter
// invert/rim/area/C first and then click — the click uses those values.
//
// Pipes are pick-first: click the START structure then the END structure; the
// pipe commits on the second click with the current diameter/n.

use acadrust::types::Vector3;
use acadrust::{Circle, EntityType, Handle, Line};
use glam::Vec3;

use stormsewer::network::NodeKind;

use super::data;
use super::preview;
use crate::command::{CadCommand, CmdResult, ObjectPickHit};
use crate::scene::{Scene, WireModel};

fn parse_num(text: &str) -> Option<f64> {
    text.trim().replace(',', ".").parse::<f64>().ok()
}

// ── Structure placement ─────────────────────────────────────────────────────

enum SStep {
    Invert,
    Rim,
    Area,
    C,
    Ready,
}

pub struct PlaceStructure {
    kind: NodeKind,
    radius: f64,
    invert: f64,
    rim: f64,
    area: f64,
    c: f64,
    step: SStep,
}

impl PlaceStructure {
    pub fn inlet() -> Self {
        Self::new(NodeKind::Inlet, 3.0)
    }
    pub fn junction() -> Self {
        Self::new(NodeKind::Junction, 4.0)
    }
    pub fn outfall() -> Self {
        Self::new(NodeKind::Outfall, 6.0)
    }
    fn new(kind: NodeKind, radius: f64) -> Self {
        Self { kind, radius, invert: 100.0, rim: 105.0, area: 1.0, c: 0.70, step: SStep::Invert }
    }
    fn commit(&self, x: f64, y: f64) -> CmdResult {
        let circ = Circle { center: Vector3::new(x, y, 0.0), radius: self.radius, ..Default::default() };
        let mut ent = EntityType::Circle(circ);
        let (area, c) = if self.kind == NodeKind::Outfall { (0.0, 0.0) } else { (self.area, self.c) };
        ent.common_mut()
            .extended_data
            .add_record(data::structure_xdata(self.kind, self.invert, self.rim, area, c));
        CmdResult::CommitAndExit(ent)
    }
}

impl CadCommand for PlaceStructure {
    fn name(&self) -> &'static str {
        "SS_STRUCTURE"
    }
    fn prompt(&self) -> String {
        match self.step {
            SStep::Invert => format!("Storm {}: invert <{:.2}> (or click to place):", data::kind_str(self.kind), self.invert),
            SStep::Rim => format!("Rim <{:.2}> (or click to place):", self.rim),
            SStep::Area => format!("Drainage area, ac <{:.2}> (or click to place):", self.area),
            SStep::C => format!("Runoff C <{:.2}> (or click to place):", self.c),
            SStep::Ready => "Click to place the structure:".into(),
        }
    }
    fn wants_text_input(&self) -> bool {
        !matches!(self.step, SStep::Ready)
    }
    fn on_text_input(&mut self, text: &str) -> Option<CmdResult> {
        let v = parse_num(text);
        match self.step {
            SStep::Invert => {
                if let Some(x) = v {
                    self.invert = x;
                }
                self.step = SStep::Rim;
            }
            SStep::Rim => {
                if let Some(x) = v {
                    self.rim = x;
                }
                self.step = if self.kind == NodeKind::Outfall { SStep::Ready } else { SStep::Area };
            }
            SStep::Area => {
                if let Some(x) = v {
                    self.area = x;
                }
                self.step = SStep::C;
            }
            SStep::C => {
                if let Some(x) = v {
                    self.c = x;
                }
                self.step = SStep::Ready;
            }
            SStep::Ready => {}
        }
        None
    }
    fn on_point(&mut self, pt: Vec3) -> CmdResult {
        self.commit(pt.x as f64, pt.y as f64)
    }
    fn on_enter(&mut self) -> CmdResult {
        CmdResult::NeedPoint
    }
}

// ── Pipe placement (pick-first, so clicks always work) ──────────────────────

enum PStep {
    PickStart,
    PickEnd,
}

pub struct PlacePipe {
    step: PStep,
    diameter: f64,
    n: f64,
    start_handle: Option<Handle>,
    start_xy: (f64, f64),
    start_label: Option<String>,
    acquire_hint: Option<String>,
}

impl PlacePipe {
    pub fn new() -> Self {
        Self {
            step: PStep::PickStart,
            diameter: 1.25,
            n: 0.013,
            start_handle: None,
            start_xy: (0.0, 0.0),
            start_label: None,
            acquire_hint: None,
        }
    }
    fn commit(&self, end_handle: Handle, ex: f64, ey: f64) -> CmdResult {
        let line = Line::from_points(
            Vector3::new(self.start_xy.0, self.start_xy.1, 0.0),
            Vector3::new(ex, ey, 0.0),
        );
        let mut ent = EntityType::Line(line);
        let from = self.start_handle.unwrap_or(Handle::new(0));
        ent.common_mut().extended_data.add_record(data::pipe_xdata(self.diameter, self.n, from, end_handle));
        CmdResult::CommitAndExit(ent)
    }
}

impl Default for PlacePipe {
    fn default() -> Self {
        Self::new()
    }
}

impl CadCommand for PlacePipe {
    fn name(&self) -> &'static str {
        "SS_PIPE"
    }
    fn prompt(&self) -> String {
        let hint = self
            .acquire_hint
            .as_deref()
            .map(|h| format!(" [{h}]"))
            .unwrap_or_default();
        match self.step {
            PStep::PickStart => format!("Pipe run: click START structure (orange snap){hint}:"),
            PStep::PickEnd => {
                let from = self
                    .start_label
                    .as_deref()
                    .unwrap_or("structure");
                format!(
                    "Pipe run: click END from {from} (dia {:.2} ft, n {:.3}){hint}:",
                    self.diameter, self.n
                )
            }
        }
    }
    fn needs_entity_pick(&self) -> bool {
        false
    }
    fn needs_structure_point_pick(&self) -> bool {
        true
    }
    fn resolve_object_pick(&self, scene: &Scene, x: f64, y: f64) -> Option<ObjectPickHit> {
        let pick = preview::structure_under_cursor(scene, x, y, false)?;
        Some(ObjectPickHit {
            handle: pick.handle,
            x: pick.x,
            y: pick.y,
            label: pick.label(),
        })
    }
    fn object_pick_hover_previews(&self, scene: &Scene, cursor: Vec3) -> Vec<WireModel> {
        preview::structure_acquire_previews(scene, cursor, false)
    }
    fn object_pick_miss_message(&self) -> &'static str {
        "No storm structure near click — move closer or press Enter for nearest."
    }
    fn set_acquisition_hint(&mut self, hint: Option<&str>) {
        self.acquire_hint = hint.map(str::to_string);
    }
    fn on_structure_pick(&mut self, handle: Handle, pt: Vec3) -> CmdResult {
        match self.step {
            PStep::PickStart => {
                self.start_handle = Some(handle);
                self.start_xy = (pt.x as f64, pt.y as f64);
                self.start_label = self.acquire_hint.clone();
                self.acquire_hint = None;
                self.step = PStep::PickEnd;
                CmdResult::NeedPoint
            }
            PStep::PickEnd => self.commit(handle, pt.x as f64, pt.y as f64),
        }
    }
    fn on_preview_wires(&mut self, pt: Vec3) -> Vec<WireModel> {
        if matches!(self.step, PStep::PickEnd) {
            vec![preview::pipe_run_rubber_band(self.start_xy.0, self.start_xy.1, pt)]
        } else {
            vec![]
        }
    }
    fn on_point(&mut self, _pt: Vec3) -> CmdResult {
        CmdResult::NeedPoint
    }
    fn on_enter(&mut self) -> CmdResult {
        CmdResult::NeedPoint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_places_structure_with_defaults() {
        let mut cmd = PlaceStructure::inlet();
        match cmd.on_point(Vec3::new(10.0, 20.0, 0.0)) {
            CmdResult::CommitAndExit(EntityType::Circle(c)) => {
                assert_eq!(c.center.x, 10.0);
                let e = EntityType::Circle(c);
                assert!(e.common().extended_data.get_record(data::APP_STRUCT).is_some());
            }
            _ => panic!("expected CommitAndExit(Circle)"),
        }
    }

    #[test]
    fn typed_values_are_captured_then_click_places() {
        let mut cmd = PlaceStructure::inlet();
        assert!(cmd.on_text_input("104").is_none());
        assert!(cmd.on_text_input("110").is_none());
        assert!(cmd.on_text_input("2.0").is_none());
        assert!(cmd.on_text_input("0.8").is_none());
        assert!(matches!(cmd.on_point(Vec3::ZERO), CmdResult::CommitAndExit(_)));
    }

    #[test]
    fn pipe_connects_two_structures_on_two_clicks() {
        let mut cmd = PlacePipe::new();
        assert!(cmd.needs_structure_point_pick());
        assert!(matches!(cmd.on_structure_pick(Handle::new(1), Vec3::new(0.0, 0.0, 0.0)), CmdResult::NeedPoint));
        match cmd.on_structure_pick(Handle::new(2), Vec3::new(100.0, 0.0, 0.0)) {
            CmdResult::CommitAndExit(EntityType::Line(l)) => {
                assert_eq!(l.start.x, 0.0);
                assert_eq!(l.end.x, 100.0);
                let e = EntityType::Line(l);
                assert!(e.common().extended_data.get_record(data::APP_PIPE).is_some());
            }
            _ => panic!("expected CommitAndExit(Line)"),
        }
    }

    #[test]
    fn pipe_preview_rubber_band_on_second_step() {
        let mut cmd = PlacePipe::new();
        cmd.on_structure_pick(Handle::new(1), Vec3::new(0.0, 0.0, 0.0));
        let wires = cmd.on_preview_wires(Vec3::new(50.0, 0.0, 0.0));
        assert_eq!(wires.len(), 1);
        assert_eq!(wires[0].points.len(), 2);
    }
}