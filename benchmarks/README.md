# CPU batch benchmark

Measured on an Apple M4 Pro with 12 Rayon workers, optimized Rust release build, on 2026-11-16. Raw output: [apple-m4-pro.json](apple-m4-pro.json).

Five repetitions of five million weighted massless gg → gg events gave a median **11.114 ms parallel** versus **85.323 ms serial**, a **7.68× speedup**. The corresponding streaming throughput was approximately **450 million events/second**. Treat this as a small analytic-kernel benchmark, not full collision simulation performance.

The measured work includes deterministic per-event random streams, polar/azimuthal sampling, outgoing momenta, analytic matrix elements, weights, histogram/error reductions and transverse-momentum sums. All generated events contribute to statistics; records are streamed rather than all retained in memory. The benchmark requests no display records and excludes HTTP/JSON serialization and rendering. `black_box` consumes the complete result to prevent unused-result elimination. Serial and parallel histogram results must agree exactly.

This has no shower, hadronization, detector, lattice or general multi-leg amplitude work. Those calculations have different costs. Thread scheduling, thermal state, compiler and background load affect timings. No comparison against another event generator or CUDA implementation is implied.

Reproduce:

```sh
cargo run --release -p floops-cli -- benchmark --events 5000000 --repeats 5
```

The CLI records every timing and a cross-section checksum. Set `RAYON_NUM_THREADS` to compare worker counts.
