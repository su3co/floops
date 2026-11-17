//! CUDA Oxide kernels with the same raw launch ABI as the validated reference.
//! Host-side allocation/lifetime checks live in floops::cuda. The unsafe boundary
//! is restricted to bounded raw buffer indexing and block-local shared storage.
use cuda_device::{
    SharedArray,
    atomic::{AtomicOrdering, BlockAtomicF64},
    kernel, thread,
};

#[path = "../../../crates/physics/src/tree_level.rs"]
mod tree_level;

#[path = "../../../crates/physics/src/random_bits.rs"]
mod random_bits;
use random_bits::next_u64 as next;

#[inline]
fn uniform(state: &mut u64) -> f64 {
    ((next(state) >> 12) as f64 + 0.5) / 4503599627370496.
}

/// # Safety
/// `events` holds 8*n doubles, n>0, offset+n fits u64, physical parameters are
/// validated by the Rust host. A 1-D launch gives disjoint grid-stride writes.
#[kernel]
pub unsafe fn generate(
    events: *mut f64,
    n: u64,
    offset: u64,
    seed: u64,
    process: i32,
    energy: f64,
    alpha: f64,
    cut: f64,
) {
    let stride = thread::blockDim_x() as u64 * thread::gridDim_x() as u64;
    let mut i = thread::index_1d().get() as u64;
    while i < n {
        let mut state = seed ^ ((offset + i).wrapping_mul(0xd2b74407b1ce6e93));
        state = next(&mut state);
        let c = cut * (2. * uniform(&mut state) - 1.);
        let phi = core::f64::consts::TAU * uniform(&mut state);
        let e = energy / 2.;
        let r = (1. - c * c).sqrt();
        let s = energy * energy;
        let m2 = tree_level::matrix_element_squared(
            process as u32,
            s,
            -s * (1. - c) / 2.,
            -s * (1. + c) / 2.,
            alpha,
        );
        let symmetry = if process == 2 || process == 6 || process == 8 {
            0.5
        } else {
            1.
        };
        let weight = 2. * cut * symmetry * m2 / (32. * core::f64::consts::PI * s) * 389379372.1;
        // Each field plane is n elements long; i is checked above.
        unsafe {
            *events.add(i as usize) = c;
            *events.add((n + i) as usize) = phi;
            *events.add((2 * n + i) as usize) = e;
            *events.add((3 * n + i) as usize) = e * r * phi.cos();
            *events.add((4 * n + i) as usize) = e * r * phi.sin();
            *events.add((5 * n + i) as usize) = e * c;
            *events.add((6 * n + i) as usize) = weight;
            *events.add((7 * n + i) as usize) = m2;
        }
        i += stride;
    }
}

/// # Safety
/// events holds 8*n initialized doubles; partials holds 122*gridDim.x doubles.
/// Every thread must reach both barriers. Only the owning block accesses its
/// shared array and its disjoint output row.
#[kernel]
pub unsafe fn reduce(events: *const f64, partials: *mut f64, n: u64, cut: f64) {
    static mut HIST: SharedArray<BlockAtomicF64, 122> = SharedArray::UNINIT;
    let h = unsafe { SharedArray::as_raw_mut_ptr(&raw mut HIST) };
    let tid = thread::threadIdx_x() as usize;
    let width = thread::blockDim_x() as usize;
    let mut j = tid;
    while j < 122 {
        unsafe {
            h.add(j).write(BlockAtomicF64::new(0.));
        }
        j += width;
    }
    thread::sync_threads();
    let mut i = thread::index_1d().get() as u64;
    let stride = width as u64 * thread::gridDim_x() as u64;
    while i < n {
        unsafe {
            let b = (((*events.add(i as usize) / cut + 1.) * 30.) as usize).min(59);
            let w = *events.add((6 * n + i) as usize);
            (&*h.add(b)).fetch_add(w, AtomicOrdering::Relaxed);
            (&*h.add(60 + b)).fetch_add(w * w, AtomicOrdering::Relaxed);
            (&*h.add(120)).fetch_add(*events.add((3 * n + i) as usize), AtomicOrdering::Relaxed);
            (&*h.add(121)).fetch_add(*events.add((4 * n + i) as usize), AtomicOrdering::Relaxed);
        }
        i += stride;
    }
    thread::sync_threads();
    j = tid;
    while j < 122 {
        unsafe {
            *partials.add(thread::blockIdx_x() as usize * 122 + j) =
                (&*h.add(j)).load(AtomicOrdering::Relaxed);
        }
        j += width;
    }
}

/// # Safety
/// events holds 8*n initialized doubles, sample holds 8*count doubles, and
/// 0<count<=n. Each thread owns exactly one sample record.
#[kernel]
pub unsafe fn gather(events: *const f64, sample: *mut f64, n: u64, count: u64) {
    let j = thread::index_1d().get() as u64;
    if j < count {
        let i = j * n / count;
        let mut field = 0;
        while field < 8 {
            unsafe {
                *sample.add((j * 8 + field) as usize) = *events.add((field * n + i) as usize);
            }
            field += 1;
        }
    }
}
