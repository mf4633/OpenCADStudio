# Rendering Optimization Roadmap: Culling & Level of Detail

## Background

H7CAD renders all entities every frame regardless of camera position or zoom level. No spatial
acceleration, no LOD, no frustum culling. Every wire, hatch, mesh, and image is uploaded and
drawn unconditionally. This scales poorly for large drawings (100k+ entities) and dense 3D solids.

Current pipeline order:
1. Hatch fills → 2. Images → 3. Meshes → 4. Face3D fills → 5. Face3D edges →
6. Wires → 7. Wipeouts → 8. Selection overlay → 9. MSAA resolve → 10. Blit

---

## Phase 4 — GPU-Side Culling (Advanced)

**Goal:** Offload culling to the GPU; zero CPU cost for large entity counts.

### 4.1 Indirect Draw + Compute Cull

Convert per-entity draw calls to indirect draw calls (`draw_indirect` / `draw_indexed_indirect`).
Run a compute shader pre-pass that tests each entity's AABB against the frustum and writes
`DrawIndirectArgs` only for visible entities.

```wgsl
// cull.wgsl
@compute @workgroup_size(64)
fn cull_entities(@builtin(global_invocation_id) id: vec3<u32>) {
    let entity = entities[id.x];
    if frustum_test(entity.aabb) {
        // atomically append to indirect draw buffer
        let slot = atomicAdd(&draw_count, 1u);
        draw_args[slot] = entity.draw_args;
    }
}
```

Requires restructuring entity data into GPU-side storage buffers. High complexity; tackle after
Phases 1–3 prove insufficient.

### 4.2 Hierarchical Z-Buffer Occlusion Culling (3D Only)

For dense 3D solid scenes, use a Hi-Z buffer to cull occluded meshes:

1. Render depth-only pass for large opaque solids.
2. Downsample depth into mip chain (Hi-Z pyramid).
3. Compute shader tests each mesh AABB against Hi-Z; skips occluded meshes.

Relevant only for perspective (3D) mode with many overlapping solids.

---

## Implementation Order

```
Phase 4.1  Indirect draw + GPU cull         high complexity, defer
Phase 4.2  Hi-Z occlusion                   high complexity, 3D only, last
```

---

## Key Files to Modify

| File | Change |
|------|--------|
| `src/scene/mod.rs` | Quadtree/octree; LOD epoch tracking |
| `src/scene/pipeline/mod.rs` | Cull hatch/image entities before upload loops |
| `src/shaders/cull.wgsl` | New — Phase 4 compute culling shader |

---

## Success Metrics

- **Phase 2 target:** Pan/zoom frame cost O(visible) not O(total).
- **Phase 4 target:** GPU-cull overhead < 0.5 ms for 1M entity scene.

