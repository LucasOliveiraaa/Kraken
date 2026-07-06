# Render Tiers

The [Rendering Pipeline](pipeline.md) describes how a *single ray* resolves
to a hit. What happens once a hit is found — how many more rays get cast,
and how the surface actually gets shaded — is a separate, tunable concern.
Kraken is structured (or, where noted, planned) around several tiers
trading accuracy for speed.

> **Status:** Tier 0 and 2 are implemented today. Tiers 1 and 3 are planned; this
> document describes the intended design so implementation work has a
> shared reference point.

## Tier 0 — Full path tracing (default)

The primary ray is cast per the pipeline above; on each hit, the path
continues by stochastically choosing a diffuse or specular bounce direction
(weighted by material reflectivity), accumulating light and throughput, up
to a configurable bounce limit. Direct lighting at each bounce is sampled
via shadow rays toward each light.

This is the most physically accurate tier: multi-bounce global
illumination, reflections, and soft indirect lighting fall out naturally
from the same ray-casting machinery, with no special-casing per effect.

**Cost:** noisy per-frame — a single frame is a partial sample, resolved
over time via temporal accumulation against the previous frame's result.
Camera movement or scene changes invalidate accumulated history and restart
convergence. Cost scales with bounce count, shadow-ray count per bounce, and
how expensive the primary-ray search ([pipeline](pipeline.md):
QuadTree + Newton) is per hit.

**Best for:** offline/reference rendering, screenshots, a "max quality"
toggle when interactivity isn't the priority.

## Tier 1 — Hybrid rasterization + implicit-surface ray casting (planned)

Ordinary triangle geometry is rasterized through the normal hardware
pipeline into a G-buffer (position, normal, albedo per pixel). Pixels
covered by implicit-surface objects are resolved via a bounding-proxy
rasterization pass (so hardware depth-testing composites correctly against
triangle geometry), with the pipeline's QuadTree + Newton search run only
for those specific pixels to find the true surface point and normal,
overwriting the proxy's G-buffer entry.

Shading, once the G-buffer is complete, uses the same local-lighting model
as Tier 2 below — this tier only changes *how the first hit is found* for
implicit surfaces, not how it's lit.

**Cost:** implicit-surface search cost is paid only for pixels actually
covered by implicit geometry, rather than the whole frame; ordinary
triangle geometry benefits from full hardware rasterization performance.

**Best for:** scenes that mix conventional mesh content with a smaller
number of implicit-surface features.

## Tier 2 — Single ray cast + local lighting

The primary ray is cast exactly once (no bounce loop). The hit is shaded
directly with an analytic local lighting model (Blinn-Phong or similar):
diffuse + specular terms from each light, attenuated by inverse-square
distance, with a shadow ray per light to determine occlusion. A flat
ambient term approximates the indirect light that Tier 0's bounces would
otherwise provide. Optionally, a single bounded reflection ray (not
stochastic, not recursive beyond one or two bounces) can be cast for
strongly reflective materials, to recover mirror-like reflections without
full path tracing.

Because shading is deterministic (no random sampling), every frame is
already the final image — no temporal accumulation or noise convergence is
needed.

**Cost:** one primary-ray search + a small, fixed number of shadow rays per
hit (one per light), independent of bounce count. Removing temporal
accumulation also removes the re-convergence latency visible in Tier 0 when
the camera moves.

**Best for:** interactive/real-time use where indirect lighting and true
reflections matter less than responsiveness and a noise-free image.

**Trade-offs vs. Tier 0:** no true reflections of arbitrary geometry (only
what an optional bounded reflection ray covers), no indirect
diffuse/color-bleeding (approximated by flat ambient instead), shadows
remain hard (single point-sample toward each light, same as Tier 0's shadow
rays today — not a regression, just not improved either).

## Tier 3 — Tessellated mesh, standard triangle pipeline (planned)

Each implicit-surface patch is sampled on a grid and converted to an
explicit triangle mesh (once, ahead of time or in an infrequent prepass —
not per frame), then rendered through the ordinary vertex/fragment
pipeline with the same local-lighting model as Tier 2 (or simpler, e.g.
baked lighting).

**Cost:** no runtime QuadTree traversal or Newton solving at all — the
per-frame cost is standard rasterization. Accuracy is bounded by
tessellation density: curved regions may show visible faceting unless
tessellated finely (uniformly, or adaptively by curvature — the latter is
the higher-quality but more involved option).

**Best for:** the cheapest tier; suitable for distant/low-priority content,
or as a fallback where implicit evaluation isn't worth its cost.

## Choosing a tier

Tier selection is intended to be a runtime, per-frame configuration choice
(alongside the existing debug view modes and Newton/bounce-count
parameters), not a compile-time fork — so the same scene can be inspected
at different tiers directly, camera position held constant, for comparison
during development.