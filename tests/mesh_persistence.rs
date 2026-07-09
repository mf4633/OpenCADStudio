// End-to-end persistence check for EXTRUDE/REVOLVE/SWEEP/LOFT/IMPORTOBJ output.
//
// Those commands store their truck tessellation as an ACAD_MESH entity (built
// by `scene::mesh_tess::mesh_entity_from_model`) so the solid survives a
// save/reload. This test drives the real acadrust writers and readers for both
// DXF and DWG and asserts the mesh geometry comes back intact — the part the
// unit tests cannot cover.

use std::path::PathBuf;

use acadrust::{CadDocument, EntityType};
use OpenCADStudio::scene::mesh_model::MeshModel;
use OpenCADStudio::scene::mesh_tess::{mesh_entity_from_model, tessellate_mesh_entity};

/// A unit tetrahedron: 4 vertices, 4 triangular faces.
fn tetrahedron_model() -> MeshModel {
    MeshModel {
        name: String::new(),
        verts: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ],
        normals: vec![],
        indices: vec![0, 1, 2, 0, 1, 3, 0, 2, 3, 1, 2, 3],
        color: [0.2, 0.4, 0.8, 1.0],
        selected: false,
    }
}

fn tmp_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    // Include the PID so parallel test runs don't collide.
    p.push(format!("ocs_mesh_persist_{}_{}", std::process::id(), name));
    p
}

/// Save a document containing one Mesh entity, reload it, and return the first
/// Mesh entity found — proving the writer + reader round-trip the geometry.
fn round_trip(ext: &str) -> (usize, [f64; 3]) {
    let model = tetrahedron_model();
    // Store with a non-zero world offset, exactly as the command handlers do.
    let world_offset = [100.0, 200.0, 0.0];
    let entity = mesh_entity_from_model(&model, world_offset);

    let mut doc = CadDocument::new();
    doc.add_entity(entity).expect("add mesh entity");

    let path = tmp_path(&format!("mesh.{ext}"));
    OpenCADStudio::io::save(&doc, &path).unwrap_or_else(|e| panic!("save {ext}: {e}"));

    let doc2 =
        OpenCADStudio::io::load_file(&path).unwrap_or_else(|e| panic!("load {ext}: {e}"));
    let _ = std::fs::remove_file(&path);

    let mesh_entity = doc2
        .entities()
        .find(|e| matches!(e, EntityType::Mesh(_)))
        .unwrap_or_else(|| panic!("no Mesh entity after {ext} round-trip"));

    // The reloaded entity must still tessellate into a renderable mesh.
    let set = tessellate_mesh_entity(mesh_entity, model.color, [0.0, 0.0, 0.0])
        .unwrap_or_else(|| panic!("reloaded {ext} mesh did not tessellate"));
    let lod = &set.lods[0];
    assert_eq!(lod.indices.len(), 12, "{ext}: expected 4 triangles");

    // Report vertex count and the WCS position of the vertex that was (1,0,0)
    // locally → (101, 200, 0) in WCS.
    if let EntityType::Mesh(m) = mesh_entity {
        (m.vertices.len(), [m.vertices[1].x, m.vertices[1].y, m.vertices[1].z])
    } else {
        unreachable!()
    }
}

#[test]
fn mesh_survives_dxf_round_trip() {
    let (nverts, v1) = round_trip("dxf");
    assert_eq!(nverts, 4, "dxf: vertex count");
    assert!((v1[0] - 101.0).abs() < 1e-4, "dxf: vertex X = {}", v1[0]);
    assert!((v1[1] - 200.0).abs() < 1e-4, "dxf: vertex Y = {}", v1[1]);
}

#[test]
fn mesh_survives_dwg_round_trip() {
    let (nverts, v1) = round_trip("dwg");
    assert_eq!(nverts, 4, "dwg: vertex count");
    assert!((v1[0] - 101.0).abs() < 1e-4, "dwg: vertex X = {}", v1[0]);
    assert!((v1[1] - 200.0).abs() < 1e-4, "dwg: vertex Y = {}", v1[1]);
}
