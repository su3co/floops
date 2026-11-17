use crate::{math::FourVector, rng::Rng, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
pub const GEV2_TO_PB: f64 = 389_379_372.1;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Process {
    ElectronMuon,
    QuarkDifferent,
    QuarkSame,
    Annihilation,
    QuarkAntiquark,
    QuarkGluon,
    QuarkToGluon,
    GluonToQuark,
    GluonGluon,
}
impl Process {
    pub fn all() -> Vec<Self> {
        vec![
            Self::ElectronMuon,
            Self::QuarkDifferent,
            Self::QuarkSame,
            Self::Annihilation,
            Self::QuarkAntiquark,
            Self::QuarkGluon,
            Self::QuarkToGluon,
            Self::GluonToQuark,
            Self::GluonGluon,
        ]
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::ElectronMuon => "e⁻ e⁺ → μ⁻ μ⁺",
            Self::QuarkDifferent => "q q′ → q q′",
            Self::QuarkSame => "q q → q q",
            Self::Annihilation => "q q̄ → q′ q̄′",
            Self::QuarkAntiquark => "q q̄ → q q̄",
            Self::QuarkGluon => "q g → q g",
            Self::QuarkToGluon => "q q̄ → g g",
            Self::GluonToQuark => "g g → q q̄",
            Self::GluonGluon => "g g → g g",
        }
    }
    pub fn species(self) -> [i32; 4] {
        match self {
            Self::ElectronMuon => [11, -11, 13, -13],
            Self::QuarkDifferent => [2, 1, 2, 1],
            Self::QuarkSame => [2, 2, 2, 2],
            Self::Annihilation => [2, -2, 1, -1],
            Self::QuarkAntiquark => [2, -2, 2, -2],
            Self::QuarkGluon => [2, 21, 2, 21],
            Self::QuarkToGluon => [2, -2, 21, 21],
            Self::GluonToQuark => [21, 21, 2, -2],
            Self::GluonGluon => [21, 21, 21, 21],
        }
    }
    pub fn symmetry(self) -> f64 {
        if matches!(
            self,
            Self::QuarkSame | Self::QuarkToGluon | Self::GluonGluon
        ) {
            0.5
        } else {
            1.
        }
    }
}
/// PDG massless leading-order matrix elements, averaged over initial spins/colors.
pub fn matrix_element_squared(p: Process, s: f64, t: f64, u: f64, alpha: f64) -> f64 {
    crate::tree_level::matrix_element_squared(p as u32, s, t, u, alpha)
}

pub fn differential_pb(p: Process, energy: f64, cos: f64, alpha: f64) -> f64 {
    let s = energy * energy;
    let t = -s * (1. - cos) / 2.;
    let u = -s * (1. + cos) / 2.;
    p.symmetry() * matrix_element_squared(p, s, t, u, alpha) / (32. * PI * s) * GEV2_TO_PB
}
pub fn com_momenta(energy: f64, cos: f64, phi: f64) -> [FourVector; 4] {
    let e = energy / 2.;
    let r = (1. - cos * cos).max(0.).sqrt();
    let o = FourVector::new(e, e * r * phi.cos(), e * r * phi.sin(), e * cos);
    [
        FourVector::new(e, 0., 0., e),
        FourVector::new(e, 0., 0., -e),
        o,
        FourVector::new(e, -o.px, -o.py, -o.pz),
    ]
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ScatterRequest {
    pub process: Process,
    pub energy: f64,
    pub alpha: f64,
    pub cos_cut: f64,
    pub events: usize,
    pub seed: u64,
    pub display: usize,
}
impl Default for ScatterRequest {
    fn default() -> Self {
        Self {
            process: Process::GluonGluon,
            energy: 200.,
            alpha: 0.118,
            cos_cut: 0.90,
            events: 100_000,
            seed: 42,
            display: 256,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: usize,
    pub momenta: [FourVector; 4],
    pub species: [i32; 4],
    pub cos_theta: f64,
    pub weight_pb: f64,
    pub matrix_element_squared: f64,
}
#[derive(Serialize, Deserialize)]
pub struct ScatterResult {
    pub request: ScatterRequest,
    pub cross_section_pb: f64,
    pub standard_error_pb: f64,
    pub histogram: Vec<f64>,
    pub histogram_error: Vec<f64>,
    pub events: Vec<Event>,
    pub elapsed_ms: f64,
    pub events_per_second: f64,
    /// Sum of outgoing transverse momentum components, exercising full kinematics in streaming runs.
    pub transverse_momentum_sum: [f64; 2],
    pub note: String,
}
impl ScatterRequest {
    pub fn validate(&self) -> Result<()> {
        if !self.energy.is_finite()
            || self.energy < 1.
            || self.energy > 1e7
            || !self.alpha.is_finite()
            || self.alpha <= 0.
            || self.alpha > 1.
            || !self.cos_cut.is_finite()
            || self.cos_cut <= 0.
            || self.cos_cut > 0.999
            || self.events < 2
            || self.events > 5_000_000
            || self.display > 2048
        {
            return Err("Use energy=1–10⁷ GeV, alpha=(0,1], angular cut=(0,0.999], 2–5 million events, display ≤2048".into());
        }
        Ok(())
    }
}
fn event(r: &ScatterRequest, i: usize) -> Event {
    let mut rng = Rng::event(r.seed, i);
    let c = r.cos_cut * (2. * rng.uniform() - 1.);
    let m = com_momenta(r.energy, c, 2. * PI * rng.uniform());
    let s = r.energy * r.energy;
    Event {
        id: i,
        momenta: m,
        species: r.process.species(),
        cos_theta: c,
        weight_pb: 2. * r.cos_cut * differential_pb(r.process, r.energy, c, r.alpha),
        matrix_element_squared: matrix_element_squared(
            r.process,
            s,
            -s * (1. - c) / 2.,
            -s * (1. + c) / 2.,
            r.alpha,
        ),
    }
}
pub fn simulate(r: &ScatterRequest) -> Result<ScatterResult> {
    simulate_with_parallel(r, true)
}
pub fn simulate_with_parallel(r: &ScatterRequest, parallel: bool) -> Result<ScatterResult> {
    r.validate()?;
    let start = std::time::Instant::now();
    let chunks = r.events.div_ceil(4096);
    // Fixed chunk boundaries and ordered reduction are identical for every thread count.
    let chunk = |k: usize| {
        let mut sums = [0.; 60];
        let mut squares = [0.; 60];
        let mut samples = vec![];
        let mut transverse = [0.; 2];
        for i in k * 4096..((k + 1) * 4096).min(r.events) {
            let e = event(r, i);
            let b = (((e.cos_theta / r.cos_cut + 1.) * 30.) as usize).min(59);
            sums[b] += e.weight_pb;
            squares[b] += e.weight_pb * e.weight_pb;
            transverse[0] += e.momenta[2].px;
            transverse[1] += e.momenta[2].py;
            if i < r.display {
                samples.push(e);
            }
        }
        (sums, squares, samples, transverse)
    };
    let parts: Vec<_> = if parallel {
        (0..chunks).into_par_iter().map(chunk).collect()
    } else {
        (0..chunks).map(chunk).collect()
    };
    let mut sum = [0.; 60];
    let mut sq = [0.; 60];
    let mut events = vec![];
    let mut transverse = [0.; 2];
    for (a, b, mut ev, pt) in parts {
        transverse[0] += pt[0];
        transverse[1] += pt[1];
        for j in 0..60 {
            sum[j] += a[j];
            sq[j] += b[j];
        }
        events.append(&mut ev);
    }
    let n = r.events as f64;
    let total = sum.iter().sum::<f64>();
    let total_sq = sq.iter().sum::<f64>();
    let sigma = total / n;
    let error = ((total_sq / n - sigma * sigma).max(0.) / (n - 1.)).sqrt();
    let elapsed = start.elapsed().as_secs_f64();
    Ok(ScatterResult {request:r.clone(),cross_section_pb:sigma,standard_error_pb:error,histogram:sum.iter().map(|x|x/n).collect(),histogram_error:sum.iter().zip(sq).map(|(a,b)|((b/n-(a/n).powi(2)).max(0.)/(n-1.)).sqrt()).collect(),events,elapsed_ms:elapsed*1000.,events_per_second:n/elapsed,transverse_momentum_sum:transverse,note:"Massless tree-level partons; fixed coupling; no PDFs, showers, hadronization, or detector response. Events are uniformly sampled with cross-section weights. Histogram bins sum to the cross section within |cos θ| < cut. Identical final states include 1/2!. One quark flavor where applicable.".into()})
}
