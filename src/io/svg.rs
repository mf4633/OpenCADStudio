// SVG export — model-space and paper-space (layout) → standalone SVG.
//
// Two entry points share the same emit helpers:
//   * `build_svg`       — model space: the drawing auto-fit to a viewBox over
//                         its own bounding box, on the model background.
//   * `build_svg_sheet` — a paper-space layout: fixed to the paper rectangle
//                         (white sheet), colours adapted for white paper and
//                         lineweights in millimetres, matching the PDF plot.
//
// The 2D view is already tessellated into `WireModel` polyline strips (curves
// flattened, `NaN` triples separating sub-strokes) and `HatchModel` fills, so
// export is a direct mapping to `<polyline>` / `<path>` / `<line>`. SVG's Y
// axis points down while CAD's points up, so Y is mirrored about the box.

use std::fmt::Write as _;

use crate::scene::model::hatch_model::HatchModel;
use crate::scene::model::wire_model::WireModel;

/// Screen px → millimetres (inverse of render_style's 96-dpi mm→px).
const PX_TO_MM: f32 = 25.4 / 96.0;

/// Build a standalone SVG from model-space wires and hatches, auto-fit to the
/// geometry's bounding box. `background` is an optional page fill in [0, 1].
/// Returns a minimal empty-canvas SVG when there is no finite geometry.
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
        return String::from(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1"></svg>"#);
    }

    let span = (max_x - min_x).max(max_y - min_y);
    let pad = if span > 1e-9 { span * 0.05 } else { 1.0 };
    min_x -= pad;
    min_y -= pad;
    max_x += pad;
    max_y += pad;
    let vw = max_x - min_x;
    let vh = max_y - min_y;
    let flip = min_y + max_y;
    let base_sw = (vw.hypot(vh) * 0.0015).max(1e-6);

    let mut s = header(min_x, min_y, vw, vh);
    if let Some([r, g, b]) = background {
        rect(&mut s, min_x, min_y, vw, vh, &rgb_hex(r, g, b));
    }
    for h in hatches {
        emit_hatch(&mut s, h, flip, base_sw);
    }
    for wire in wires {
        let color = rgb_hex(wire.color[0], wire.color[1], wire.color[2]);
        let sw = base_sw * wire.line_weight_px.max(1.0);
        emit_wire(&mut s, &wire.points, flip, &color, sw);
    }
    s.push_str("</svg>");
    s
}

/// Build a paper-sheet SVG for a layout: the viewBox is the paper rectangle
/// `(x0, y0, paper_w, paper_h)` (paper millimetres), drawn on a white sheet.
/// Colours are adapted for white paper (near-white / yellow → black, cyan →
/// dark blue, matching the PDF plot) and lineweights are in millimetres.
/// `wipeouts` are drawn first (masks), then `hatches`, then `wires`.
pub fn build_svg_sheet(
    wires: &[WireModel],
    hatches: &[HatchModel],
    wipeouts: &[HatchModel],
    x0: f64,
    y0: f64,
    paper_w: f64,
    paper_h: f64,
) -> String {
    let (x0, y0) = (x0 as f32, y0 as f32);
    let (pw, ph) = (paper_w.max(1.0) as f32, paper_h.max(1.0) as f32);
    // Mirror Y within [y0, y0+ph]: world y → (y0 + y1) - y, y1 = y0 + ph.
    let flip = 2.0 * y0 + ph;
    // Thin default hairline for hatch pattern lines, in mm.
    let base_sw = 0.15_f32;

    let mut s = header(x0, y0, pw, ph);
    rect(&mut s, x0, y0, pw, ph, "#ffffff");

    for h in wipeouts.iter().chain(hatches.iter()) {
        emit_hatch(&mut s, h, flip, base_sw);
    }
    for wire in wires {
        // The white sheet already provides the paper boundary.
        if wire.name == "__paper_boundary__" {
            continue;
        }
        let [ar, ag, ab] = print_adapt(wire.color);
        let color = rgb_hex(ar, ag, ab);
        let sw = (wire.line_weight_px * PX_TO_MM).max(0.13);
        emit_wire(&mut s, &wire.points, flip, &color, sw);
    }
    s.push_str("</svg>");
    s
}

// ── shared emit helpers ─────────────────────────────────────────────────────

fn header(min_x: f32, min_y: f32, w: f32, h: f32) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{min_x:.3} {min_y:.3} {w:.3} {h:.3}">"#
    )
}

fn rect(s: &mut String, x: f32, y: f32, w: f32, h: f32, fill: &str) {
    let _ = write!(
        s,
        r#"<rect x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{h:.3}" fill="{fill}"/>"#
    );
}

/// Emit one wire's sub-strokes (NaN-split) as `<polyline>`s, Y mirrored by `flip`.
fn emit_wire(s: &mut String, points: &[[f32; 3]], flip: f32, color: &str, sw: f32) {
    for stroke in points.split(|p| p[0].is_nan() || p[1].is_nan()) {
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

/// Emit a hatch: pattern lines for pattern fills, else a filled boundary path.
fn emit_hatch(s: &mut String, h: &HatchModel, flip: f32, base_sw: f32) {
    let color = rgb_hex(h.color[0], h.color[1], h.color[2]);
    let segs = h.pattern_segments();
    if !segs.is_empty() {
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
        return;
    }
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

fn rgb_hex(r: f32, g: f32, b: f32) -> String {
    let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", c(r), c(g), c(b))
}

/// Adapt an entity colour for a white sheet, matching the PDF plot: near-white
/// and viewport-yellow become black; cyan (active viewport border) becomes dark
/// blue; everything else is unchanged.
fn print_adapt([r, g, b, _a]: [f32; 4]) -> [f32; 3] {
    let is_light = r > 0.80 && g > 0.80 && b > 0.80;
    let is_yellow = r > 0.80 && g > 0.70 && b < 0.30;
    let is_cyan = r < 0.30 && g > 0.70 && b > 0.70;
    if is_light || is_yellow {
        [0.0, 0.0, 0.0]
    } else if is_cyan {
        [0.0, 0.15, 0.50]
    } else {
        [r, g, b]
    }
}

// ── Autocomplete registry ─────────────────────────────────
inventory::submit!(crate::command::CommandRegistration { names: &["SVGOUT", "EXPORTSVG"] });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::model::hatch_model::{HatchModel, HatchPattern};
    use std::sync::Arc;

    fn wire(points: Vec<[f32; 3]>, color: [f32; 4]) -> WireModel {
        WireModel::solid("w".to_string(), points, color, false)
    }

    fn named_wire(name: &str, points: Vec<[f32; 3]>, color: [f32; 4]) -> WireModel {
        WireModel::solid(name.to_string(), points, color, false)
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

    fn stroke_width_of(svg: &str) -> f32 {
        let key = "stroke-width=\"";
        let i = svg.find(key).expect("a stroke-width") + key.len();
        let rest = &svg[i..];
        let end = rest.find('"').unwrap();
        rest[..end].parse().unwrap()
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

    #[test]
    fn stroke_width_scales_with_lineweight() {
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
        assert!(svg.contains("M0.000,") && svg.contains('Z'));
    }

    #[test]
    fn hatch_expands_the_viewbox_and_flips() {
        let h = solid_hatch(
            [100.0, 200.0],
            vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]],
            [0.5, 0.5, 0.5, 1.0],
        );
        let svg = build_svg(&[], &[h], None);
        assert!(svg.contains("<path"));
        assert!(svg.contains("110.000,"));
    }

    // ── paper sheet ─────────────────────────────────────────────────────────

    #[test]
    fn sheet_uses_paper_viewbox_and_white_background() {
        let svg = build_svg_sheet(
            &[wire(vec![[10.0, 10.0, 0.0], [287.0, 10.0, 0.0]], [0.0, 0.0, 0.0, 1.0])],
            &[],
            &[],
            0.0,
            0.0,
            297.0,
            210.0,
        );
        assert!(svg.contains(r#"viewBox="0.000 0.000 297.000 210.000""#));
        assert!(svg.contains("<rect") && svg.contains(r##"fill="#ffffff""##));
    }

    #[test]
    fn sheet_flips_y_within_the_paper() {
        // A wire on the paper's bottom edge (y=0) maps to the sheet bottom
        // (svg y = paper height); the top edge (y=210) maps to svg y=0.
        let svg = build_svg_sheet(
            &[wire(vec![[0.0, 0.0, 0.0], [0.0, 210.0, 0.0]], [0.0, 0.0, 0.0, 1.0])],
            &[],
            &[],
            0.0,
            0.0,
            297.0,
            210.0,
        );
        assert!(svg.contains("0.000,210.000"));
        assert!(svg.contains("0.000,0.000"));
    }

    #[test]
    fn sheet_adapts_white_lines_to_black() {
        // White model lines would vanish on white paper — they print black.
        let svg = build_svg_sheet(
            &[wire(vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0]], [1.0, 1.0, 1.0, 1.0])],
            &[],
            &[],
            0.0,
            0.0,
            297.0,
            210.0,
        );
        assert!(svg.contains(r##"stroke="#000000""##));
    }

    #[test]
    fn sheet_skips_paper_boundary_wire() {
        let svg = build_svg_sheet(
            &[named_wire(
                "__paper_boundary__",
                vec![[0.0, 0.0, 0.0], [297.0, 0.0, 0.0], [297.0, 210.0, 0.0]],
                [0.0, 0.0, 0.0, 1.0],
            )],
            &[],
            &[],
            0.0,
            0.0,
            297.0,
            210.0,
        );
        assert_eq!(count(&svg, "<polyline"), 0);
    }
}
