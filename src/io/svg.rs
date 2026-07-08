// SVG export — model-space wires → a standalone SVG document.
//
// The 2D drafting view is already tessellated into `WireModel` polyline strips
// (curves flattened, `NaN` triples separating sub-strokes), so an SVG export is
// a direct mapping: one `<polyline>` per sub-stroke, coloured by the wire's RGB.
// The drawing is fit to a `viewBox` over its bounding box. SVG's Y axis points
// down while CAD's points up, so we mirror Y about the box to keep the drawing
// upright. Colours are taken as-is from the (background-adapted) wires, so the
// export is WYSIWYG against the supplied `background`.

use std::fmt::Write as _;

use crate::scene::wire_model::WireModel;

/// Build a standalone SVG document from model-space wires. `background` is an
/// optional page fill as RGB in [0, 1]; `None` leaves the page transparent.
/// Returns a minimal empty-canvas SVG when there is no finite geometry.
pub fn build_svg(wires: &[WireModel], background: Option<[f32; 3]>) -> String {
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
    // Hairline stroke in world units, scaled to the drawing so it reads at any
    // size (there is no device DPI at export time).
    let stroke_w = (vw.hypot(vh) * 0.0015).max(1e-6);

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
    for wire in wires {
        let [r, g, b, _a] = wire.color;
        let color = rgb_hex(r, g, b);
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
                r#"<polyline points="{}" fill="none" stroke="{color}" stroke-width="{stroke_w:.4}" stroke-linecap="round" stroke-linejoin="round"/>"#,
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

    fn wire(points: Vec<[f32; 3]>, color: [f32; 4]) -> WireModel {
        WireModel::solid("w".to_string(), points, color, false)
    }

    fn count(hay: &str, needle: &str) -> usize {
        hay.matches(needle).count()
    }

    #[test]
    fn empty_input_is_empty_canvas() {
        let svg = build_svg(&[], None);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains(r#"viewBox="0 0 1 1""#));
        assert_eq!(count(&svg, "<polyline"), 0);
    }

    #[test]
    fn single_line_emits_one_polyline_within_viewbox() {
        let svg = build_svg(
            &[wire(vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]], [0.0, 0.0, 0.0, 1.0])],
            None,
        );
        assert_eq!(count(&svg, "<polyline"), 1);
        // A horizontal line has zero-height bbox; padding must still give a
        // finite viewBox and both endpoints.
        assert!(svg.contains("0.000,0.000"));
        assert!(svg.contains("10.000,0.000"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn y_axis_is_flipped_upright() {
        // Vertical world line (0,0)→(0,10): the top (y=10) maps to the top of
        // the SVG viewBox (small svg-y), i.e. y is mirrored about the bbox.
        let svg = build_svg(
            &[wire(vec![[0.0, 0.0, 0.0], [0.0, 10.0, 0.0]], [0.0, 0.0, 0.0, 1.0])],
            None,
        );
        // world y=0 → svg y=10 ; world y=10 → svg y=0.
        assert!(svg.contains("0.000,10.000"));
        assert!(svg.contains("0.000,0.000"));
    }

    #[test]
    fn color_becomes_hex_stroke() {
        let svg = build_svg(
            &[wire(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]], [1.0, 0.0, 0.0, 1.0])],
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
            None,
        );
        assert_eq!(count(&svg, "<polyline"), 2);
    }

    #[test]
    fn background_emits_a_filled_rect() {
        let svg = build_svg(
            &[wire(vec![[0.0, 0.0, 0.0], [10.0, 10.0, 0.0]], [1.0, 1.0, 1.0, 1.0])],
            Some([0.0, 0.0, 0.0]),
        );
        assert!(svg.contains("<rect"));
        assert!(svg.contains(r##"fill="#000000""##));
    }
}
