// SVG export — model-space wires and hatch fills → a standalone SVG document.
//
// The 2D drafting view is already tessellated into `WireModel` polyline strips
// (curves flattened, `NaN` triples separating sub-strokes) and `HatchModel`
// fills, so an SVG export is a direct mapping: one `<polyline>` per wire
// sub-stroke and, per hatch, either a filled `<path>` (solid / gradient) or
// pattern `<line>`s (hatch patterns), coloured by the model's RGB. Everything
// is fit to a `viewBox` over the combined bounding box. SVG's Y axis points
// down while CAD's points up, so we mirror Y about the box to keep the drawing
// upright. Colours are taken as-is from the (background-adapted) models, so the
// export is WYSIWYG against the supplied `background`.

use std::fmt::Write as _;

use crate::scene::hatch_model::HatchModel;
use crate::scene::wire_model::WireModel;

/// Build a standalone SVG document from model-space wires and hatches.
/// `background` is an optional page fill as RGB in [0, 1]; `None` leaves the
/// page transparent. Returns a minimal empty-canvas SVG when there is no finite
/// geometry.
pub fn build_svg(
    wires: &[WireModel],
    hatches: &[HatchModel],
    background: Option<[f32; 3]>,
) -> String {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for w in wires {
        for &[x, y, _] in &w.points {
            if x.is_finite() && y.is_finite() {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    for h in hatches {
        let (ox, oy) = (h.world_origin[0] as f32, h.world_origin[1] as f32);
        for &[vx, vy] in h.boundary.iter() {
            if vx.is_finite() && vy.is_finite() {
                let (x, y) = (ox + vx, oy + vy);
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    if !min_x.is_finite() {
        // No finite geometry at all.
        return String::from(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1"></svg>"#);
    }

    // Pad so edge strokes aren't clipped; handle a zero-extent axis (a single
    // straight line, or a point) by falling back to a unit pad.
    let span = (max_x - min_x).max(max_y - min_y);
    let pad = if span > 1e-9 { span * 0.05 } else { 1.0 };
    min_x -= pad;
    min_y -= pad;
    max_x += pad;
    max_y += pad;
    let vw = max_x - min_x;
    let vh = max_y - min_y;
    // world y → (min_y + max_y) - y keeps the drawing upright under SVG's
    // Y-down axis while staying inside the same viewBox.
    let flip = min_y + max_y;
    // Base hairline in world units, scaled to the drawing so it reads at any
    // size (there is no device DPI at export time). Per-wire widths multiply
    // this by the entity's resolved lineweight so heavy lines (walls) export
    // thicker than thin ones (dimensions), matching the on-screen weighting.
    let base_sw = (vw.hypot(vh) * 0.0015).max(1e-6);

    let mut s = String::new();
    let _ = write!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{min_x:.3} {min_y:.3} {vw:.3} {vh:.3}">"#
    );
    if let Some([r, g, b]) = background {
        let _ = write!(
            s,
            r#"<rect x="{min_x:.3}" y="{min_y:.3}" width="{vw:.3}" height="{vh:.3}" fill="{}"/>"#,
            rgb_hex(r, g, b)
        );
    }

    // Hatches first, so wires draw on top of their fills (matching render order).
    for h in hatches {
        let [r, g, b, _a] = h.color;
        let color = rgb_hex(r, g, b);
        let segs = h.pattern_segments();
        if !segs.is_empty() {
            // Pattern hatch: the family lines (already in WCS-relative coords).
            for [a, b_pt] in segs {
                let _ = write!(
                    s,
                    r#"<line x1="{:.3}" y1="{:.3}" x2="{:.3}" y2="{:.3}" stroke="{color}" stroke-width="{base_sw:.4}"/>"#,
                    a[0],
                    flip - a[1],
                    b_pt[0],
                    flip - b_pt[1]
                );
            }
        } else {
            // Solid / gradient: fill the boundary rings (NaN-separated), with
            // even-odd fill so islands / holes cut out correctly.
            let (ox, oy) = (h.world_origin[0] as f32, h.world_origin[1] as f32);
            let mut d = String::new();
            let mut ring_len = 0usize;
            let mut started = false;
            for &[vx, vy] in h.boundary.iter() {
                if vx.is_nan() || vy.is_nan() {
                    if ring_len >= 3 {
                        d.push_str("Z ");
                    }
                    started = false;
                    ring_len = 0;
                    continue;
                }
                let (x, y) = (ox + vx, flip - (oy + vy));
                if started {
                    let _ = write!(d, "L{x:.3},{y:.3} ");
                } else {
                    let _ = write!(d, "M{x:.3},{y:.3} ");
                    started = true;
                }
                ring_len += 1;
            }
            if ring_len >= 3 {
                d.push_str("Z ");
            }
            if !d.is_empty() {
                let _ = write!(
                    s,
                    r#"<path d="{}" fill="{color}" fill-rule="evenodd" stroke="none"/>"#,
                    d.trim_end()
                );
            }
        }
    }

    for wire in wires {
        let [r, g, b, _a] = wire.color;
        let color = rgb_hex(r, g, b);
        // Resolved lineweight (screen px, always ≥ 1.0 from render_style) scales
        // the hairline; clamped to ≥ 1× so a default-weight line keeps the base
        // width and heavier lines get proportionally thicker.
        let sw = base_sw * wire.line_weight_px.max(1.0);
        // `points` is one flat strip; NaN triples split it into sub-strokes.
        for stroke in wire.points.split(|p| p[0].is_nan() || p[1].is_nan()) {
            if stroke.len() < 2 {
                continue;
            }
            let mut pts = String::new();
            for &[x, y, _] in stroke {
                let _ = write!(pts, "{x:.3},{:.3} ", flip - y);
            }
            let _ = write!(
                s,
                r#"<polyline points="{}" fill="none" stroke="{color}" stroke-width="{sw:.4}" stroke-linecap="round" stroke-linejoin="round"/>"#,
                pts.trim_end()
            );
        }
    }
    s.push_str("</svg>");
    s
}

fn rgb_hex(r: f32, g: f32, b: f32) -> String {
    let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", c(r), c(g), c(b))
}

// ── Autocomplete registry ─────────────────────────────────
inventory::submit!(crate::command::CommandRegistration { names: &["SVGOUT", "EXPORTSVG"] });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::hatch_model::{HatchModel, HatchPattern};
    use std::sync::Arc;

    fn wire(points: Vec<[f32; 3]>, color: [f32; 4]) -> WireModel {
        WireModel::solid("w".to_string(), points, color, false)
    }

    fn solid_hatch(origin: [f64; 2], boundary: Vec<[f32; 2]>, color: [f32; 4]) -> HatchModel {
        HatchModel {
            world_origin: origin,
            boundary: Arc::new(boundary),
            pattern: HatchPattern::Solid,
            name: "SOLID".into(),
            color,
            angle_offset: 0.0,
            scale: 1.0,
            vp_scissor: None,
            draw_depth: 0.0,
        }
    }

    fn count(hay: &str, needle: &str) -> usize {
        hay.matches(needle).count()
    }

    #[test]
    fn empty_input_is_empty_canvas() {
        let svg = build_svg(&[], &[], None);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains(r#"viewBox="0 0 1 1""#));
        assert_eq!(count(&svg, "<polyline"), 0);
        assert_eq!(count(&svg, "<path"), 0);
    }

    #[test]
    fn single_line_emits_one_polyline_within_viewbox() {
        let svg = build_svg(
            &[wire(vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]], [0.0, 0.0, 0.0, 1.0])],
            &[],
            None,
        );
        assert_eq!(count(&svg, "<polyline"), 1);
        assert!(svg.contains("0.000,0.000"));
        assert!(svg.contains("10.000,0.000"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn y_axis_is_flipped_upright() {
        let svg = build_svg(
            &[wire(vec![[0.0, 0.0, 0.0], [0.0, 10.0, 0.0]], [0.0, 0.0, 0.0, 1.0])],
            &[],
            None,
        );
        assert!(svg.contains("0.000,10.000"));
        assert!(svg.contains("0.000,0.000"));
    }

    #[test]
    fn color_becomes_hex_stroke() {
        let svg = build_svg(
            &[wire(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]], [1.0, 0.0, 0.0, 1.0])],
            &[],
            None,
        );
        assert!(svg.contains(r##"stroke="#ff0000""##));
    }

    #[test]
    fn nan_splits_into_separate_polylines() {
        let nan = f32::NAN;
        let svg = build_svg(
            &[wire(
                vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [nan, nan, nan],
                    [2.0, 0.0, 0.0],
                    [3.0, 0.0, 0.0],
                ],
                [0.0, 0.0, 0.0, 1.0],
            )],
            &[],
            None,
        );
        assert_eq!(count(&svg, "<polyline"), 2);
    }

    #[test]
    fn background_emits_a_filled_rect() {
        let svg = build_svg(
            &[wire(vec![[0.0, 0.0, 0.0], [10.0, 10.0, 0.0]], [1.0, 1.0, 1.0, 1.0])],
            &[],
            Some([0.0, 0.0, 0.0]),
        );
        assert!(svg.contains("<rect"));
        assert!(svg.contains(r##"fill="#000000""##));
    }

    fn stroke_width_of(svg: &str) -> f32 {
        let key = "stroke-width=\"";
        let i = svg.find(key).expect("a stroke-width") + key.len();
        let rest = &svg[i..];
        let end = rest.find('"').unwrap();
        rest[..end].parse().unwrap()
    }

    #[test]
    fn stroke_width_scales_with_lineweight() {
        // Same geometry (so the base hairline is identical) at two lineweights;
        // a 3× resolved weight must yield a 3× stroke width.
        let mut thin = wire(vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]], [0.0, 0.0, 0.0, 1.0]);
        thin.line_weight_px = 1.0;
        let mut thick = wire(vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]], [0.0, 0.0, 0.0, 1.0]);
        thick.line_weight_px = 3.0;
        let wt = stroke_width_of(&build_svg(&[thin], &[], None));
        let wk = stroke_width_of(&build_svg(&[thick], &[], None));
        assert!((wk / wt - 3.0).abs() < 0.02, "thin {wt} thick {wk}");
    }

    #[test]
    fn solid_hatch_becomes_filled_path() {
        let h = solid_hatch(
            [0.0, 0.0],
            vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]],
            [1.0, 0.0, 0.0, 1.0],
        );
        let svg = build_svg(&[], &[h], None);
        assert_eq!(count(&svg, "<path"), 1);
        assert!(svg.contains(r##"fill="#ff0000""##));
        assert!(svg.contains(r#"fill-rule="evenodd""#));
        // Closed ring: M … L … Z.
        assert!(svg.contains("M0.000,") && svg.contains('Z'));
    }

    #[test]
    fn hatch_expands_the_viewbox_and_flips() {
        // A hatch alone must drive the viewBox; its top edge (y=10) maps to the
        // top of the box (flipped), and world_origin offsets the vertices.
        let h = solid_hatch(
            [100.0, 200.0],
            vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]],
            [0.5, 0.5, 0.5, 1.0],
        );
        let svg = build_svg(&[], &[h], None);
        assert!(svg.contains("<path"));
        // Vertex (100+10, 200+10) present with Y mirrored — appears as 110.000
        // in x and the flipped y (min_y+max_y - 210).
        assert!(svg.contains("110.000,"));
    }
}
