//! Pure SU(3) Wilson action on a periodic four-dimensional Euclidean lattice.
use crate::{rng::Rng, su3::Matrix, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Lattice {
    pub length: usize,
    pub links: Vec<Matrix>,
}
impl Lattice {
    pub fn new(length: usize, hot: bool, rng: &mut Rng) -> Result<Self> {
        if !(2..=12).contains(&length) {
            return Err("Lattice side must be 2–12".into());
        }
        Ok(Self {
            length,
            links: (0..4 * length.pow(4))
                .map(|_| {
                    if hot {
                        Matrix::haar(rng)
                    } else {
                        Matrix::identity()
                    }
                })
                .collect(),
        })
    }
    pub fn volume(&self) -> usize {
        self.length.pow(4)
    }
    pub fn coordinates(&self, mut x: usize) -> [usize; 4] {
        let mut c = [0; 4];
        for v in &mut c {
            *v = x % self.length;
            x /= self.length;
        }
        c
    }
    pub fn index(&self, c: [usize; 4]) -> usize {
        c[0] + self.length * (c[1] + self.length * (c[2] + self.length * c[3]))
    }
    pub fn shift(&self, x: usize, mu: usize, step: isize) -> usize {
        let mut c = self.coordinates(x);
        c[mu] = (c[mu] as isize + step).rem_euclid(self.length as isize) as usize;
        self.index(c)
    }
    pub fn link(&self, x: usize, mu: usize) -> Matrix {
        self.links[4 * x + mu]
    }
    pub fn plaquette(&self, x: usize, mu: usize, nu: usize) -> Matrix {
        self.link(x, mu)
            * self.link(self.shift(x, mu, 1), nu)
            * self.link(self.shift(x, nu, 1), mu).dagger()
            * self.link(x, nu).dagger()
    }
    pub fn mean_plaquette(&self) -> f64 {
        (0..self.volume())
            .map(|x| {
                (0..4)
                    .map(|mu| {
                        ((mu + 1)..4)
                            .map(|nu| self.plaquette(x, mu, nu).trace().re / 3.)
                            .sum::<f64>()
                    })
                    .sum::<f64>()
            })
            .sum::<f64>()
            / (6 * self.volume()) as f64
    }
    pub fn action(&self, beta: f64) -> f64 {
        beta * 6. * self.volume() as f64 * (1. - self.mean_plaquette())
    }
    pub fn staple(&self, x: usize, mu: usize) -> Matrix {
        let mut v = Matrix::zero();
        let xp = self.shift(x, mu, 1);
        for nu in 0..4 {
            if nu == mu {
                continue;
            }
            let xm = self.shift(x, nu, -1);
            v = v + self.link(xp, nu)
                * self.link(self.shift(x, nu, 1), mu).dagger()
                * self.link(x, nu).dagger();
            v = v + self.link(self.shift(xp, nu, -1), nu).dagger()
                * self.link(xm, mu).dagger()
                * self.link(xm, nu);
        }
        v
    }
    pub fn sweep(&mut self, beta: f64, step: f64, rng: &mut Rng) -> f64 {
        let mut accepted = 0;
        for x in 0..self.volume() {
            for mu in 0..4 {
                let v = self.staple(x, mu);
                for (i, j) in [(0, 1), (0, 2), (1, 2)] {
                    let old = self.link(x, mu);
                    let new =
                        Matrix::subgroup(i, j, rng.direction(), step * (2. * rng.uniform() - 1.))
                            * old;
                    let delta = -beta / 3. * ((new - old) * v).trace().re;
                    if delta <= 0. || rng.uniform().ln() < -delta {
                        self.links[4 * x + mu] = new;
                        accepted += 1;
                    }
                }
            }
        }
        accepted as f64 / (12 * self.volume()) as f64
    }
    pub fn gauge_transform(&mut self, g: &[Matrix]) -> Result<()> {
        if g.len() != self.volume() {
            return Err("Need one gauge matrix per site".into());
        }
        for x in 0..self.volume() {
            for mu in 0..4 {
                self.links[4 * x + mu] = g[x] * self.link(x, mu) * g[self.shift(x, mu, 1)].dagger();
            }
        }
        Ok(())
    }
    pub fn wilson_loop(&self, r: usize, t: usize) -> f64 {
        let mut sum = 0.;
        for x in 0..self.volume() {
            for mu in 0..4 {
                for nu in mu + 1..4 {
                    let mut y = x;
                    let mut w = Matrix::identity();
                    for _ in 0..r {
                        w = w * self.link(y, mu);
                        y = self.shift(y, mu, 1);
                    }
                    for _ in 0..t {
                        w = w * self.link(y, nu);
                        y = self.shift(y, nu, 1);
                    }
                    for _ in 0..r {
                        y = self.shift(y, mu, -1);
                        w = w * self.link(y, mu).dagger();
                    }
                    for _ in 0..t {
                        y = self.shift(y, nu, -1);
                        w = w * self.link(y, nu).dagger();
                    }
                    sum += w.trace().re / 3.;
                }
            }
        }
        sum / (6 * self.volume()) as f64
    }
    pub fn polyakov(&self) -> [f64; 2] {
        let mut z = num_complex::Complex64::new(0., 0.);
        for x in 0..self.length.pow(3) {
            let mut w = Matrix::identity();
            let mut y = x;
            for _ in 0..self.length {
                w = w * self.link(y, 3);
                y = self.shift(y, 3, 1);
            }
            z += w.trace() / 3.;
        }
        z /= self.length.pow(3) as f64;
        [z.re, z.im]
    }
    pub fn slice(&self, t: usize) -> Vec<f64> {
        (0..self.length.pow(3))
            .map(|x| {
                let x = x + t * self.length.pow(3);
                (0..3)
                    .map(|mu| {
                        ((mu + 1)..3)
                            .map(|nu| self.plaquette(x, mu, nu).trace().re / 3.)
                            .sum::<f64>()
                    })
                    .sum::<f64>()
                    / 3.
            })
            .collect()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LatticeRequest {
    pub length: usize,
    pub beta: f64,
    pub sweeps: usize,
    pub warmup: usize,
    pub chains: usize,
    pub step: f64,
    pub hot: bool,
    pub seed: u64,
}
impl Default for LatticeRequest {
    fn default() -> Self {
        Self {
            length: 4,
            beta: 5.7,
            sweeps: 30,
            warmup: 20,
            chains: 2,
            step: 0.35,
            hot: false,
            seed: 7,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct Measurement {
    pub sweep: usize,
    pub plaquette: f64,
    pub acceptance: f64,
    pub wilson_2x2: f64,
    pub polyakov: [f64; 2],
}
#[derive(Serialize, Deserialize)]
pub struct Chain {
    pub measurements: Vec<Measurement>,
    pub frames: Vec<Vec<f64>>,
    pub max_unitarity_error: f64,
}
#[derive(Serialize, Deserialize)]
pub struct LatticeResult {
    pub request: LatticeRequest,
    pub chains: Vec<Chain>,
    pub elapsed_ms: f64,
    pub note: String,
}
pub fn simulate(req: &LatticeRequest) -> Result<LatticeResult> {
    if !(2..=10).contains(&req.length)
        || req.sweeps == 0
        || req.sweeps > 300
        || req.warmup > 300
        || req.chains == 0
        || req.chains > 16
        || !req.beta.is_finite()
        || req.beta < 0.
        || req.beta > 30.
        || !req.step.is_finite()
        || req.step <= 0.
        || req.step > 3.
    {
        return Err(
            "Use L=2–10, 1–300 sweeps, 0–300 warmup, 1–16 chains, beta=0–30, step=(0,3]".into(),
        );
    }
    if req.length.pow(4) * (req.sweeps + req.warmup) * req.chains > 8_000_000 {
        return Err(
            "Lattice request exceeds 8 million site-sweeps; reduce volume, sweeps, or chains"
                .into(),
        );
    }
    let start = std::time::Instant::now();
    let chains: Result<Vec<Chain>> = (0..req.chains)
        .into_par_iter()
        .map(|i| {
            let mut rng = Rng::event(req.seed, i);
            let mut l = Lattice::new(req.length, req.hot, &mut rng)?;
            for _ in 0..req.warmup {
                l.sweep(req.beta, req.step, &mut rng);
            }
            let mut measurements = vec![];
            let mut frames = vec![l.slice(0)];
            for s in 0..req.sweeps {
                let a = l.sweep(req.beta, req.step, &mut rng);
                measurements.push(Measurement {
                    sweep: s + 1,
                    plaquette: l.mean_plaquette(),
                    acceptance: a,
                    wilson_2x2: l.wilson_loop(2, 2),
                    polyakov: l.polyakov(),
                });
                if s % ((req.sweeps / 30).max(1)) == 0 {
                    frames.push(l.slice(0));
                }
            }
            Ok(Chain {
                measurements,
                frames,
                max_unitarity_error: l
                    .links
                    .iter()
                    .map(|u| u.unitary_error())
                    .fold(0_f64, f64::max),
            })
        })
        .collect();
    Ok(LatticeResult {request:req.clone(),chains:chains?,elapsed_ms:start.elapsed().as_secs_f64()*1000.,note:"Pure gauge SU(3), periodic 4D Euclidean Wilson action. Independent chains run in parallel. Sweeps are not physical time. Warmup and autocorrelation must be studied before precision inference; no dynamical quarks or continuum extrapolation.".into()})
}
