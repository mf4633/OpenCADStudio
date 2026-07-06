// Regression: hatch fills that live *inside* a block INSERT must reach the
// plot / PDF-export hatch set, not just the on-screen viewport.
//
// The export path (`paper_canvas_hatches`) used to collect only hatches owned
// directly by the layout block, so a hatch nested in a block was dropped and a
// plot printed the block as bare monochrome outlines. `synced_hatch_models`
// (the viewport) explodes visible INSERTs and materializes their fills; both
// paths now share `exploded_insert_hatch_models`.
use acadrust::entities::hatch::{
    BoundaryEdge, BoundaryPath, BoundaryPathFlags, HatchPatternLine, LineEdge,
};
use acadrust::entities::Hatch;
use acadrust::types::{Color as AcadColor, Vector2};
use acadrust::EntityType;
use OpenCADStudio::scene::model::hatch_model::HatchPattern;
use OpenCADStudio::scene::Scene;

fn is_blue(c: &[f32; 4]) -> bool {
    c[2] > 0.85 && c[0] < 0.20 && c[1] < 0.20
}

/// Build a 10x10 square solid hatch of the given ACI colour, offset by (cx, cy).
fn square_hatch_at(aci: u8, cx: f64, cy: f64) -> Hatch {
    let mut path = BoundaryPath::new();
    for (s, e) in [
        ((0.0, 0.0), (10.0, 0.0)),
        ((10.0, 0.0), (10.0, 10.0)),
        ((10.0, 10.0), (0.0, 10.0)),
        ((0.0, 10.0), (0.0, 0.0)),
    ] {
        path.edges.push(BoundaryEdge::Line(LineEdge {
            start: Vector2::new(s.0 + cx, s.1 + cy),
            end: Vector2::new(e.0 + cx, e.1 + cy),
        }));
    }
    let mut hatch = Hatch::new();
    hatch.paths.push(path);
    hatch.common.color = AcadColor::Index(aci); // 5 = blue
    hatch
}

fn square_hatch(aci: u8) -> Hatch {
    square_hatch_at(aci, 0.0, 0.0)
}

// A pattern (non-solid) hatch that carries its OWN resolved pattern line —
// a 45° family whose world-unit perpendicular spacing is `spacing` — plus a
// large `pattern_scale` that the (wrong) catalog path would multiply in.
fn ansi31_stored(spacing: f64, pattern_scale: f64) -> Hatch {
    ansi31_stored_at(spacing, pattern_scale, 0.0, 0.0)
}

fn ansi31_stored_at(spacing: f64, pattern_scale: f64, cx: f64, cy: f64) -> Hatch {
    let mut hatch = square_hatch_at(5, cx, cy);
    hatch.is_solid = false;
    hatch.pattern.name = "ANSI31".into();
    // offset = perpendicular step to the next line at 45°: (-s/√2, s/√2).
    let d = spacing / std::f64::consts::SQRT_2;
    // Set the field directly — `set_pattern_scale()` would recompute and
    // clobber the stored lines we are deliberately testing.
    hatch.pattern_scale = pattern_scale;
    hatch.pattern.lines = vec![HatchPatternLine {
        angle: std::f64::consts::FRAC_PI_4,
        base_point: Vector2::new(0.0, 0.0),
        offset: Vector2::new(-d, d),
        dash_lengths: vec![],
    }];
    hatch
}

// Self-contained guard that runs everywhere (no external asset needed).
#[test]
fn block_internal_hatch_reaches_export() {
    let mut scene = Scene::new();

    // A blue hatch, wrapped into a block and inserted in model space — the
    // minimal shape of "coloured fill nested in a block".
    let h = scene.add_entity(EntityType::Hatch(square_hatch(5)));
    scene
        .create_block_from_entities(&[h], "LOGO", glam::Vec3::ZERO)
        .expect("wrap hatch into a block + insert");
    scene.populate_hatches_from_document();

    let hatches = scene.paper_canvas_hatches();
    let blue = hatches.iter().filter(|m| is_blue(&m.color)).count();
    assert!(
        blue > 0,
        "block-internal hatch dropped from the export set (len={}) — a plot \
         would print the block without its colours",
        hatches.len()
    );
}

// A pattern hatch that stores its own resolved line geometry must render at
// THAT spacing — not the name-matched catalog spacing × pattern_scale. Before
// the fix, ANSI31 was re-derived from the metric catalog (3.175) × scale, so a
// hatch authored at ~0.5-unit spacing collapsed to a near-empty few lines.
#[test]
fn pattern_hatch_uses_stored_line_spacing() {
    let mut scene = Scene::new();
    // Spacing 0.5; a big pattern_scale (10) that the catalog path would apply.
    scene.add_entity(EntityType::Hatch(ansi31_stored(0.5, 10.0)));
    scene.populate_hatches_from_document();

    let hatches = scene.paper_canvas_hatches();
    let m = hatches
        .iter()
        .find(|m| matches!(m.pattern, HatchPattern::Pattern(_)))
        .expect("pattern hatch present in export set");

    let HatchPattern::Pattern(fams) = &m.pattern else { unreachable!() };
    // Effective perpendicular spacing = family.dy * model.scale.
    let dy = fams[0].dy.abs();
    let spacing = dy * m.scale;
    assert!(
        (spacing - 0.5).abs() < 0.05,
        "expected ~0.5-unit line spacing from the stored offset, got {spacing} \
         (dy={dy}, scale={}) — catalog×pattern_scale would give ~31.75",
        m.scale
    );
    // Density sanity: a 10x10 boundary at 0.5 spacing -> ~28 lines, not ~1.
    let lines = m.pattern_segments().len();
    assert!(
        lines >= 10,
        "expected a dense fill (~28 lines), got {lines} — pattern too sparse"
    );
}

// A fine-spaced pattern hatch placed far from the pattern origin (0,0) must
// still fill. `pattern_segments` used to clamp the ABSOLUTE line index to
// ±MAX_LINES_PER_FAMILY; a hatch thousands of units away has large-magnitude k
// on both ends, so the clamp inverted the range and emitted nothing — a
// fine-spaced fill far from the origin printed as empty outlines.
#[test]
fn far_from_origin_pattern_hatch_still_fills() {
    let mut scene = Scene::new();
    // Spacing 0.3 at ~4000 units out → k ≈ 4000/0.3·√2 ≈ 18000, well past the
    // 4096 clamp on both ends.
    scene.add_entity(EntityType::Hatch(ansi31_stored_at(0.3, 1.0, 4000.0, 4000.0)));
    scene.populate_hatches_from_document();

    let hatches = scene.paper_canvas_hatches();
    let m = hatches
        .iter()
        .find(|m| matches!(m.pattern, HatchPattern::Pattern(_)))
        .expect("pattern hatch present");
    let lines = m.pattern_segments().len();
    assert!(
        lines >= 10,
        "far-from-origin pattern hatch produced {lines} lines — fill was dropped"
    );
}

// A TEXTBOX boundary path (AutoCAD's text-bounding-box, used only for island
// detection) must NOT be filled. Painting its rectangle solid produces a
// phantom bar AutoCAD never shows.
#[test]
fn textbox_boundary_path_is_not_filled() {
    fn rect_path(x0: f64, y0: f64, x1: f64, y1: f64) -> BoundaryPath {
        let mut p = BoundaryPath::new();
        for (s, e) in [
            ((x0, y0), (x1, y0)),
            ((x1, y0), (x1, y1)),
            ((x1, y1), (x0, y1)),
            ((x0, y1), (x0, y0)),
        ] {
            p.edges.push(BoundaryEdge::Line(LineEdge {
                start: Vector2::new(s.0, s.1),
                end: Vector2::new(e.0, e.1),
            }));
        }
        p
    }

    let mut scene = Scene::new();
    let mut hatch = Hatch::new();
    hatch.common.color = AcadColor::Index(5);
    // A small real fill region...
    hatch.paths.push(rect_path(0.0, 0.0, 10.0, 10.0));
    // ...plus a large TEXTBOX rectangle that must be ignored.
    let mut tb = rect_path(0.0, 0.0, 200.0, 50.0);
    tb.flags = BoundaryPathFlags::from_bits(8 | 1); // TEXTBOX + EXTERNAL
    hatch.paths.push(tb);
    scene.add_entity(EntityType::Hatch(hatch));
    scene.populate_hatches_from_document();

    let hatches = scene.paper_canvas_hatches();
    let m = hatches.first().expect("hatch present");
    // The boundary must not extend into the 200x50 TEXTBOX rectangle.
    let max_x = m
        .boundary
        .iter()
        .filter(|v| v[0].is_finite())
        .map(|v| v[0])
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        max_x < 50.0,
        "TEXTBOX rectangle leaked into the fill boundary (max_x={max_x}) — it \
         would paint a phantom bar"
    );
}
