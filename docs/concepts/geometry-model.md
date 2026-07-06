# Geometry Model

Kraken represents surfaces as **mathematical functions**, not triangle meshes.

## The core idea

A traditional renderer stores geometry as explicit data: a list of vertices and
the triangles connecting them. The shape *is* the data — to render a smoother
curve, you need more vertices.

Kraken instead stores a small set of coefficients describing a **height
function**:

```
f(x, y) → z
```

Given a 2D position `(x, y)`, the function returns a height `z`. The surface
is never stored as points or triangles — it's *evaluated* wherever it's
needed, at whatever precision the ray currently requires.

## Why not a mesh?

Meshes are simple and fast, but a curved surface is only ever an
approximation of the underlying shape — accuracy costs vertices, and vertices
cost memory and bandwidth. A height function is exact everywhere in its
domain, regardless of how closely you look, at the fixed cost of a handful of
coefficients.

## Why not ray marching / SDFs?

Signed Distance Fields are the other common function-based approach: define
`f(x, y, z) → distance to nearest surface` and step along the ray until the
distance is ~zero. This is flexible but has no closed-form stopping point —
you're always taking another step and hoping you're close enough.

Kraken's height functions instead support **direct root-finding**. Because
`f(x, y) - z = 0` is a well-behaved, differentiable equation along a ray,
Newton's method can converge on the exact intersection in only a few
iterations, rather than marching. See [Volumes](volumes.md) and
[Rendering Pipeline](../rendering/pipeline.md) for how this is used in
practice.

## The actual function: biquadratic Bézier patches

Concretely, each height function is a **biquadratic tensor-product Bézier
patch** — a 3×3 grid of control point heights, blended by quadratic Bernstein
basis functions in `x` and `y` independently:

```
height(x, y) = Σᵢ Σⱼ Bᵢ(x) · Bⱼ(y) · controlPoint[i][j]     i, j ∈ {0, 1, 2}
```

where `Bᵢ(t)` are the three quadratic Bernstein basis polynomials
(`(1-t)²`, `2t(1-t)`, `t²`, in expanded form). This basis is:

- **Local and bounded** — the patch only produces sensible values for `x, y
  ∈ [0, 1]`, which is exactly the unit domain a [Domain](planes-and-domains.md)
  maps its local space into.
- **Analytically differentiable** — the gradient (`∂height/∂x`,
  `∂height/∂y`) has a closed form derived from the same basis functions,
  which is exactly what Newton's method needs at each iteration (see
  [Rendering Pipeline](../rendering/pipeline.md)).
- **Cheap to evaluate** — 9 control points, a fixed 3×3 double sum, no
  branching. This runs per Newton iteration, per plane test, per pixel, so
  its cost matters more than almost anything else in the renderer.

Nine floats (the control points) fully describe an entire curved patch —
compare that to the dozens or hundreds of vertices a mesh would need to
approximate the same curvature convincingly.

## What this buys, and what it costs

**Benefits:**
- Extremely compact representation — a handful of floats per patch instead of
  a vertex buffer.
- Surfaces are exact at any zoom level; there's no tessellation-density
  tradeoff to tune for a single patch in isolation.
- Naturally composable with spatial acceleration — see
  [QuadTree Acceleration](quadtree.md).

**Costs:**
- Every visible pixel requires solving for the intersection at render time
  (root-finding), rather than a cheap rasterization test against
  pre-computed triangles.
- The technique is naturally suited to *height-field-like* surfaces (terrain,
  bumps, waves). Arbitrary closed/non-heightfield shapes (a full sphere, a
  torus) don't map onto a single-valued `f(x, y) → z` function and need a
  different representation or a hybrid approach — see
  [Render Tiers](../rendering/render-tiers.md) for how Kraken plans to mix
  this technique with conventional triangle rendering.