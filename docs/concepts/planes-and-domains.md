# Planes & Domains

## Plane

A **Plane** is a single, bounded instance of the height function described in
[Geometry Model](geometry-model.md) — a 3×3 grid of control points, plus a
base `height` offset.

```rust
struct GlPlane {
    height: f32,   // base height this patch's control points are offset from
    max_h: f32,    // highest point the patch can reach (height + max control point)
    min_h: f32,    // lowest point the patch can reach (height + min control point)
    first_cp_idx: u32, // index into the shared control-point buffer
}
```

`min_h`/`max_h` are precomputed once, at scene-build time, from the control
points themselves. They exist purely as a cheap conservative bound: before
ever evaluating the actual bicubic function, the renderer can reject a plane
outright if a ray's height range doesn't overlap `[min_h, max_h]` at all —
avoiding the cost of even a single Newton iteration for planes the ray can't
possibly hit. See [Rendering Pipeline](../rendering/pipeline.md) for exactly
where this check happens.

A Plane is *local* — its `(x, y)` domain is normalized to `[0, 1] × [0, 1]`,
scaled and positioned by whatever [Domain](#domain) contains it.

## Domain

Planes don't stack arbitrarily — within a single spatial cell, they're
**stacked by height, sorted, and non-overlapping**. A Domain (represented in
code as a leaf `QuadNode`) is that stack:

```rust
struct GlQuadNode {
    children: u32,     // 0 = leaf (has planes), nonzero = internal (has 4 children)
    first_idx: u32,    // for a leaf: index into the plane buffer
                        // for an internal node: index into the quad-node buffer
    plane_count: u32,  // number of planes in this domain (leaves only)
    position: [f32; 2],// domain's origin in its parent's local XY space
    size: f32,         // domain's width/height (domains are always square)
}
```

At scene-build time, planes assigned to the same domain are asserted to be
sorted by height with no overlap between consecutive planes' `[min_h, max_h]`
ranges — this invariant is what lets the renderer scan them in a single
height-ordered pass per ray (ascending or descending, depending on the ray's
direction) instead of testing every plane in the domain unconditionally. See
[Rendering Pipeline](../rendering/pipeline.md) for how the scan direction and
early-exit ("this plane's height range is entirely past where the ray could
still be going, so nothing after it can matter either") are derived from
that sort order.

Practically: a Domain answers "in this one column of XY space, what are the
(possibly several, stacked) surfaces a ray could pass through, from lowest to
highest?"

## Why split geometry this way?

- **Planes** keep the actual curved-surface math (Newton root-finding)
  scoped to a single, small, well-conditioned patch.
- **Domains** let multiple stacked surfaces exist in the same XY footprint —
  a canyon floor and an overhang above it, for example — without needing a
  single height function to represent both.
- Domains are the leaves of a [QuadTree](quadtree.md), which is what actually
  lets the renderer skip regions of space that have no geometry at all,
  rather than testing every Domain in the scene for every ray.