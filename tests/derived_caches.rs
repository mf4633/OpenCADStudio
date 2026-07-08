// Coverage for the fused planning pass in `build_derived_caches` (Roadmap
// 1.3): a single walk now detects corrupt entities, classifies the
// hatch/image/mesh cache handles, and accumulates the world-offset centroids.
// The corrupt-entity purge used to be a separate `purge_corrupt_entities`
// walk; this test pins that it still drops junk entities, keeps valid ones,
// and reports the count — the behaviour the merge must preserve.

use acadrust::entities::Circle;
use acadrust::types::Vector3;
use acadrust::{CadDocument, EntityType};
use OpenCADStudio::scene::build_derived_caches;

fn circle(cx: f64, radius: f64) -> EntityType {
    EntityType::Circle(Circle {
        center: Vector3::new(cx, 0.0, 0.0),
        radius,
        ..Default::default()
    })
}

#[test]
fn build_derived_caches_purges_corrupt_and_keeps_valid() {
    let mut doc = CadDocument::new();
    // One valid circle plus two corrupt ones the sanity guard rejects:
    // a zero-radius circle (degenerate curve) and a NaN-centre circle.
    doc.add_entity(circle(0.0, 5.0)).expect("add valid circle");
    doc.add_entity(circle(10.0, 0.0)).expect("add zero-radius circle");
    doc.add_entity(EntityType::Circle(Circle {
        center: Vector3::new(f64::NAN, 0.0, 0.0),
        radius: 3.0,
        ..Default::default()
    }))
    .expect("add NaN-centre circle");

    assert_eq!(doc.entities().count(), 3, "three entities before the build");

    let caches = build_derived_caches(&mut doc);

    assert_eq!(
        caches.corrupt_dropped, 2,
        "zero-radius and NaN-centre circles are both corrupt"
    );
    assert_eq!(
        doc.entities().count(),
        1,
        "only the valid circle survives the purge"
    );
    // A plain circle is none of hatch / image / mesh, so every cache is empty.
    assert!(caches.hatches.is_empty(), "no hatch entities");
    assert!(caches.images.is_empty(), "no image entities");
    assert!(caches.meshes.is_empty(), "no mesh entities");
}
