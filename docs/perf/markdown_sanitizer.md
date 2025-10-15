# Markdown Sanitizer Performance Report

- **Date**: 2025-10-15
- **Environment**: macOS (apple silicon) · Rust `cargo build --release` artifacts · Flutter 3.35.2 (fvm)
- **Command**:

```bash
KELIVO_SANITIZER_IMAGE_DIR="$(pwd)/build/bench_images" \
  fvm flutter test test/benchmark/markdown_sanitizer_benchmark.dart --plain-name benchmark
```

## Benchmark Summary (5 iterations)

| Mode | `replaceInlineBase64Images` avg (ms) | `inlineLocalImagesToBase64` avg (ms) |
|------|-------------------------------------:|--------------------------------------:|
| Mock (Dart) | 8.34 | 0.30 |
| Real (Rust) | 2.41 | 0.26 |

- Replace phase improvement: **≈71% faster** (dominant workload and primary goal).
- Inline phase improvement: **≈13% faster**; largely I/O bound, so gains are smaller but still positive.

## Notes

- Fixture: `test/testdata/benchmark/benchmark_markdown.md` with three 1.5 KB inline Base64 images repeated four times and three local image references.
- Local image sources are generated per run inside the benchmark harness (3 KB each).
- Output directory controlled via `KELIVO_SANITIZER_IMAGE_DIR` to isolate artifacts under `build/bench_images`.
- Both mock and real modes reset their output folders between iterations to avoid caching effects.
- Rust code compiled with `cargo build --release` before running the benchmark.
