//! Optional CUDA implementation of the nine massless tree-level scattering channels.
//! Kernels are written in Rust and compiled by CUDA Oxide to bundled sm_80 PTX.
//! Requires an NVIDIA Ampere-or-newer GPU and a CUDA 12.8+ compatible driver.
//! No CUDA library is loaded by the default CPU build. FP64 is used throughout.
use crate::{
    math::FourVector,
    rng::Rng,
    scattering::{self, Event, Process, ScatterRequest},
};
use cudarc::{
    driver::{
        sys::CUevent_flags, CudaContext, CudaFunction, CudaSlice, CudaStream, LaunchConfig,
        PushKernelArg,
    },
    nvrtc::Ptx,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path, sync::Arc};

pub type CudaResult<T> = std::result::Result<T, Box<dyn Error>>;
const FIELDS: usize = 8;
const REDUCE_BLOCKS: u32 = 4096;
const PARTIAL_FIELDS: usize = 122;
pub const BYTES_PER_EVENT: usize = FIELDS * size_of::<f64>();

pub struct GpuScattering {
    pub context: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    generate: CudaFunction,
    reduce: CudaFunction,
    gather: CudaFunction,
}
pub struct ResidentEvents {
    data: CudaSlice<f64>,
    partials: CudaSlice<f64>,
    owner: Arc<CudaStream>,
    count: usize,
}
impl ResidentEvents {
    pub fn count(&self) -> usize {
        self.count
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchStats {
    pub events: u64,
    pub gpu_seconds: f64,
    pub histogram_sum: Vec<f64>,
    pub histogram_square_sum: Vec<f64>,
    pub transverse_momentum_sum: [f64; 2],
}

impl GpuScattering {
    /// Load a CUDA Oxide PTX module built from gpu/oxide.
    ///
    /// # Safety
    /// The PTX must implement the exact generate/reduce/gather ABI and bounded
    /// accesses documented in gpu/oxide/src/lib.rs. Arbitrary PTX can violate
    /// host allocation guarantees. Prefer the bundled, validated constructor.
    pub unsafe fn from_ptx(ordinal: usize, path: &Path) -> CudaResult<Self> {
        let context = CudaContext::new(ordinal)?;
        let stream = context.default_stream();
        let module = context.load_module(cudarc::nvrtc::Ptx::from_file(path))?;
        Ok(Self {
            context,
            stream,
            generate: module.load_function("generate")?,
            reduce: module.load_function("reduce")?,
            gather: module.load_function("gather")?,
        })
    }
    pub fn new(ordinal: usize) -> CudaResult<Self> {
        let context = CudaContext::new(ordinal)?;
        let stream = context.default_stream();
        let ptx = Ptx::from_src(include_str!("scattering.ptx"));
        let module = context.load_module(ptx)?;
        Ok(Self {
            context,
            stream,
            generate: module.load_function("generate")?,
            reduce: module.load_function("reduce")?,
            gather: module.load_function("gather")?,
        })
    }
    pub fn allocate(&self, count: usize) -> CudaResult<ResidentEvents> {
        if count < 2 || count > u32::MAX as usize {
            return Err("GPU batch must contain 2..=u32::MAX events".into());
        }
        let elements = count
            .checked_mul(FIELDS)
            .ok_or("event allocation overflow")?;
        Ok(ResidentEvents {
            data: self.stream.alloc_zeros::<f64>(elements)?,
            partials: self
                .stream
                .alloc_zeros::<f64>(REDUCE_BLOCKS as usize * PARTIAL_FIELDS)?,
            owner: Arc::clone(&self.stream),
            count,
        })
    }
    pub fn run(
        &self,
        buffer: &mut ResidentEvents,
        request: &ScatterRequest,
        offset: u64,
    ) -> CudaResult<BatchStats> {
        if !Arc::ptr_eq(&self.stream, &buffer.owner) {
            return Err("Event buffer belongs to another CUDA backend".into());
        }
        let mut validation = request.clone();
        validation.events = 2;
        validation.display = 0;
        validation.validate()?;
        let n = buffer.count as u64;
        offset.checked_add(n).ok_or("event index overflow")?;
        let process = Process::all()
            .iter()
            .position(|p| *p == request.process)
            .ok_or("unknown process")? as i32;
        let config = LaunchConfig {
            grid_dim: (buffer.count.div_ceil(256).min(65535) as u32, 1, 1),
            block_dim: (256, 1, 1),
            shared_mem_bytes: 0,
        };
        let start = self
            .stream
            .record_event(Some(CUevent_flags::CU_EVENT_DEFAULT))?;
        // SAFETY: the compiled signatures match these arguments. Each grid-stride
        // index is bounded by n; data has 8*n doubles and partials 122*4096 doubles.
        // The default stream orders generation, reduction, timing and host copies.
        unsafe {
            self.stream
                .launch_builder(&self.generate)
                .arg(&mut buffer.data)
                .arg(&n)
                .arg(&offset)
                .arg(&request.seed)
                .arg(&process)
                .arg(&request.energy)
                .arg(&request.alpha)
                .arg(&request.cos_cut)
                .launch(config)?;
            self.stream
                .launch_builder(&self.reduce)
                .arg(&buffer.data)
                .arg(&mut buffer.partials)
                .arg(&n)
                .arg(&request.cos_cut)
                .launch(LaunchConfig {
                    grid_dim: (REDUCE_BLOCKS, 1, 1),
                    block_dim: (256, 1, 1),
                    shared_mem_bytes: 0,
                })?;
        }
        let end = self
            .stream
            .record_event(Some(CUevent_flags::CU_EVENT_DEFAULT))?;
        let gpu_seconds = start.elapsed_ms(&end)? as f64 / 1000.;
        let partials = self.stream.clone_dtoh(&buffer.partials)?;
        let mut sums = [0.; PARTIAL_FIELDS];
        for block in partials.as_chunks::<PARTIAL_FIELDS>().0 {
            for (out, x) in sums.iter_mut().zip(block) {
                *out += x;
            }
        }
        if !sums.iter().all(|x| x.is_finite()) {
            return Err("Nonfinite CUDA reduction".into());
        }
        Ok(BatchStats {
            events: n,
            gpu_seconds,
            histogram_sum: sums[..60].to_vec(),
            histogram_square_sum: sums[60..120].to_vec(),
            transverse_momentum_sum: [sums[120], sums[121]],
        })
    }
    pub fn sample(
        &self,
        buffer: &ResidentEvents,
        request: &ScatterRequest,
        offset: u64,
        count: usize,
    ) -> CudaResult<Vec<Event>> {
        if !Arc::ptr_eq(&self.stream, &buffer.owner) {
            return Err("Event buffer belongs to another CUDA backend".into());
        }
        if count == 0 || count > buffer.count || count > 65536 {
            return Err("Sample size must be 1..=min(batch,65536)".into());
        }
        let mut sample = self.stream.alloc_zeros::<f64>(count * FIELDS)?;
        let n = buffer.count as u64;
        offset.checked_add(n).ok_or("event index overflow")?;
        let k = count as u64;
        // SAFETY: gather writes exactly 8*count values with bounded input indices.
        unsafe {
            self.stream
                .launch_builder(&self.gather)
                .arg(&buffer.data)
                .arg(&mut sample)
                .arg(&n)
                .arg(&k)
                .launch(LaunchConfig::for_num_elems(count as u32))?;
        }
        let host = self.stream.clone_dtoh(&sample)?;
        Ok(host
            .as_chunks::<FIELDS>()
            .0
            .iter()
            .enumerate()
            .map(|(j, v)| {
                let e = v[2];
                let outgoing = FourVector::new(e, v[3], v[4], v[5]);
                Event {
                    id: (offset + j as u64 * n / k) as usize,
                    cos_theta: v[0],
                    weight_pb: v[6],
                    matrix_element_squared: v[7],
                    species: request.process.species(),
                    momenta: [
                        FourVector::new(e, 0., 0., e),
                        FourVector::new(e, 0., 0., -e),
                        outgoing,
                        FourVector::new(e, -v[3], -v[4], -v[5]),
                    ],
                }
            })
            .collect())
    }
    /// Independent native-backend comparison: all nine channels, awkward batch
    /// boundary, per-event RNG/kinematics, histogram, and a large global offset.
    pub fn validate(&self) -> CudaResult<f64> {
        let mut buffer = self.allocate(65537)?;
        let mut max_error = 0.0_f64;
        for (k, process) in Process::all().into_iter().enumerate() {
            let request = ScatterRequest {
                process,
                events: 65537,
                display: 0,
                cos_cut: 0.97,
                seed: 100 + k as u64,
                ..Default::default()
            };
            let stats = self.run(&mut buffer, &request, 0)?;
            let cpu = scattering::simulate(&request)?;
            for ((q, mean), error) in stats
                .histogram_square_sum
                .iter()
                .zip(&cpu.histogram)
                .zip(&cpu.histogram_error)
            {
                let reference = (error * error * 65536. + mean * mean) * 65537.;
                if (q - reference).abs() / reference.abs().max(1e-20) > 2e-10 {
                    return Err(format!("CUDA second-moment mismatch: {process:?}").into());
                }
            }
            for (a, b) in stats
                .histogram_sum
                .iter()
                .map(|x| x / 65537.)
                .zip(&cpu.histogram)
            {
                let err = (a - b).abs() / b.abs().max(1e-20);
                max_error = max_error.max(err);
                if err > 2e-10 {
                    return Err(format!("CUDA histogram mismatch: {process:?}: {a} vs {b}").into());
                }
            }
            for offset in [0, 1_u64 << 35] {
                if offset != 0 {
                    self.run(&mut buffer, &request, offset)?;
                }
                for event in self.sample(&buffer, &request, offset, 513)? {
                    let mut rng = Rng::event(request.seed, event.id);
                    let c = request.cos_cut * (2. * rng.uniform() - 1.);
                    let m = scattering::com_momenta(
                        request.energy,
                        c,
                        std::f64::consts::TAU * rng.uniform(),
                    );
                    let expected = 2.
                        * request.cos_cut
                        * scattering::differential_pb(process, request.energy, c, request.alpha);
                    let err = (event.weight_pb - expected).abs() / expected.abs();
                    max_error = max_error.max(err);
                    if err > 2e-11
                        || (event.momenta[2] - m[2]).max_abs() > 1e-10
                        || (event.cos_theta - c).abs() > 1e-14
                    {
                        return Err(
                            format!("CUDA event mismatch: {process:?} index {}", event.id).into(),
                        );
                    }
                }
            }
        }
        Ok(max_error)
    }
}
