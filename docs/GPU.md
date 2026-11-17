# CUDA scattering

The optional `cuda` feature computes the same nine massless tree-level QED/QCD channels as the Rust CPU backend. GPU kernels are written in Rust and compiled with NVIDIA CUDA Oxide; cudarc handles the host-side CUDA driver calls. The CPU and GPU share the matrix-element formulas and SplitMix64 implementation. Arithmetic is FP64 with FMA contraction disabled. The default Python wheel and default Rust build do not require CUDA.

Running the bundled kernels requires an NVIDIA Ampere-or-newer GPU, a CUDA 12.8+ compatible driver and Rust. No CUDA C++ compiler or NVRTC compilation is used at runtime.

To rebuild the kernels, install CUDA Oxide `cuda-oxide/v0.4.0` (commit `9d0c9bfbc954a9a29565ab81b78da70f054d8e04`) and its pinned Rust 1.99.1 toolchain following [NVIDIA's instructions](https://github.com/NVIDIA/cuda-rust/tree/cuda-oxide/v0.4.0/cuda-oxide). Run `bash tools/build-oxide.sh`, then run the GPU validation below. `gpu/oxide` has its own lockfile and toolchain; the normal workspace remains on stable Rust. The bundled PTX targets `sm_80` and is JIT-compiled by the NVIDIA driver for the installed GPU.

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

## Recorded campaign and animation

See the [20-minute campaign](../benchmarks/GPU.md) for measurements and backend provenance. Its compressed receipt contains the actual samples and histograms used in the README animation. Render it with Node.js and FFmpeg:

```sh
npm ci --prefix tools
node tools/render-campaign.mjs benchmarks/gpu-20min/result.json.gz output/frames 180
ffmpeg -framerate 15 -i output/frames/frame-%04d.png \
  -filter_complex "[0:v]split[a][b];[a]palettegen=stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=3" \
  -loop 0 output/gpu-scattering.gif
```

To check and archive a new run, use `node tools/summarize-campaign.mjs output/cuda-run/result.json output/cuda-archive`.
