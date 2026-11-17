# CUDA scattering

The optional `cuda` feature computes the same nine massless tree-level QED/QCD channels as the Rust CPU backend. It uses Rust with cudarc and a small CUDA kernel, with FP64 arithmetic and fast-math disabled. The default Python wheel and default Rust build do not require CUDA.

Requires an NVIDIA GPU, an NVIDIA driver, Rust and CUDA NVRTC 12.8 or newer. Add the installed CUDA `lib64` directory to `LD_LIBRARY_PATH` where necessary.

```sh
cargo run --release -p floops --features cuda --example cuda_scatter -- --validate-only
cargo run --release -p floops --features cuda --example cuda_scatter -- \
  --gpu 0 --seconds 1200 --reserve-gib 2 --energy 2000 --out output/cuda-run
```

The runner first compares all nine channels against native Rust: event kinematics, the random-number stream, histogram sums and second moments, including non-aligned batch sizes and global event indices above 2³⁵. It then allocates resident event data using free VRAM minus a reserve. Pass `--events N` for a smaller batch.

Each event occupies 64 bytes: scattering angle, azimuth, outgoing energy and three-momentum, cross-section weight and squared matrix element. The other outgoing momentum follows from momentum conservation. Every record is generated on the GPU and contributes to the histogram; the allocation contains no padding added to inflate VRAM usage. Batches reuse the buffer with disjoint random-number indices.

`--seconds` counts CUDA-event time for generation and reduction, excluding initialization, compilation, copies and cooling. The runner retains the context while cooling above its temperature threshold and resumes computation below its recovery threshold. It does not change clocks, power limits or other services.

The output contains `result.json` (parameters, hardware model, measured timings, allocation size, numerical reference, snapshots and sampled four-momenta) and `progress.jsonl` (per-batch progress). Existing result receipts are never overwritten. Reference histograms use numerical integration of the native Rust differential cross section.

These are independent, weighted parton-scattering events at fixed coupling. The animation represents sampled momenta; it is not a shower, hadronization, detector simulation or physical-time evolution. Loop amplitudes are not included.
