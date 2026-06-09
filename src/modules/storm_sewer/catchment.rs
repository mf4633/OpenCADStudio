// Interactive catchment tagging: closed LwPolyline + STORMSEWER_CATCHMENT XDATA.

use acadrust::entities::LwPolyline;
use acadrust::{EntityType, Handle};
use glam::Vec3;

use stormsewer::catchment::{default_flow_length_ft, polygon_centroid, shoelace_area_sqft, sqft_to_acres};

use super::data;
use super::preview;
use crate::command::{CadCommand, CmdResult, ObjectPickHit};
use crate::scene::{Scene, WireModel};

fn parse_num(text: &str) -> Option<f64> {
    text.trim().replace(',', ".").parse::<f64>().ok()
}

enum CStep {
    PickPolyline,
    RunoffC,
    FlowLength,
    Slope,
    PickInlet,
}

pub struct TagCatchment {
    step: CStep,
    picked: Option<EntityType>,
    c: f64,
    flow_length: f64,
    slope: f64,
    inlet_handle: Handle,
    area_ac: f64,
    acquire_hint: Option<String>,
}

impl TagCatchment {
    pub fn new() -> Self {
        Self {
            step: CStep::PickPolyline,
            picked: None,
            c: 0.70,
            flow_length: 0.0,
            slope: 0.01,
            inlet_handle: Handle::NULL,
            area_ac: 0.0,
            acquire_hint: None,
        }
    }

    fn commit(&self) -> CmdResult {
        let mut ent = self.picked.clone().expect("polyline not picked");
        let EntityType::LwPolyline(pl) = &ent else {
            return CmdResult::Cancel;
        };
        if !pl.is_closed {
            return CmdResult::Cancel;
        }
        let handle = ent.common().handle;
        let xd = &mut ent.common_mut().extended_data;
        let kept: Vec<_> = xd
            .records()
            .iter()
            .filter(|r| r.application_name != data::APP_CATCHMENT)
            .cloned()
            .collect();
        xd.clear();
        for r in kept {
            xd.add_record(r);
        }
        xd.add_record(data::catchment_xdata(
            self.c,
            self.flow_length,
            self.slope,
            self.inlet_handle,
        ));
        CmdResult::ReplaceMany(vec![(handle, vec![ent])], vec![])
    }

    fn assign_inlet(&mut self, handle: Handle, pt: Vec3) {
        self.inlet_handle = handle;
        if self.flow_length <= 0.0 {
            if let Some(ref ent) = self.picked {
                if let EntityType::LwPolyline(pl) = ent {
                    let verts: Vec<_> = pl.vertices.iter().map(|v| (v.location.x, v.location.y)).collect();
                    let centroid = polygon_centroid(&verts);
                    self.flow_length =
                        default_flow_length_ft(centroid, (pt.x as f64, pt.y as f64));
                }
            }
        }
    }
}

impl Default for TagCatchment {
    fn default() -> Self {
        Self::new()
    }
}

impl CadCommand for TagCatchment {
    fn name(&self) -> &'static str {
        "SS_CATCHMENT"
    }

    fn prompt(&self) -> String {
        match self.step {
            CStep::PickPolyline => {
                "Catchment: click closed drainage-area polyline (highlights orange):".into()
            }
            CStep::RunoffC => format!(
                "Catchment area {:.3} ac — runoff C <{:.2}> (Enter to accept):",
                self.area_ac, self.c
            ),
            CStep::FlowLength => format!(
                "Flow path length, ft <{:.1}> (0 = auto from centroid to inlet):",
                self.flow_length
            ),
            CStep::Slope => format!("Average slope, ft/ft <{:.4}> (Enter to accept):", self.slope),
            CStep::PickInlet => {
                let hint = self
                    .acquire_hint
                    .as_deref()
                    .map(|h| format!(" [{h}]"))
                    .unwrap_or_default();
                format!(
                    "Catchment: click inlet/junction to drain to (orange snap){hint} — Enter = nearest:"
                )
            }
        }
    }

    fn set_acquisition_hint(&mut self, hint: Option<&str>) {
        if matches!(self.step, CStep::PickInlet) {
            self.acquire_hint = hint.map(str::to_string);
        }
    }

    fn wants_text_input(&self) -> bool {
        matches!(self.step, CStep::RunoffC | CStep::FlowLength | CStep::Slope)
    }

    fn needs_entity_pick(&self) -> bool {
        matches!(self.step, CStep::PickPolyline)
    }

    fn needs_structure_point_pick(&self) -> bool {
        matches!(self.step, CStep::PickInlet)
    }

    fn resolve_object_pick(&self, scene: &Scene, x: f64, y: f64) -> Option<ObjectPickHit> {
        let pick = preview::structure_under_cursor(scene, x, y, true)?;
        Some(ObjectPickHit {
            handle: pick.handle,
            x: pick.x,
            y: pick.y,
            label: pick.label(),
        })
    }

    fn object_pick_hover_previews(&self, scene: &Scene, cursor: Vec3) -> Vec<WireModel> {
        preview::structure_acquire_previews(scene, cursor, true)
    }

    fn object_pick_miss_message(&self) -> &'static str {
        "No storm structure near click — move closer or press Enter for nearest."
    }

    fn entity_pick_acquire_previews(&self, scene: &Scene, handle: Handle) -> Vec<WireModel> {
        if matches!(self.step, CStep::PickPolyline) {
            preview::catchment_poly_under_cursor(scene, handle)
        } else {
            vec![]
        }
    }

    fn entity_pick_acquire_hint(&self, _handle: Handle) -> Option<&'static str> {
        if matches!(self.step, CStep::PickPolyline) {
            Some("Catchment area")
        } else {
            None
        }
    }

    fn inject_picked_entity(&mut self, entity: EntityType) {
        if !matches!(self.step, CStep::PickPolyline) {
            return;
        }
        if let EntityType::LwPolyline(ref pl) = entity {
            if pl.is_closed && pl.vertices.len() >= 3 {
                let verts: Vec<_> = pl.vertices.iter().map(|v| (v.location.x, v.location.y)).collect();
                self.area_ac = sqft_to_acres(shoelace_area_sqft(&verts));
                self.picked = Some(entity);
            }
        }
    }

    fn inject_before_entity_pick(&self) -> bool {
        true
    }

    fn on_entity_pick(&mut self, _handle: Handle, _pt: Vec3) -> CmdResult {
        if self.picked.is_none() {
            return CmdResult::NeedPoint;
        }
        self.step = CStep::RunoffC;
        CmdResult::NeedPoint
    }

    fn on_structure_pick(&mut self, handle: Handle, pt: Vec3) -> CmdResult {
        self.assign_inlet(handle, pt);
        self.acquire_hint = None;
        self.commit()
    }

    fn on_text_input(&mut self, text: &str) -> Option<CmdResult> {
        let v = parse_num(text);
        match self.step {
            CStep::RunoffC => {
                if let Some(x) = v {
                    self.c = x;
                }
                self.step = CStep::FlowLength;
            }
            CStep::FlowLength => {
                if let Some(x) = v {
                    self.flow_length = x;
                }
                self.step = CStep::Slope;
            }
            CStep::Slope => {
                if let Some(x) = v {
                    self.slope = x;
                }
                self.step = CStep::PickInlet;
            }
            _ => {}
        }
        None
    }

    fn on_enter(&mut self) -> CmdResult {
        match self.step {
            CStep::PickInlet => {
                self.acquire_hint = None;
                self.commit()
            }
            CStep::RunoffC => {
                self.step = CStep::FlowLength;
                CmdResult::NeedPoint
            }
            CStep::FlowLength => {
                self.step = CStep::Slope;
                CmdResult::NeedPoint
            }
            CStep::Slope => {
                self.step = CStep::PickInlet;
                CmdResult::NeedPoint
            }
            CStep::PickPolyline => CmdResult::NeedPoint,
        }
    }

    fn on_point(&mut self, _pt: Vec3) -> CmdResult {
        CmdResult::NeedPoint
    }
}

/// Inspect a closed polyline and suggest defaults from geometry.
pub fn catchment_defaults_from_poly(pl: &LwPolyline, target_xy: Option<(f64, f64)>) -> (f64, f64, f64) {
    let verts: Vec<_> = pl.vertices.iter().map(|v| (v.location.x, v.location.y)).collect();
    let area_ac = sqft_to_acres(shoelace_area_sqft(&verts));
    let centroid = polygon_centroid(&verts);
    let flow_len = target_xy
        .map(|t| default_flow_length_ft(centroid, t))
        .unwrap_or(0.0);
    (area_ac, flow_len, 0.01)
}

/// Update structure entities with Tc computed from catchments + network assembly.
pub fn apply_tc_from_network<'a>(
    entities: impl Iterator<Item = &'a EntityType>,
    entities_mut: impl Iterator<Item = &'a mut EntityType>,
) -> Result<usize, String> {
    data::apply_tc_in_document(entities, entities_mut)
}

#[cfg(test)]
mod tests {
    use super::*;
    use acadrust::entities::LwVertex;
    use acadrust::types::Vector2;

    #[test]
    fn defaults_from_unit_square() {
        let mut pl = LwPolyline::default();
        pl.is_closed = true;
        pl.vertices = vec![
            LwVertex::new(Vector2::new(0.0, 0.0)),
            LwVertex::new(Vector2::new(10.0, 0.0)),
            LwVertex::new(Vector2::new(10.0, 10.0)),
            LwVertex::new(Vector2::new(0.0, 10.0)),
        ];
        let (area, flow, slope) = catchment_defaults_from_poly(&pl, Some((0.0, 0.0)));
        assert!((area - sqft_to_acres(100.0)).abs() < 1e-6);
        assert!(flow > 0.0);
        assert!((slope - 0.01).abs() < 1e-9);
    }
}