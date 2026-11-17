#![cfg(feature = "cuda")]
#[test]
#[ignore = "requires an NVIDIA GPU; run explicitly on a CUDA host"]
fn gpu_events_and_reductions_match_rust() {
    let gpu = floops::cuda::GpuScattering::new(0).unwrap();
    assert!(gpu.validate().unwrap() < 2e-10);
}

#[test]
#[ignore = "requires an NVIDIA GPU; run explicitly on a CUDA host"]
fn invalid_buffers_and_indices_are_rejected_before_launch() {
    use floops::{cuda::GpuScattering, scattering::ScatterRequest};
    let gpu = GpuScattering::new(0).unwrap();
    let other = GpuScattering::new(0).unwrap();
    assert!(gpu.allocate(1).is_err());
    let mut buffer = gpu.allocate(257).unwrap();
    assert_eq!(buffer.count(), 257);
    let request = ScatterRequest::default();
    assert!(other.run(&mut buffer, &request, 0).is_err());
    assert!(other.sample(&buffer, &request, 0, 1).is_err());
    assert!(gpu.run(&mut buffer, &request, u64::MAX).is_err());
    assert!(gpu.sample(&buffer, &request, u64::MAX, 1).is_err());
    assert!(gpu.sample(&buffer, &request, 0, 258).is_err());
    assert!(gpu.sample(&buffer, &request, 0, 0).is_err());
    gpu.run(&mut buffer, &request, 0).unwrap();
    assert_eq!(gpu.sample(&buffer, &request, 0, 257).unwrap().len(), 257);
}
