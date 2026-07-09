// End-to-end persistence check for Design-group boolean output (UNION /
// SUBTRACT / INTERSECT).
//
// The truck-shapeops result cannot be written back as ACIS, so `solid_boolean`
// stores its tessellation as an ACAD_MESH entity (via
// `scene::mesh_tess::mesh_entity_from_model`) — the same durable path
// EXTRUDE/REVOLVE use. Before that, the result lived in a bare `Solid3D` with
// only edge wires and was lost on save/reload. This test drives the real
// acadrust writers/readers for DXF and DWG and asserts the boolean geometry
// comes back as a renderable mesh.

use acadrust::{CadDocument, EntityType};
use OpenCADStudio::scene::mesh_model::MeshModel;
use OpenCADStudio::scene::mesh_tess::{mesh_entity_from_model, tessellate_mesh_entity};
use OpenCADStudio::scene::model_solid::{boolean, box_solid, Bool};
use OpenCADStudio::scene::truck_tess::{tessellate_solid, TruckTessResult};

/// Union two overlapping boxes and build the persistable Mesh entity exactly as
/// `solid_boolean` does (tessellate the truck result, then
/// `mesh_entity_from_model`).
fn boolean_mesh_entity(world_offset: [f64; 3]) -> (EntityType, usize) {
    let a = box_solid([0.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let b = box_solid([5.0, 5.0, 5.0], 10.0, 10.0, 10.0);
    let result = boolean(Bool::Union, &a, &b).expect("union produced a solid");

    let TruckTessResult::Mesh { verts, normals, indices } =
        tessellate_solid(&result, world_offset)
    else {
        panic!("boolean result did not tessellate to a mesh");
    };
    let tri_count = indices.len() / 3;
    assert!(tri_count > 0, "boolean result tessellated to no triangles");
    let model = MeshModel {
        name: String::new(),
        verts,
        normals,
        indices,
        color: [0.8, 0.8, 0.85, 1.0],
        selected: false,
    };
    (mesh_entity_from_model(&model, world_offset), tri_count)
}

fn tmp_path(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("ocs_bool_persist_{}_{}", std::process::id(), name));
    p
}

/// Save a document holding one boolean-result Mesh entity, reload it, and
/// return the reloaded triangle count.
fn round_trip(ext: &str) -> usize {
    // Non-zero offset: mesh_entity_from_model re-adds it so the stored entity is
    // in WCS, just like the real handlers.
    let (entity, _tris) = boolean_mesh_entity([100.0, 200.0, 0.0]);

    let mut doc = CadDocument::new();
    doc.add_entity(entity).expect("add boolean mesh entity");

    let path = tmp_path(&format!("bool.{ext}"));
    OpenCADStudio::io::save(&doc, &path).unwrap_or_else(|e| panic!("save {ext}: {e}"));
    let doc2 =
        OpenCADStudio::io::load_file(&path).unwrap_or_else(|e| panic!("load {ext}: {e}"));
    let _ = std::fs::remove_file(&path);

    let mesh_entity = doc2
        .entities()
        .find(|e| matches!(e, EntityType::Mesh(_)))
        .unwrap_or_else(|| panic!("no Mesh entity after {ext} round-trip"));

    let set = tessellate_mesh_entity(mesh_entity, [0.8, 0.8, 0.85, 1.0])
        .unwrap_or_else(|| panic!("reloaded {ext} boolean mesh did not tessellate"));
    set.lods[0].indices.len() / 3
}

#[test]
fn boolean_survives_dxf_round_trip() {
    let tris = round_trip("dxf");
    assert!(tris > 0, "dxf: boolean mesh lost its geometry");
}

#[test]
fn boolean_survives_dwg_round_trip() {
    let tris = round_trip("dwg");
    assert!(tris > 0, "dwg: boolean mesh lost its geometry");
}
