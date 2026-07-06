# Volumes

A **Volume** is the top-level container: one entire [QuadTree](quadtree.md)
of [Domains](planes-and-domains.md), placed and oriented somewhere in the
world.

```rust
struct GlVolume {
    base_node_idx: u32,             // root node of this volume's quadtree
    world_to_volume: [[f32; 4]; 4], // world space -> volume-local space
    volume_to_world: [[f32; 4]; 4], // volume-local space -> world space
    min_p: [f32; 3],                // world-space AABB, for cheap culling
    max_p: [f32; 3],
}
```

Think of a Volume as a small, self-contained procedural landscape: its
QuadTree is defined in its own local coordinate space (conventionally, XY in
`[-0.5, 0.5]`, Z unconstrained), and `volume_to_world`/`world_to_volume`
place that local space anywhere in the scene — translated, rotated, scaled.

A scene can contain multiple Volumes, each independently transformed, each
with its own QuadTree.

## Why a separate transform layer, rather than baking geometry into world
space directly?

- Each Volume's QuadTree, Domains, and Planes only ever need to reason about
  a single, consistent local coordinate space — the traversal and
  root-finding math in [Rendering Pipeline](../rendering/pipeline.md) never
  needs to know or care where the Volume actually sits in the world.
- The same Volume data could, in principle, be instanced multiple times at
  different transforms without duplicating the underlying QuadTree/Plane
  data — the transform is the only thing that differs per-instance. (Kraken
  doesn't yet do this, but the structure supports it.)
- A world-space AABB (`min_p`/`max_p`) is cheap to test *before* ever
  transforming a ray into the Volume's local space — see
  [Rendering Pipeline](../rendering/pipeline.md) for where this is used to
  skip whole Volumes that a ray can't possibly reach.

## Rendering, at the Volume level

1. Test the ray against the Volume's world-space AABB. Miss → skip this
   Volume entirely, no further work.
2. Transform the ray into the Volume's local space via `world_to_volume`.
3. Traverse the Volume's QuadTree (see [QuadTree](quadtree.md)) using the
   ray's local XY.
4. On a hit, transform the resulting point and normal back into world space
   via `volume_to_world` (and its inverse-transpose, for the normal) before
   returning.

With multiple Volumes in a scene, the renderer repeats this per-Volume and
keeps the closest hit across all of them — see
[Rendering Pipeline](../rendering/pipeline.md) for the full, combined
algorithm.