#!/usr/bin/env bash
set -euo pipefail
floops_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$floops_root/gpu/oxide"
# Pin the compiler and dependencies in Cargo.lock/rust-toolchain.toml. Keep FMA
# disabled so CPU/native-GPU comparisons use the same arithmetic convention.
if [[ -n "${FLOOPS_CARGO_OXIDE:-}" ]]; then
  "$FLOOPS_CARGO_OXIDE" build --arch sm_80 --no-fmad -- --lib --release --locked
else
  cargo oxide build --arch sm_80 --no-fmad -- --lib --release --locked
fi
test -s floops_oxide_kernels.ptx
for kernel in generate reduce gather; do
  grep -q ".entry $kernel(" floops_oxide_kernels.ptx
done
cp floops_oxide_kernels.ptx "$floops_root/crates/physics/src/scattering.ptx"
echo "Updated bundled CUDA Oxide kernels. Run the GPU validation before publishing."
