# Kraken

Kraken is an experimental 3D renderer written in Rust that represents geometry as **mathematical functions** instead of triangle meshes.

Surfaces are defined by height functions over a 2D domain and evaluated directly at render time, rather than being pre-tessellated into vertices — enabling compact, potentially infinite geometry with minimal memory usage.

```
f(x, y) → z
```

For the full explanation of the geometry model, spatial structure, and rendering pipeline, see [`docs/`](docs/README.md).

## Status
Early development. The rendering model, structure, and APIs are actively evolving and subject to change without notice.

## Getting Started
To run the editor, clone the repository and run:

```bash
cargo run --release --bin kraken
```

## License
MIT License