// Triangle mesh model — produced by truck Shell/Solid tessellation.
//
// Stored alongside WireModels in the scene; rendered by the mesh pipeline
// (wgpu TriangleList with depth test, flat normals).

/// A tessellated triangle mesh ready to upload to the GPU.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MeshModel {
    /// Unique identifier (entity handle value as decimal string).
    pub name: String,
    /// World-space vertex positions.
    pub verts: Vec<[f32; 3]>,
    /// Per-vertex normals (may be empty if not available).
    pub normals: Vec<[f32; 3]>,
    /// Triangle indices into `verts` (every 3 values = one triangle).
    pub indices: Vec<u32>,
    /// RGBA colour in [0, 1].
    pub color: [f32; 4],
    /// Whether this mesh is currently selected.
    pub selected: bool,
}

/// Bundle of mesh tessellations at different sampling densities, picked
/// per frame by the render pipeline based on the projected pixel size of
/// `world_aabb`. Phase 3.4 LOD ladder:
///
/// | LOD | Source     | Use when projected diagonal |
/// |-----|------------|------------------------------|
/// | 0   | HIGH       | > 200 px                     |
/// | 1   | MID (½)    | 50–200 px                    |
/// | 2   | LOW (¼)    | < 50 px                      |
///
/// `lods` holds up to one MeshModel per LOD level (high → low). Empty
/// slots fall back to the nearest available LOD at render time.
#[derive(Clone, Debug)]
pub struct MeshLodSet {
    pub lods: Vec<MeshModel>,
    /// World XY AABB `[min_x, min_y, max_x, max_y]` of the mesh — used
    /// by the per-frame LOD selector to compute the projected pixel
    /// diagonal.
    pub world_aabb: [f32; 4],
    /// World Z extent `[min_z, max_z]`. The cull / LOD projection needs it:
    /// a mesh elevated off the z=0 plane projects to a different screen
    /// position in a tilted 3D view, so projecting the XY box at z=0 alone
    /// mis-culls and mis-LODs it. `[0.0, 0.0]` for a flat z=0 mesh.
    pub z_range: [f32; 2],
}

/// World Z extent `[min_z, max_z]` over a set of mesh vertices (non-finite
/// values skipped). Returns `[0.0, 0.0]` when no finite vertex is present.
pub fn verts_z_range<'a>(verts: impl IntoIterator<Item = &'a [f32; 3]>) -> [f32; 2] {
    let mut min_z = f32::INFINITY;
    let mut max_z = f32::NEG_INFINITY;
    for v in verts {
        let z = v[2];
        if !z.is_finite() {
            continue;
        }
        if z < min_z { min_z = z; }
        if z > max_z { max_z = z; }
    }
    if min_z.is_finite() {
        [min_z, max_z]
    } else {
        [0.0, 0.0]
    }
}

impl MeshLodSet {
    /// Wrap a single MeshModel as a one-LOD set. Used by interactive
    /// commands that only produce one tessellation (e.g. truck-based
    /// BOX/CYLINDER creation). The LOD selector will pick slot 0 for
    /// every zoom level.
    pub fn from_single(mesh: MeshModel) -> Self {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for &[x, y, _] in &mesh.verts {
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            if x < min_x { min_x = x; }
            if y < min_y { min_y = y; }
            if x > max_x { max_x = x; }
            if y > max_y { max_y = y; }
        }
        let z_range = verts_z_range(&mesh.verts);
        Self {
            lods: vec![mesh],
            world_aabb: [min_x, min_y, max_x, max_y],
            z_range,
        }
    }
}
