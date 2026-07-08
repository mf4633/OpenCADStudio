// Persistable-mesh tessellation: acadrust `Mesh` (ACAD_MESH / AcDbSubDMesh)
// entity <-> scene `MeshModel`.
//
// EXTRUDE / REVOLVE / SWEEP / LOFT build a truck B-rep and tessellate it to a
// triangle mesh. truck B-reps cannot be written back to DWG/DXF as ACIS, so a
// solid stored under a placeholder empty `Solid3D` was lost on save/reload.
// Instead we persist the triangle tessellation as a real `Mesh` entity — both
// the DWG and DXF writers serialize it (ACAD_MESH / AcDbSubDMesh) and both
// readers parse it back — and re-tessellate it into a shaded `MeshModel` on
// load, exactly like the ACIS solid path.

use acadrust::entities::Mesh;
use acadrust::types::{Color, Vector3};
use acadrust::EntityType;

use crate::scene::mesh_model::{MeshLodSet, MeshModel};

/// Build a persistable `Mesh` entity from a scene-local `MeshModel`.
///
/// `MeshModel` vertices are in local space (WCS − `world_offset`); the offset
/// is added back so the stored entity carries WCS coordinates, matching every
/// other document entity. The requested colour is stored as a true colour so
/// `render_style` resolves the reloaded mesh back to the same look.
pub fn mesh_entity_from_model(mesh: &MeshModel, world_offset: [f64; 3]) -> EntityType {
    let [ox, oy, oz] = world_offset;
    let verts: Vec<Vector3> = mesh
        .verts
        .iter()
        .map(|&[x, y, z]| Vector3::new(x as f64 + ox, y as f64 + oy, z as f64 + oz))
        .collect();
    let tris: Vec<(usize, usize, usize)> = mesh
        .indices
        .chunks_exact(3)
        .map(|t| (t[0] as usize, t[1] as usize, t[2] as usize))
        .collect();

    // `from_triangles` also computes the edge list the writers need.
    let mut m = Mesh::from_triangles(verts, &tris);
    let [r, g, b, _] = mesh.color;
    m.common.color = Color::Rgb {
        r: (r.clamp(0.0, 1.0) * 255.0).round() as u8,
        g: (g.clamp(0.0, 1.0) * 255.0).round() as u8,
        b: (b.clamp(0.0, 1.0) * 255.0).round() as u8,
    };
    EntityType::Mesh(m)
}

/// Tessellate a `Mesh` entity into a WORLD-space single-LOD `MeshLodSet`.
///
/// Mirrors the ACIS solid path: the caller subtracts `world_offset` via
/// `offset_mesh_lod_set`. Faces with more than three vertices are
/// fan-triangulated; per-vertex normals are area-weighted from the incident
/// triangles. Returns `None` for a non-`Mesh` entity or one with no geometry.
pub fn tessellate_mesh_entity(entity: &EntityType, color: [f32; 4]) -> Option<MeshLodSet> {
    let mesh = match entity {
        EntityType::Mesh(m) => m,
        _ => return None,
    };
    if mesh.vertices.is_empty() || mesh.faces.is_empty() {
        return None;
    }

    let verts: Vec<[f32; 3]> = mesh
        .vertices
        .iter()
        .map(|v| [v.x as f32, v.y as f32, v.z as f32])
        .collect();

    let mut indices: Vec<u32> = Vec::new();
    for face in &mesh.faces {
        let vs = &face.vertices;
        if vs.len() < 3 {
            continue;
        }
        // Fan-triangulate around the first vertex; skip any face index that
        // falls outside the vertex list rather than panicking on bad data.
        for k in 1..vs.len() - 1 {
            let (a, b, c) = (vs[0], vs[k], vs[k + 1]);
            if a < verts.len() && b < verts.len() && c < verts.len() {
                indices.push(a as u32);
                indices.push(b as u32);
                indices.push(c as u32);
            }
        }
    }
    if indices.is_empty() {
        return None;
    }

    let normals = compute_vertex_normals(&verts, &indices);
    let model = MeshModel {
        name: String::new(),
        verts,
        normals,
        indices,
        color,
        selected: false,
    };
    Some(MeshLodSet::from_single(model))
}

/// Area-weighted per-vertex normals from a triangle-indexed mesh. The cross
/// product magnitude equals twice the triangle area, so larger triangles
/// contribute proportionally more to each shared vertex.
fn compute_vertex_normals(verts: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; verts.len()];
    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let a = verts[i0];
        let b = verts[i1];
        let c = verts[i2];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        for &idx in &[i0, i1, i2] {
            normals[idx][0] += n[0];
            normals[idx][1] += n[1];
            normals[idx][2] += n[2];
        }
    }
    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-12 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        } else {
            *n = [0.0, 0.0, 1.0];
        }
    }
    normals
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tri_model() -> MeshModel {
        MeshModel {
            name: String::new(),
            verts: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![],
            indices: vec![0, 1, 2],
            color: [0.25, 0.5, 0.75, 1.0],
            selected: false,
        }
    }

    #[test]
    fn round_trips_geometry_and_offset() {
        let model = unit_tri_model();
        // Store with a world offset; the entity must carry WCS coordinates.
        let entity = mesh_entity_from_model(&model, [10.0, 20.0, 30.0]);
        let EntityType::Mesh(ref m) = entity else {
            panic!("expected a Mesh entity")
        };
        assert_eq!(m.vertices.len(), 3);
        assert_eq!(m.faces.len(), 1);
        assert!((m.vertices[1].x - 11.0).abs() < 1e-6); // 1 + 10
        assert!((m.vertices[2].y - 21.0).abs() < 1e-6); // 1 + 20

        // Re-tessellate: vertices come back in WCS, ready for offset_mesh_lod_set.
        let set = tessellate_mesh_entity(&entity, model.color).unwrap();
        let back = &set.lods[0];
        assert_eq!(back.indices, vec![0, 1, 2]);
        assert!((back.verts[1][0] - 11.0).abs() < 1e-4);
        assert_eq!(back.color, model.color);
        // A normal was synthesised for every vertex (triangle lies in Z=0 plane).
        assert_eq!(back.normals.len(), 3);
        assert!((back.normals[0][2].abs() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn color_is_stored_as_true_color() {
        let entity = mesh_entity_from_model(&unit_tri_model(), [0.0, 0.0, 0.0]);
        let EntityType::Mesh(ref m) = entity else {
            panic!("expected a Mesh entity")
        };
        assert_eq!(m.common.color, Color::Rgb { r: 64, g: 128, b: 191 });
    }

    #[test]
    fn quad_face_is_fan_triangulated() {
        let mut entity = mesh_entity_from_model(&unit_tri_model(), [0.0, 0.0, 0.0]);
        if let EntityType::Mesh(ref mut m) = entity {
            m.vertices.push(Vector3::new(1.0, 1.0, 0.0));
            m.faces.clear();
            m.faces
                .push(acadrust::entities::MeshFace::new(vec![0, 1, 3, 2]));
        }
        let set = tessellate_mesh_entity(&entity, [1.0, 1.0, 1.0, 1.0]).unwrap();
        // One quad → two triangles → six indices.
        assert_eq!(set.lods[0].indices.len(), 6);
    }

    #[test]
    fn non_mesh_entity_is_none() {
        let line = EntityType::Line(acadrust::entities::Line::new());
        assert!(tessellate_mesh_entity(&line, [1.0, 1.0, 1.0, 1.0]).is_none());
    }
}
