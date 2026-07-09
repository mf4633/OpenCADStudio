// 3D solid modelling support on the App: committing Model-tab primitives,
// and the Design-group boolean operations (truck-shapeops) over the scene's
// session-cached truck B-reps.

use acadrust::{EntityType, Handle};
use iced::Task;
use truck_modeling::Solid;

use super::Message;
use crate::modules::model::boolean_cmd::BoolOp;
use crate::scene::model_solid::{self, Bool};

impl super::OpenCADStudio {
    /// Commit a Model-tab solid: add its acadrust entity to the document, then
    /// register the truck B-rep (caches it for booleans + tessellates it into
    /// the shaded mesh pipeline). Returns the new entity handle.
    pub(super) fn add_model_solid(&mut self, entity: EntityType, solid: Solid) -> Handle {
        let i = self.active_tab;
        let Some(handle) = self.commit_entity_handle(entity) else {
            return Handle::NULL;
        };
        self.tabs[i].scene.register_model_solid(handle, solid);
        handle
    }

    /// Run a boolean (`union` / `subtract` / `intersect`) on exactly two
    /// selected solids whose truck B-reps are in the session cache.
    pub(super) fn solid_boolean(&mut self, op: BoolOp) -> Task<Message> {
        let i = self.active_tab;
        // Selected entities that have a cached truck B-rep.
        let handles: Vec<Handle> = self.tabs[i]
            .scene
            .selected
            .iter()
            .copied()
            .filter(|h| self.tabs[i].scene.model_solids.contains_key(h))
            .collect();
        if handles.len() != 2 {
            self.command_line.push_error(
                "Boolean: select exactly two solids created this session.",
            );
            return Task::none();
        }
        let a = self.tabs[i].scene.model_solids[&handles[0]].clone();
        let b = self.tabs[i].scene.model_solids[&handles[1]].clone();
        let kind = match op {
            BoolOp::Union => Bool::Union,
            BoolOp::Subtract => Bool::Subtract,
            BoolOp::Intersect => Bool::Intersect,
        };
        let Some(result) = model_solid::boolean(kind, &a, &b) else {
            self.command_line
                .push_error("Boolean failed — the solids may not overlap.");
            return Task::none();
        };

        // Tessellate the truck result up front so we can bail before mutating
        // if it produces no geometry.
        let woff = self.tabs[i].scene.world_offset;
        let color = self.tabs[i].scene.entity_resolved_color(handles[0]);
        let mesh = match crate::scene::truck_tess::tessellate_solid(&result, woff) {
            crate::scene::truck_tess::TruckTessResult::Mesh {
                verts,
                normals,
                indices,
            } => crate::scene::mesh_model::MeshModel {
                name: String::new(),
                verts,
                normals,
                indices,
                color,
                selected: false,
            },
            _ => {
                self.command_line
                    .push_error("Boolean: result could not be tessellated.");
                return Task::none();
            }
        };

        self.push_undo_snapshot(i, "BOOLEAN");
        // Remove the two operands (entity + mesh + cached B-rep).
        self.tabs[i].scene.erase_entities(&handles);
        // truck B-reps can't be written back to DWG/DXF as ACIS, so a bare
        // Solid3D result (wires only, no body) was lost on save/reload and
        // rebuilt to nothing on undo/redo. Persist the tessellation as a real
        // Mesh (ACAD_MESH) entity — add_entity re-tessellates it into
        // scene.meshes for shaded display, exactly like EXTRUDE/REVOLVE — and
        // keep the truck B-rep in the session model_solids cache keyed by the
        // new handle so the result stays booleanable for the rest of the
        // session.
        let entity = crate::scene::mesh_tess::mesh_entity_from_model(&mesh, woff);
        let handle = self.tabs[i].scene.add_entity(entity);
        self.tabs[i].scene.deselect_all();
        if !handle.is_null() {
            self.tabs[i].scene.model_solids.insert(handle, result);
            self.tabs[i].scene.select_entity(handle, false);
        }
        self.tabs[i].dirty = true;
        self.refresh_properties();
        Task::none()
    }
}
