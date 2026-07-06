# Rendering Pipeline

This describes how a single ray is resolved to a surface hit, start to
finish — tying together [Volumes](../concepts/volumes.md),
[QuadTree traversal](../concepts/quadtree.md), and
[Planes](../concepts/planes-and-domains.md) into the concrete algorithm.

## 1. World traversal

For a ray cast into the scene, the renderer iterates every
[Volume](../concepts/volumes.md) and:

1. Tests the ray against the Volume's world-space AABB (`min_p`/`max_p`) —
   a cheap slab test that rejects Volumes the ray can't possibly reach
   before paying for any transform or traversal work.
2. On a possible overlap, transforms the ray into the Volume's local space
   and traverses its QuadTree (step 2 below).
3. Keeps the closest hit found across all Volumes.

## 2. QuadTree traversal (per Volume)

Within a Volume's local space, an iterative, stack-based walk (see
[QuadTree](../concepts/quadtree.md) for the full mechanics) narrows down
which leaf **Domain** the ray's XY position could pass through, testing ray
vs. axis-aligned XY squares at each level and descending nearest-child-first.

When a leaf is reached, control passes to Domain-level testing (step 3).

## 3. Domain: scanning stacked Planes

A Domain (leaf `QuadNode`) holds a **height-sorted stack** of
[Planes](../concepts/planes-and-domains.md) — see that page for the
sortedness/non-overlap invariant this relies on.

The scan direction depends on the ray's vertical direction: if the ray's Z
is increasing along its path, planes are visited low-to-high (the order the
ray physically reaches them); if decreasing, high-to-low. For each plane in
that order:

- **Skip** it if the ray's height range over the current traversal segment
  doesn't reach that plane's `[min_h, max_h]` bound yet, but a later
  (further along the scan direction) plane still might.
- **Stop** scanning entirely if the plane's bound is already past every
  remaining height the ray could reach in this segment — because planes are
  height-sorted, every subsequent plane is even further out of range, so
  there's nothing left to check.
- Otherwise, the plane is a real candidate: hand off to Newton root-finding
  (step 4).

This early-exit is the payoff of the sortedness invariant: without it, every
plane in a Domain would need to be tested unconditionally.

## 4. Plane: Newton root-finding

Within a single candidate Plane, the exact intersection point is found by
solving

```
implicit_surface_function(t) = height(x(t), y(t)) - z(t) = 0
```

for the ray parameter `t`, where `(x(t), y(t), z(t))` is the ray's position
at distance `t`, and `height(...)` is the biquadratic patch evaluation
described in [Geometry Model](../concepts/geometry-model.md).

Newton's method iterates:

```
t_next = t - isf(t) / isf'(t)
```

where `isf'(t)` (the derivative along the ray) comes from the patch's
analytic gradient, dotted with the ray's local direction. Because the patch
is smooth and (for a flat or gently-curved patch) close to linear, this
typically converges in very few iterations — for an exactly flat patch, the
very first step lands on the exact root, since a linear function's tangent
*is* the function.

Convergence is checked against a distance-scaled epsilon (tighter
tolerance for rays that have already traveled further, to keep floating
point precision meaningful at longer path lengths), and iteration is capped
by a configurable maximum step count to bound worst-case cost per pixel.

On convergence, the surface normal is derived directly from the patch's
analytic gradient at the hit point (no finite differencing needed).

## 5. Combining with other scene elements

Implicit-surface hits are combined with any other intersectable scene
content (for example, analytic light spheres) by taking whichever hit is
closest along the ray — the same pattern used at every level above (closest
Volume hit, closest plane-in-domain hit) applies at the top level too.

## Summary

```
cast ray
  → for each Volume: AABB test → transform to local space
    → QuadTree traversal: find candidate Domain (leaf)
      → Domain: scan height-sorted Planes, skip/stop by height bound
        → Plane: Newton-iterate to exact surface intersection
  → keep closest hit across all Volumes
  → combine with other scene geometry (e.g. lights) by closest distance
```

Everything above resolves a *single* ray to a *single* hit (position,
normal, material). What happens after a hit — how many rays get cast, and
what lighting model is applied — is a separate concern, covered in
[Render Tiers](render-tiers.md).