//! CPU-side complex linetype tessellation.
//!
//! Walks the entity path and applies LT elements (dashes, gaps, dots, shapes),
//! returning one `WireModel` per continuous stroke.
//!
//! Coordinate convention: 2D entities live in the **XZ** plane (Y = elevation).
//! Shape X → along the linetype direction; Shape Y → perpendicular in XZ.

use crate::entities::text_support::resolve_dxf_special_chars;
use crate::linetypes::{ComplexLt, LtSegment};
use crate::scene::lff;
use crate::scene::wire_model::WireModel;

// ── Public entry point ────────────────────────────────────────────────────

/// Apply `lt` along `path_pts` (the entity's tessellated world-space strip).
///
/// Returns one `WireModel` per continuous stroke. Pass the entity's `name`,
/// `color`, `selected` flag, and `line_weight_px` so the WireModels inherit
/// the entity's visual properties.
pub fn apply_along(
    name: &str,
    path_pts: &[[f32; 3]],
    lt: &ComplexLt,
    scale: f32,
    color: [f32; 4],
    selected: bool,
    line_weight_px: f32,
) -> Vec<WireModel> {
    if path_pts.len() < 2 || lt.segments.is_empty() {
        return vec![];
    }

    let scaled: Vec<LtSeg> = scale_segments(&lt.segments, scale);
    if scaled.is_empty() {
        return vec![];
    }

    let pattern_len: f32 = scaled
        .iter()
        .map(|s| match s {
            LtSeg::Dash(l) | LtSeg::Space(l) => *l,
            LtSeg::Dot | LtSeg::Shape { .. } | LtSeg::Text { .. } => 0.0,
        })
        .sum();
    if pattern_len < 1e-10 {
        return vec![];
    }

    let mut strokes: Vec<Vec<[f32; 3]>> = Vec::new();
    let mut cur_stroke: Vec<[f32; 3]> = Vec::new();

    let mut elem_idx: usize = 0;
    let mut elem_consumed: f32 = 0.0;

    for i in 0..path_pts.len() - 1 {
        let ps = path_pts[i];
        let pe = path_pts[i + 1];

        let dx = pe[0] - ps[0];
        let dy = pe[1] - ps[1];
        let dz = pe[2] - ps[2];
        let seg_len = (dx * dx + dy * dy + dz * dz).sqrt();
        if seg_len < 1e-10 {
            continue;
        }

        let fwd = [dx / seg_len, dy / seg_len, dz / seg_len];
        let perp = [-fwd[1], fwd[0], fwd[2]];

        let mut pos = 0.0f32;
        let mut stuck = 0usize;

        while pos < seg_len - 1e-6 {
            let pos_before = pos;
            let idx = elem_idx % scaled.len();

            match &scaled[idx] {
                LtSeg::Dash(dash_len) => {
                    let remaining_dash = dash_len - elem_consumed;
                    let remaining_seg = seg_len - pos;
                    let advance = remaining_dash.min(remaining_seg);

                    let p_start = lerp(ps, pe, pos / seg_len);
                    let p_end = lerp(ps, pe, (pos + advance) / seg_len);

                    if cur_stroke.is_empty() {
                        cur_stroke.push(p_start);
                    }
                    cur_stroke.push(p_end);

                    pos += advance;
                    elem_consumed += advance;
                    if (elem_consumed - dash_len).abs() < 1e-6 {
                        elem_idx += 1;
                        elem_consumed = 0.0;
                    }
                }

                LtSeg::Space(space_len) => {
                    let remaining = space_len - elem_consumed;
                    let advance = remaining.min(seg_len - pos);

                    flush(&mut cur_stroke, &mut strokes);

                    pos += advance;
                    elem_consumed += advance;
                    if (elem_consumed - space_len).abs() < 1e-6 {
                        elem_idx += 1;
                        elem_consumed = 0.0;
                    }
                }

                LtSeg::Dot => {
                    flush(&mut cur_stroke, &mut strokes);
                    let p = lerp(ps, pe, pos / seg_len);
                    strokes.push(vec![p, p]);
                    elem_idx += 1;
                    elem_consumed = 0.0;
                }

                LtSeg::Shape {
                    name: sh_name,
                    x,
                    y,
                    scale: sh_scale,
                    rot_deg,
                } => {
                    flush(&mut cur_stroke, &mut strokes);

                    let insert = lerp(ps, pe, pos / seg_len);
                    let insert = offset_pt(insert, fwd, perp, *x, *y);

                    let shape_strokes = emit_shape(sh_name, insert, fwd, perp, *sh_scale, *rot_deg);
                    strokes.extend(shape_strokes);

                    elem_idx += 1;
                    elem_consumed = 0.0;
                }
                LtSeg::Text {
                    text,
                    style,
                    x,
                    y,
                    scale: tx_scale,
                    rot_deg,
                } => {
                    flush(&mut cur_stroke, &mut strokes);

                    let insert = lerp(ps, pe, pos / seg_len);
                    let insert = offset_pt(insert, fwd, perp, *x, *y);
                    let fwd_angle = fwd[1].atan2(fwd[0]) + rot_deg.to_radians();
                    let resolved = resolve_dxf_special_chars(text);
                    let text_strokes = lff::tessellate_text_ex(
                        [insert[0], insert[1]],
                        *tx_scale,
                        fwd_angle,
                        1.0,
                        0.0,
                        style,
                        &resolved,
                    );
                    for stroke in &text_strokes {
                        if stroke.len() >= 2 {
                            let pts: Vec<[f32; 3]> =
                                stroke.iter().map(|&[sx, sy]| [sx, sy, insert[2]]).collect();
                            strokes.push(pts);
                        }
                    }

                    elem_idx += 1;
                    elem_consumed = 0.0;
                }
            }

            // Zero-length elements (dot / shape / text) advance `elem_idx` but
            // not `pos`. Bail only if a full pattern cycle passes without any
            // progress — this avoids an infinite loop without skipping the rest
            // of the path segment (the old `if pos < 1e-6 { break }` dropped
            // every dash after a leading dot/shape/text).
            if pos <= pos_before + 1e-9 {
                stuck += 1;
                if stuck > scaled.len() {
                    break;
                }
            } else {
                stuck = 0;
            }
        }
    }

    flush(&mut cur_stroke, &mut strokes);

    strokes
        .into_iter()
        .filter(|s| s.len() >= 2)
        .map(|pts| WireModel {
            name: name.to_string(),
            points: pts,
            color,
            selected,
            pattern_length: 0.0,
            pattern: [0.0; 8],
            line_weight_px,
            snap_pts: vec![],
            tangent_geoms: vec![],
            aci: 0,
            key_vertices: vec![],
            aabb: WireModel::UNBOUNDED_AABB,
            plinegen: true,
            vp_scissor: None,
            fill_tris: vec![],
        })
        .collect()
}

// ── Helpers ───────────────────────────────────────────────────────────────

#[derive(Clone)]
enum LtSeg {
    Dash(f32),
    Space(f32),
    Dot,
    Shape {
        name: String,
        x: f32,
        y: f32,
        scale: f32,
        rot_deg: f32,
    },
    Text {
        text: String,
        style: String,
        x: f32,
        y: f32,
        scale: f32,
        rot_deg: f32,
    },
}

fn scale_segments(segs: &[LtSegment], scale: f32) -> Vec<LtSeg> {
    segs.iter()
        .map(|s| match s {
            LtSegment::Dash(l) => LtSeg::Dash(l * scale),
            LtSegment::Space(l) => LtSeg::Space(l * scale),
            LtSegment::Dot => LtSeg::Dot,
            LtSegment::Shape {
                name,
                x,
                y,
                scale: sh_scale,
                rot_deg,
            } => LtSeg::Shape {
                name: name.clone(),
                x: x * scale,
                y: y * scale,
                scale: *sh_scale * scale,
                rot_deg: *rot_deg,
            },
            LtSegment::Text {
                text,
                style,
                x,
                y,
                scale: tx_scale,
                rot_deg,
            } => LtSeg::Text {
                text: text.clone(),
                style: style.clone(),
                x: x * scale,
                y: y * scale,
                scale: *tx_scale * scale,
                rot_deg: *rot_deg,
            },
        })
        .collect()
}

fn flush(cur: &mut Vec<[f32; 3]>, strokes: &mut Vec<Vec<[f32; 3]>>) {
    if !cur.is_empty() {
        strokes.push(std::mem::take(cur));
    }
}

fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn offset_pt(pt: [f32; 3], fwd: [f32; 3], perp: [f32; 3], dx: f32, dy: f32) -> [f32; 3] {
    [
        pt[0] + fwd[0] * dx + perp[0] * dy,
        pt[1] + fwd[1] * dx + perp[1] * dy,
        pt[2] + fwd[2] * dx + perp[2] * dy,
    ]
}

/// Transform a named linetype shape (from the converted `ltypeshp` LFF font)
/// into world-space strokes at the pen position.
fn emit_shape(
    name: &str,
    insert: [f32; 3],
    fwd: [f32; 3],
    perp: [f32; 3],
    scale: f32,
    rot_deg: f32,
) -> Vec<Vec<[f32; 3]>> {
    let shape = match lff::shape(name) {
        Some(s) => s,
        None => return vec![],
    };

    let rot_r = rot_deg.to_radians();
    let (cos_r, sin_r) = (rot_r.cos(), rot_r.sin());

    shape
        .strokes
        .iter()
        .map(|stroke| {
            stroke
                .iter()
                .map(|&[lx, ly]| {
                    let scaled_x = lx * scale;
                    let scaled_y = ly * scale;
                    // Rotate in the shape's local frame, then place along the
                    // line tangent (fwd) and perpendicular (perp).
                    let along_fwd = cos_r * scaled_x - sin_r * scaled_y;
                    let along_perp = sin_r * scaled_x + cos_r * scaled_y;
                    [
                        insert[0] + fwd[0] * along_fwd + perp[0] * along_perp,
                        insert[1] + fwd[1] * along_fwd + perp[1] * along_perp,
                        insert[2] + fwd[2] * along_fwd + perp[2] * along_perp,
                    ]
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linetypes::{ComplexLt, LtSegment};

    fn max_x(wires: &[WireModel]) -> f32 {
        wires
            .iter()
            .flat_map(|w| w.points.iter())
            .map(|p| p[0])
            .fold(f32::MIN, f32::max)
    }

    #[test]
    fn leading_dot_does_not_drop_the_rest_of_the_segment() {
        // A pattern starting with a zero-length element (Dot) applied to a
        // single path segment must still draw the dashes that follow.
        let lt = ComplexLt {
            segments: vec![
                LtSegment::Dot,
                LtSegment::Dash(5.0),
                LtSegment::Space(5.0),
            ],
        };
        let path = [[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]];
        let wires = apply_along("DOTLINE", &path, &lt, 1.0, [0.0, 0.0, 0.0, 1.0], false, 1.0);
        assert!(
            max_x(&wires) > 4.0,
            "dashes after the leading dot were dropped (max_x = {})",
            max_x(&wires)
        );
    }

    #[test]
    fn plain_dash_space_pattern_is_unchanged() {
        // Common case must be unaffected: 20 units of [dash 5, space 5] gives
        // exactly two dashes ([0,5] and [10,15]).
        let lt = ComplexLt {
            segments: vec![LtSegment::Dash(5.0), LtSegment::Space(5.0)],
        };
        let path = [[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]];
        let wires = apply_along("DASHED", &path, &lt, 1.0, [0.0, 0.0, 0.0, 1.0], false, 1.0);
        assert_eq!(wires.len(), 2, "expected two dashes");
        assert!((max_x(&wires) - 15.0).abs() < 1e-3, "second dash should end at 15");
    }

    #[test]
    fn dot_in_pattern_across_vertices_terminates_and_covers() {
        // A dot mid-pattern over a multi-segment polyline must terminate (no
        // infinite loop) and still draw dashes on every segment.
        let lt = ComplexLt {
            segments: vec![LtSegment::Dash(3.0), LtSegment::Dot, LtSegment::Space(3.0)],
        };
        let path = [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [20.0, 0.0, 0.0]];
        let wires = apply_along("DASHDOT", &path, &lt, 1.0, [0.0, 0.0, 0.0, 1.0], false, 1.0);
        assert!(!wires.is_empty());
        assert!(max_x(&wires) > 18.0, "dashes should reach the far end (max_x = {})", max_x(&wires));
    }
}
