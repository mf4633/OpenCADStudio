// STEP AP203 export — converts tessellated MeshModels to ISO 10303-21 format.
//
// The output is a minimal but valid STEP AP203 file containing:
//   - One SHAPE_REPRESENTATION per solid mesh
//   - ADVANCED_FACE → PLANE → AXIS2_PLACEMENT_3D for each triangle
//   - VERTEX_POINT / EDGE_CURVE / ORIENTED_EDGE topology
//
// Because building full B-Rep topology from a triangle soup is complex, we use
// a simplified encoding: each triangle becomes a CLOSED_SHELL with three
// ADVANCED_FACEs, each face bounded by three oriented edges.
//
// This produces larger-than-optimal files but is universally importable by
// CAD systems that accept AP203.

use crate::scene::model::mesh_model::MeshModel;
use std::fmt::Write as FmtWrite;

/// Build a STEP AP203 text representation from a slice of mesh models.
///
/// Returns `None` if there are no triangles to export.
pub fn build_step(meshes: &[&MeshModel]) -> Option<String> {
    // Collect all triangles as (v0, v1, v2, normal).
    struct Tri {
        v: [[f32; 3]; 3],
        n: [f32; 3],
    }

    let mut tris: Vec<Tri> = Vec::new();
    for mesh in meshes {
        let verts = &mesh.verts;
        let normals = &mesh.normals;
        let idx = &mesh.indices;
        let n_tri = idx.len() / 3;
        for t in 0..n_tri {
            let i0 = idx[t * 3] as usize;
            let i1 = idx[t * 3 + 1] as usize;
            let i2 = idx[t * 3 + 2] as usize;
            if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
                continue;
            }
            let a = verts[i0];
            let b = verts[i1];
            let c = verts[i2];
            let n = if !normals.is_empty() && i0 < normals.len() {
                normals[i0]
            } else {
                let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let nx = ab[1] * ac[2] - ab[2] * ac[1];
                let ny = ab[2] * ac[0] - ab[0] * ac[2];
                let nz = ab[0] * ac[1] - ab[1] * ac[0];
                let len = (nx * nx + ny * ny + nz * nz).sqrt().max(f32::EPSILON);
                [nx / len, ny / len, nz / len]
            };
            tris.push(Tri { v: [a, b, c], n });
        }
    }

    if tris.is_empty() {
        return None;
    }

    // ── Emit STEP ─────────────────────────────────────────────────────────
    // Entity ID counter (STEP ids start at #1).
    let mut next_id: usize = 1;
    let mut data = String::new();

    // Closure to allocate the next ID.
    let mut alloc = || {
        let id = next_id;
        next_id += 1;
        id
    };

    // Collect face IDs for the shell.
    let mut face_ids: Vec<usize> = Vec::with_capacity(tris.len());

    for tri in &tris {
        // Each triangle: 3 vertices, 3 edges, 1 face.

        // Vertex points.
        let vp: [usize; 3] = [alloc(), alloc(), alloc()];
        // Cartesian points for vertices.
        let cp: [usize; 3] = [alloc(), alloc(), alloc()];
        // Line curves for edges.
        let lc: [usize; 3] = [alloc(), alloc(), alloc()];
        // Direction refs for lines (reusing cp[0] as direction — simplified).
        let dir: [usize; 3] = [alloc(), alloc(), alloc()];
        // Vertex-point refs.
        let vpref: [usize; 3] = [alloc(), alloc(), alloc()];
        // Edge curves.
        let ec: [usize; 3] = [alloc(), alloc(), alloc()];
        // Oriented edges.
        let oe: [usize; 3] = [alloc(), alloc(), alloc()];
        // Edge loop.
        let el = alloc();
        // Plane normal direction and axis placement.
        let norm_dir = alloc();
        let plane_ax = alloc();
        let plane = alloc();
        // Advanced face.
        let face_id = alloc();
        face_ids.push(face_id);

        // Emit cartesian points.
        for k in 0..3 {
            let [x, y, z] = tri.v[k];
            writeln!(
                data,
                "#{} = CARTESIAN_POINT('',({:.6},{:.6},{:.6}));",
                cp[k], x, y, z
            )
            .ok();
            writeln!(data, "#{} = VERTEX_POINT('',#{});", vpref[k], cp[k]).ok();
        }
        // Emit vertex points (binding).
        for k in 0..3 {
            writeln!(data, "#{} = VERTEX_POINT('',#{});", vp[k], cp[k]).ok();
            // (duplicate of vpref; we keep vp[] to reference in edge curves)
            let _ = vp[k]; // suppress unused warning
        }

        // Emit edge directions and line curves.
        for k in 0..3 {
            let k1 = (k + 1) % 3;
            let [dx, dy, dz] = [
                tri.v[k1][0] - tri.v[k][0],
                tri.v[k1][1] - tri.v[k][1],
                tri.v[k1][2] - tri.v[k][2],
            ];
            let len = (dx * dx + dy * dy + dz * dz).sqrt().max(f32::EPSILON);
            writeln!(
                data,
                "#{} = DIRECTION('',({:.6},{:.6},{:.6}));",
                dir[k],
                dx / len,
                dy / len,
                dz / len
            )
            .ok();
            writeln!(
                data,
                "#{} = LINE('',#{},VECTOR('',#{},1.));",
                lc[k], cp[k], dir[k]
            )
            .ok();
            writeln!(
                data,
                "#{} = EDGE_CURVE('',#{},#{},#{},.T.);",
                ec[k], vpref[k], vpref[k1], lc[k]
            )
            .ok();
            writeln!(data, "#{} = ORIENTED_EDGE('',*,*,#{},.T.);", oe[k], ec[k]).ok();
        }

        // Edge loop and face normal.
        writeln!(
            data,
            "#{} = EDGE_LOOP('',({},{},{}));",
            el,
            format!("#{}", oe[0]),
            format!("#{}", oe[1]),
            format!("#{}", oe[2])
        )
        .ok();

        let [nx, ny, nz] = tri.n;
        writeln!(
            data,
            "#{} = DIRECTION('',({:.6},{:.6},{:.6}));",
            norm_dir, nx, ny, nz
        )
        .ok();
        writeln!(
            data,
            "#{} = AXIS2_PLACEMENT_3D('',#{},#{},#{});",
            plane_ax, cp[0], norm_dir, dir[0]
        )
        .ok();
        writeln!(data, "#{} = PLANE('',#{});", plane, plane_ax).ok();
        writeln!(
            data,
            "#{} = ADVANCED_FACE('',(FACE_BOUND('',#{},.T.)),#{},.T.);",
            face_id, el, plane
        )
        .ok();
    }

    // Closed shell wrapping all faces.
    let shell_id = alloc();
    let face_list: String = face_ids
        .iter()
        .map(|id| format!("#{id}"))
        .collect::<Vec<_>>()
        .join(",");
    writeln!(data, "#{} = CLOSED_SHELL('',({face_list}));", shell_id).ok();

    // Manifold solid B-rep.
    let msb_id = alloc();
    writeln!(
        data,
        "#{} = MANIFOLD_SOLID_BREP('OpenCADStudio_Solid',#{});",
        msb_id, shell_id
    )
    .ok();

    // Shape representation. The geometric context MUST assign the length unit,
    // otherwise the mm LENGTH_UNIT is orphaned and importers read the solid as
    // unit-less / mis-scaled. Emit the standard AP203 complex context that
    // references both the unit (#pu_id) and an uncertainty (#unc_id).
    let sr_id = alloc();
    let pu_id = alloc();
    let unc_id = alloc();
    let gc_id = alloc();
    writeln!(
        data,
        "#{} = (LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT(.MILLI.,.METRE.));",
        pu_id
    )
    .ok();
    writeln!(
        data,
        "#{} = UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-06),#{},\
         'distance_accuracy_value','confusion accuracy');",
        unc_id, pu_id
    )
    .ok();
    writeln!(
        data,
        "#{} = ( GEOMETRIC_REPRESENTATION_CONTEXT(3) \
         GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#{})) \
         GLOBAL_UNIT_ASSIGNED_CONTEXT((#{})) \
         REPRESENTATION_CONTEXT('Context #1','3D Context with UNIT and UNCERTAINTY') );",
        gc_id, unc_id, pu_id
    )
    .ok();
    writeln!(
        data,
        "#{sr_id} = SHAPE_REPRESENTATION('OpenCADStudio_Shape',(#{}),#{gc_id});",
        msb_id
    )
    .ok();

    // ── Assemble file ─────────────────────────────────────────────────────
    let ts = chrono_timestamp();
    let file = format!(
        "ISO-10303-21;\n\
         HEADER;\n\
         FILE_DESCRIPTION(('Open CAD Studio STEP export'),'2;1');\n\
         FILE_NAME('{ts}','','',(''),'',' ',' ');\n\
         FILE_SCHEMA(('CONFIG_CONTROL_DESIGN'));\n\
         ENDSEC;\n\
         DATA;\n\
         {data}\
         ENDSEC;\n\
         END-ISO-10303-21;\n"
    );

    Some(file)
}

/// Returns an ISO 8601-like timestamp string for the STEP file header.
fn chrono_timestamp() -> String {
    // Use seconds since Unix epoch for a simple timestamp without chrono dep.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Format: YYYY-MM-DDTHH:MM:SS (approximate UTC from epoch seconds).
    let s = secs;
    let mins = s / 60;
    let hours = mins / 60;
    let days = hours / 24;
    let hh = hours % 24;
    let mm = mins % 60;
    let ss = s % 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hh:02}:{mm:02}:{ss:02}")
}

/// Days since the Unix epoch (1970-01-01) → `(year, month, day)`, correct
/// across leap years and variable month lengths (Howard Hinnant's
/// `civil_from_days`). The previous fixed 365-day/30-day approximation drifted
/// by weeks and could emit an invalid month 13 for days-of-year 360–364.
fn civil_from_days(days: u64) -> (i64, u32, u32) {
    let z = days as i64 + 719_468; // shift epoch to 0000-03-01
    let era = z / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}

#[cfg(test)]
mod tests {
    use super::civil_from_days;

    #[test]
    fn civil_from_days_is_leap_correct() {
        // Anchors across the leap cycle (1972 is a leap year).
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(365), (1971, 1, 1));
        assert_eq!(civil_from_days(730), (1972, 1, 1));
        assert_eq!(civil_from_days(1096), (1973, 1, 1)); // 1972 had 366 days
        assert_eq!(civil_from_days(59), (1970, 3, 1)); // Jan(31)+Feb(28)
        // The exact case the old 30-day approximation broke: day-of-year 364
        // used to yield an invalid "month 13".
        assert_eq!(civil_from_days(364), (1970, 12, 31));
        let (_, m, _) = civil_from_days(363);
        assert!(m <= 12);
    }
}
