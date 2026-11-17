# GPU scattering campaign

One NVIDIA RTX PRO 6000 Blackwell Max-Q Workstation Edition, FP64, on 2026-11-17. The fixed-energy process is massless tree-level **gg → gg**, √s = 2 TeV, αs = 0.118, |cos θ| < 0.9, seed 20261117. These are weighted integration samples, without PDFs, showers, hadronization or detector response.

| Measurement | Recorded result |
| --- | ---: |
| Weighted events | 2,784,130,062,336 |
| GPU generation + reduction time | 1,200.180 s |
| Campaign wall time | 1,310.218 s |
| Batches | 1,806 |
| Resident events per batch | 1,541,600,256 |
| Resident event data | 91.887 GiB |
| Typical total board occupancy | 95,266 / 97,887 MiB (97.3%) |
| Maximum sampled board occupancy | 95,379 MiB |
| Median sampled utilization | 100% |
| Maximum sampled temperature | 82 °C |
| Cross section | 330.941886 ± 0.000320 pb |
| Numerically integrated reference | 330.942023 pb |
| Difference from reference | −0.426 standard errors |
| Largest absolute residual across 60 angular bins | 2.644 standard errors |

CUDA events time the generation and reduction kernels, excluding startup, compilation, host copies and cooling. CUDA-event intervals can include driver scheduling and concurrent work. Tiny validation kernels were run briefly alongside the campaign; this is a recorded workload, not an isolated hardware throughput benchmark. The allocator reserves 2 GiB plus reduction scratch. Board occupancy also includes the driver and other applications.

All 64 bytes per resident event hold generated data: angle, azimuth, energy, three momentum components, weight and squared matrix element. The buffer is overwritten with disjoint event indices on each batch. The event count is the total across batches; the run does not retain trillions of events simultaneously.

## Backend provenance

The 20-minute campaign ran revision [`c4a93f1`](https://github.com/su3co/floops/commit/c4a93f1), which used a Rust host and CUDA C kernels compiled with NVRTC. Its receipt is preserved as recorded.

The current backend uses **Rust kernels compiled with NVIDIA CUDA Oxide**, introduced in [`4aab7cf`](https://github.com/su3co/floops/commit/4aab7cf). CUDA Oxide v0.4.0 and Rust 1.99.1 are pinned; matrix-element formulas and SplitMix64 are shared with the CPU implementation. Bundled PTX removes runtime NVRTC compilation. The current backend was independently checked on the same GPU:

- All nine channels match the native CPU implementation, including event momenta, seeded streams, histograms, second moments and indices above 2³⁵. Maximum relative difference: **1.858 × 10⁻¹⁵**.
- A subsequent full-memory run generated **23,124,003,840 events in 10.062 GPU seconds**, with the same **91.887 GiB** resident allocation. Its cross section was **330.941983 ± 0.003512 pb**, 0.011 standard errors from the reference.
- GPU integration tests reject foreign buffers, invalid sample sizes and overflowing indices before kernel launch. Eighteen CPU physics tests and the source-built Python extension smoke test passed.

The long campaign and the shorter CUDA Oxide check are separate runs. The latter establishes full-memory operation and numerical agreement; it is not a second 20-minute endurance test.

## Data and reproduction

- Long run: [summary](gpu-20min/summary.json), [compressed receipt](gpu-20min/result.json.gz).
- CUDA Oxide check: [summary](oxide-full-memory/summary.json), [compressed receipt](oxide-full-memory/result.json.gz).
- [Build, run and render instructions](../docs/GPU.md).

Each summary includes the original receipt's SHA-256. The bundled Rust-generated PTX has SHA-256 `56272980e6b7b1744e9120057a70fdf1fff85c8f5320b7f3f9ec739ed4eb7c0c`.

The 5-second GIF contains 250 frames at 50 fps. It displays 32 actual event pairs from the final recorded snapshot, using every eighth event from its 256-event sample. Both outgoing directions come from the recorded four-momenta; track brightness reflects sample weight and depth. A fixed sample, continuous camera motion and fading trails make the loop seamless. The three circles are a momentum-space reference, not a detector. Rotation and track motion are visual aids, not physical time evolution. The only label uses LaTeX Computer Modern vector outlines. Numerical results remain in the measurements above and the archived receipts.

Sampled four-momenta conserve total momentum exactly in the receipt, with relative mass-shell residuals below 4.7 × 10⁻¹⁶. These numerical checks validate the implementation within its stated tree-level model; they do not add higher-order or nonperturbative physics.
