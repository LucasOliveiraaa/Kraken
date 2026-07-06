# QuadTree Acceleration Structure

[Domains](planes-and-domains.md) are arranged in a **QuadTree** over the XY
plane of a [Volume](volumes.md), so a ray doesn't need to test every domain
in the scene — only the ones its path could plausibly pass through.

## Structure

The QuadTree is stored as a flat buffer of `QuadNode`s (the same struct used
to represent a leaf Domain — see [Planes & Domains](planes-and-domains.md)):

```rust
struct GlQuadNode {
    children: u32,     // 0 = leaf, otherwise index of the first of 4 children
    first_idx: u32,    // leaf: first plane index. internal: first child index
    plane_count: u32,  // leaf only
    position: [f32; 2],
    size: f32,
}
```

An internal node's four children occupy four *consecutive* slots in the node
buffer (`first_idx .. first_idx + 4`), each covering one quadrant of the
parent's square footprint. A leaf node (`children == 0`) is a Domain, with
its own stack of height-sorted Planes.

## Traversal

Traversal is iterative and stack-based (not recursive — GPU shaders don't
support recursion), using a fixed-capacity stack of `(node, tEntry, tExit)`
triples:

1. Pop a node and its ray-parameter interval off the stack.
2. Re-test the ray against that node's actual XY bounding square
   (`intersectQuadXY`) — the interval carried on the stack is inherited from
   the parent and needs tightening against this specific child's box.
3. If the ray misses the node's box entirely, discard it — this is the
   actual "fast rejection of empty space": an entire subtree, and everything
   under it, is skipped without visiting a single Plane.
4. If it's a **leaf**, test the ray against the Domain's Planes (see
   [Rendering Pipeline](../rendering/pipeline.md)) and return immediately on
   a hit.
5. If it's **internal**, compute entry/exit intervals for all 4 children,
   sort them by entry distance (nearest-first), and push them onto the stack
   **far-to-near** — so the nearest child is popped and processed first.

Processing nearest-first matters for correctness of early exit: as soon as
any leaf produces a hit, traversal can stop, but only because nearer nodes
are guaranteed to have been checked before farther ones. Without the sort,
a farther domain could be found "first" and incorrectly returned even though
a nearer domain also contains geometry along the same ray.

## Why a QuadTree specifically

The scene's geometry is fundamentally 2D-parameterized (each height function
maps `(x, y) → z`), so partitioning space by XY footprint — rather than by
full 3D bounding volumes, as a BVH would — matches the data directly. A
QuadTree also composes naturally with [Volumes](volumes.md): the whole
Volume's ray-to-local-space transform happens once, and then traversal is a
straightforward 2D ray-box problem from that point on, with height handled
separately per-Domain via the min/max bounds described in
[Planes & Domains](planes-and-domains.md).