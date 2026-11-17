#![cfg(feature = "cuda")]
#[test]
#[ignore = "requires an NVIDIA GPU and NVRTC; run explicitly on a CUDA host"]
fn gpu_events_and_reductions_match_rust() {
    let gpu = floops::cuda::GpuScattering::new(0).unwrap();
    assert!(gpu.validate().unwrap() < 2e-10);
}
