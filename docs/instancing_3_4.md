# Roadmap 3.4 — Hardware instancing for repeated block inserts

Design + implementation notes for instancing repeated `INSERT`s in the wire
pipeline. Tracked because the change spans tessellation **and** the GPU wire
pipeline and cannot be verified in headless CI — it needs the app run against a
block-heavy drawing.

## Problem

The wire pipeline is already *segment*-instanced: `WireInstance`
(`src/scene/pipeline/wire_gpu.rs`) is one **world-space** segment
(`pos_a`/`pos_b` + colour / half-width / pattern / draw-depth), drawn with
`pass.draw(0..6, 0..instance_count)`.

But `expand_insert` (`src/scene/block_cache.rs:602`) **bakes each insert's
transform into world-space segments**. A block placed N times therefore uploads
N copies of its segment geometry. On architectural drawings (every door /
window / fixture is the same block) that is the dominant wire cost.

## Goal

Upload a block definition's geometry **once, in block-local coordinates**, plus
a per-insert transform buffer; the vertex shader transforms local → world per
instance. At 100 repeated inserts this collapses ~100× the vertex upload and
draw work into one instanced draw.

## Design

Additive — a new path gated behind an eligibility check, with the existing
`expand_insert` world-bake path as the fallback. Nothing is rewritten.

### 1. GPU path — `InstancedBlockGpu` (new, beside `WireGpu`)

- **`defn_segments`** storage buffer — the defn's segments in **block-local**
  coords, each carrying colour / half-width / pattern / draw-depth and the
  **local** cumulative distance for dashing. Built once per eligible defn.
- **`insert_transforms`** buffer — one entry per *visible* insert:
  - a 2×3 affine (6×`f32`) for the common planar case, or a `mat4` if rotated /
    3D blocks are in scope;
  - per-insert overrides: resolved **ByBlock** colour, `draw_depth`, and the
    **scale factor** used to correct dash spacing.
- **Draw**: `pass.draw(0..6, 0..(defn_seg_count * visible_inserts))`. The
  vertex shader decodes `seg = instance_index % defn_seg_count`,
  `insert = instance_index / defn_seg_count`, reads the local segment + the
  insert transform, and emits the world-space quad — same quad expansion the
  current wire shader already does, with a transform applied first.

### 2. Eligibility gate (tessellation assembly)

Instance a defn only when it is:
- INSERTed **≥ THRESHOLD** times in the visible set (tune; start ~8),
- a **leaf** defn, or one-level-flattened (nested INSERTs either force
  fallback or are flattened once at build),
- free of per-insert **geometry-changing** overrides (exploded inserts,
  attribute-driven geometry).

Everything failing the gate renders through the existing `expand_insert` path.
The two paths co-exist in the same frame.

### 3. Culling / LOD

Reuse the existing per-insert AABBs (`compute_block_aabbs`) to upload only
visible inserts' transforms. Instancing uses the defn-build tessellation
tolerance (no per-insert zoom-adaptive LOD) — acceptable, and consistent with
how block defns are already tessellated once.

## Risk areas (verify visually)

1. **Dash / linetype spacing** — dash distance is arc-length based; it must be
   computed defn-local and **scaled by the insert scale in the shader**, or
   dashed linetypes inside scaled blocks space incorrectly.
2. **ByBlock colour** — resolves per insert → per-insert override in the
   instance buffer. **ByLayer** stays resolved in the defn geometry.
3. **Non-uniform scale / rotation / mirroring** — transform correctness,
   including negative determinants (mirrored blocks).
4. **Draw order / scissor / depth-bias** — the instanced draws must interleave
   correctly with the batched non-instanced runs (3.3) and the separate
   preview/overlay buffer, preserving depth bias and alpha.
5. **Selection / hover / grip** — a selected or grip-dragged insert must fall
   back to the non-instanced path (its geometry changes per frame).

## Verification (requires running the app)

1. Open an architectural DWG with many repeated door/window blocks.
2. Confirm **pixel parity** against `main` (toggle the feature off/on).
3. `PERF` HUD: `tess ms` and draw-call count should drop sharply.
4. Test dashed linetypes inside scaled blocks (risk #1).
5. Test ByBlock-coloured blocks on several layers (risk #2).
6. Test mirrored / rotated / non-uniformly scaled inserts (risk #3).
7. Select / hover / grip-move an instanced insert — must highlight and edit
   correctly via the fallback (risk #5).
8. Enter a paper-space viewport containing instanced blocks — projection parity.

## Rough size

One new WGSL shader + ~300–500 lines across `pipeline/mod.rs`,
`pipeline/wire_gpu.rs`, `block_cache.rs`, and the tessellation assembly. The
CPU-side eligibility analysis and defn-local geometry extraction are
unit-testable; the shader / draw path is what needs the running-app pass.
