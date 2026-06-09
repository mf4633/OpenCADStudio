// C3D-style acquisition previews: orange structure highlight, pipe rubber-band.

use acadrust::{EntityType, Handle};
use glam::Vec3;

use crate::scene::{Scene, WireModel};

use super::data::{self, StructurePick};

pub const STRUCTURE_PICK_PAD_FT: f64 = 15.0;

/// Orange highlight wires for a structure under the cursor.
pub fn highlight_structure(scene: &Scene, pick: &StructurePick) -> Vec<WireModel> {
    highlight_handles(scene, &[pick.handle], WireModel::PICK_HIGHLIGHT, 3.0)
}

/// Highlight any entity (e.g. catchment polyline) in acquisition color.
pub fn highlight_entity(scene: &Scene, handle: Handle) -> Vec<WireModel> {
    highlight_handles(scene, &[handle], WireModel::PICK_HIGHLIGHT, 2.5)
}

/// Dim orange fill on closed catchment while hovering.
pub fn highlight_catchment_poly(scene: &Scene, handle: Handle) -> Vec<WireModel> {
    let mut wires = highlight_handles(scene, &[handle], WireModel::PICK_HIGHLIGHT_DIM, 2.0);
    for w in &mut wires {
        w.line_weight_px = 2.0;
    }
    wires
}

pub fn highlight_handles(
    scene: &Scene,
    handles: &[Handle],
    color: [f32; 4],
    line_weight_px: f32,
) -> Vec<WireModel> {
    let mut out = scene.wire_models_for(handles);
    for w in &mut out {
        w.color = color;
        w.line_weight_px = line_weight_px;
    }
    out
}

/// Resolve the structure under the cursor for storm network commands.
pub fn structure_under_cursor(
    scene: &Scene,
    x: f64,
    y: f64,
    catchment_inlet_only: bool,
) -> Option<StructurePick> {
    data::structure_at_point(
        scene.document.entities(),
        x,
        y,
        STRUCTURE_PICK_PAD_FT,
        !catchment_inlet_only,
    )
}

/// Cyan rubber-band from a fixed structure center to the cursor (pipe run preview).
pub fn pipe_run_rubber_band(from_x: f64, from_y: f64, to: Vec3) -> WireModel {
    WireModel::solid(
        "__ss_pipe_preview__".into(),
        vec![
            [from_x as f32, from_y as f32, 0.0],
            [to.x, to.y, to.z],
        ],
        WireModel::CYAN,
        false,
    )
}

/// Extra preview wires for storm-sewer structure acquisition at `cursor`.
pub fn structure_acquire_previews(
    scene: &Scene,
    cursor: Vec3,
    catchment_inlet_only: bool,
) -> Vec<WireModel> {
    let Some(pick) = structure_under_cursor(scene, cursor.x as f64, cursor.y as f64, catchment_inlet_only)
    else {
        return vec![];
    };
    highlight_structure(scene, &pick)
}

/// Closed polyline under cursor (catchment step 1).
pub fn catchment_poly_under_cursor(scene: &Scene, handle: Handle) -> Vec<WireModel> {
    let Some(ent) = scene.document.get_entity(handle) else {
        return vec![];
    };
    if !matches!(ent, EntityType::LwPolyline(pl) if pl.is_closed) {
        return vec![];
    }
    highlight_catchment_poly(scene, handle)
}