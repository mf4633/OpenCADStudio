// DXF/DWG round-trip coverage for core 2D entities. Builds a document, saves
// it through the real acadrust writers, reads it back, and asserts the geometry
// survives — the file I/O every drawing depends on. A field that doesn't
// round-trip shows up as a failing assertion here rather than as silent data
// loss in a saved drawing.

use acadrust::entities::{Arc, Circle, Ellipse, Line, LwPolyline, LwVertex, Point, Text};
use acadrust::types::{Vector2, Vector3};
use acadrust::{CadDocument, EntityType};

fn sample_doc() -> CadDocument {
    let mut doc = CadDocument::new();

    doc.add_entity(EntityType::Line(Line::from_coords(1.0, 2.0, 0.0, 8.0, 9.0, 0.0)))
        .expect("add line");

    doc.add_entity(EntityType::Circle(Circle::from_center_radius(
        Vector3::new(5.0, 6.0, 0.0),
        3.5,
    )))
    .expect("add circle");

    let mut arc = Arc::new();
    arc.center = Vector3::new(0.0, 0.0, 0.0);
    arc.radius = 4.0;
    arc.start_angle = 0.5;
    arc.end_angle = 2.0;
    doc.add_entity(EntityType::Arc(arc)).expect("add arc");

    let mut poly = LwPolyline::new();
    poly.vertices.push(LwVertex::new(Vector2::new(0.0, 0.0)));
    let mut mid = LwVertex::new(Vector2::new(10.0, 0.0));
    mid.bulge = 0.5; // arc segment
    poly.vertices.push(mid);
    poly.vertices.push(LwVertex::new(Vector2::new(10.0, 10.0)));
    poly.is_closed = true;
    doc.add_entity(EntityType::LwPolyline(poly)).expect("add lwpolyline");

    let mut text = Text::new();
    text.value = "HELLO".to_string();
    text.insertion_point = Vector3::new(3.0, 4.0, 0.0);
    text.height = 2.5;
    text.rotation = 0.3;
    doc.add_entity(EntityType::Text(text)).expect("add text");

    let mut ell = Ellipse::from_center_axes(Vector3::new(1.0, 1.0, 0.0), Vector3::new(5.0, 0.0, 0.0), 0.4);
    ell.start_parameter = 0.0;
    ell.end_parameter = std::f64::consts::PI;
    doc.add_entity(EntityType::Ellipse(ell)).expect("add ellipse");

    let mut pt = Point::new();
    pt.location = Vector3::new(7.0, 8.0, 0.0);
    doc.add_entity(EntityType::Point(pt)).expect("add point");

    doc
}

fn round_trip(doc: &CadDocument, ext: &str) -> CadDocument {
    let mut path = std::env::temp_dir();
    path.push(format!("ocs_roundtrip_{}.{ext}", std::process::id()));
    OpenCADStudio::io::save(doc, &path).unwrap_or_else(|e| panic!("save {ext}: {e}"));
    let back = OpenCADStudio::io::load_file(&path).unwrap_or_else(|e| panic!("load {ext}: {e}"));
    let _ = std::fs::remove_file(&path);
    back
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

fn assert_geometry_survives(doc: &CadDocument, ext: &str) {
    let mut seen = [false; 7];
    for e in doc.entities() {
        match e {
            EntityType::Line(l) => {
                seen[0] = true;
                assert!(close(l.start.x, 1.0) && close(l.start.y, 2.0), "{ext}: line start");
                assert!(close(l.end.x, 8.0) && close(l.end.y, 9.0), "{ext}: line end");
            }
            EntityType::Circle(c) => {
                seen[1] = true;
                assert!(close(c.center.x, 5.0) && close(c.center.y, 6.0), "{ext}: circle center");
                assert!(close(c.radius, 3.5), "{ext}: circle radius = {}", c.radius);
            }
            EntityType::Arc(a) => {
                seen[2] = true;
                assert!(close(a.center.x, 0.0) && close(a.center.y, 0.0), "{ext}: arc center");
                assert!(close(a.radius, 4.0), "{ext}: arc radius = {}", a.radius);
                assert!(close(a.start_angle, 0.5), "{ext}: arc start = {}", a.start_angle);
                assert!(close(a.end_angle, 2.0), "{ext}: arc end = {}", a.end_angle);
            }
            EntityType::LwPolyline(p) => {
                seen[3] = true;
                assert_eq!(p.vertices.len(), 3, "{ext}: lwpolyline vertex count");
                assert!(p.is_closed, "{ext}: lwpolyline closed flag");
                assert!(
                    close(p.vertices[1].location.x, 10.0) && close(p.vertices[1].bulge, 0.5),
                    "{ext}: lwpolyline vertex/bulge (bulge = {})",
                    p.vertices[1].bulge
                );
            }
            EntityType::Text(t) => {
                seen[4] = true;
                assert_eq!(t.value, "HELLO", "{ext}: text value");
                assert!(close(t.height, 2.5), "{ext}: text height = {}", t.height);
                assert!(close(t.rotation, 0.3), "{ext}: text rotation = {}", t.rotation);
                assert!(
                    close(t.insertion_point.x, 3.0) && close(t.insertion_point.y, 4.0),
                    "{ext}: text insertion"
                );
            }
            EntityType::Ellipse(el) => {
                seen[5] = true;
                assert!(close(el.center.x, 1.0) && close(el.center.y, 1.0), "{ext}: ellipse center");
                assert!(close(el.major_axis.x, 5.0), "{ext}: ellipse major axis");
                assert!(
                    close(el.minor_axis_ratio, 0.4),
                    "{ext}: ellipse ratio = {}",
                    el.minor_axis_ratio
                );
            }
            EntityType::Point(p) => {
                seen[6] = true;
                assert!(
                    close(p.location.x, 7.0) && close(p.location.y, 8.0),
                    "{ext}: point location"
                );
            }
            _ => {}
        }
    }
    let names = ["line", "circle", "arc", "lwpolyline", "text", "ellipse", "point"];
    for (ok, name) in seen.iter().zip(names) {
        assert!(ok, "{ext}: {name} missing after round-trip");
    }
}

#[test]
fn entities_survive_dxf() {
    let back = round_trip(&sample_doc(), "dxf");
    assert_geometry_survives(&back, "dxf");
}

#[test]
fn entities_survive_dwg() {
    let back = round_trip(&sample_doc(), "dwg");
    assert_geometry_survives(&back, "dwg");
}
