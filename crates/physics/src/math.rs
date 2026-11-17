use crate::{rng::Rng, Result};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FourVector {
    pub e: f64,
    pub px: f64,
    pub py: f64,
    pub pz: f64,
}
impl FourVector {
    pub fn new(e: f64, px: f64, py: f64, pz: f64) -> Self {
        Self { e, px, py, pz }
    }
    pub fn rest(m: f64) -> Self {
        Self::new(m, 0., 0., 0.)
    }
    pub fn dot(self, b: Self) -> f64 {
        self.e * b.e - self.px * b.px - self.py * b.py - self.pz * b.pz
    }
    pub fn mass2(self) -> f64 {
        self.dot(self)
    }
    pub fn spatial(self) -> [f64; 3] {
        [self.px, self.py, self.pz]
    }
    pub fn finite(self) -> bool {
        [self.e, self.px, self.py, self.pz]
            .iter()
            .all(|x| x.is_finite())
    }
    /// Active boost from a rest frame into a frame moving at velocity beta.
    pub fn boost(self, beta: [f64; 3]) -> Result<Self> {
        let b2: f64 = beta.iter().map(|x| x * x).sum();
        if !b2.is_finite() || b2 >= 1. || !self.finite() {
            return Err("Boost requires finite |beta| < 1".into());
        }
        if b2 == 0. {
            return Ok(self);
        }
        let gamma = 1. / (1. - b2).sqrt();
        let bp = self.px * beta[0] + self.py * beta[1] + self.pz * beta[2];
        let f = gamma * gamma / (gamma + 1.) * bp + gamma * self.e;
        Ok(Self::new(
            gamma * (self.e + bp),
            self.px + f * beta[0],
            self.py + f * beta[1],
            self.pz + f * beta[2],
        ))
    }
    pub fn beta(self) -> [f64; 3] {
        [self.px / self.e, self.py / self.e, self.pz / self.e]
    }
    pub fn max_abs(self) -> f64 {
        [self.e, self.px, self.py, self.pz]
            .iter()
            .fold(0_f64, |a, b| a.max(b.abs()))
    }
}
impl Add for FourVector {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.e + b.e, self.px + b.px, self.py + b.py, self.pz + b.pz)
    }
}
impl Sub for FourVector {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        self + (-b)
    }
}
impl Neg for FourVector {
    type Output = Self;
    fn neg(self) -> Self {
        self * -1.
    }
}
impl Mul<f64> for FourVector {
    type Output = Self;
    fn mul(self, b: f64) -> Self {
        Self::new(self.e * b, self.px * b, self.py * b, self.pz * b)
    }
}
pub fn kallen(x: f64, y: f64, z: f64) -> f64 {
    x * x + y * y + z * z - 2. * (x * y + x * z + y * z)
}

pub fn two_body(parent: FourVector, masses: [f64; 2], rng: &mut Rng) -> Result<[FourVector; 2]> {
    let m2 = parent.mass2();
    if !parent.finite()
        || parent.e <= 0.
        || m2 <= 0.
        || masses.iter().any(|m| !m.is_finite() || *m < 0.)
    {
        return Err("Invalid parent or daughter mass".into());
    }
    let m = m2.sqrt();
    if m < masses[0] + masses[1] {
        return Err("Decay is below threshold".into());
    }
    let q = (((m - masses[0] - masses[1])
        * (m + masses[0] + masses[1])
        * (m - masses[0] + masses[1])
        * (m + masses[0] - masses[1]))
        .max(0.))
    .sqrt()
        / (2. * m);
    let d = rng.direction();
    let a = FourVector::new(
        (q * q + masses[0] * masses[0]).sqrt(),
        q * d[0],
        q * d[1],
        q * d[2],
    )
    .boost(parent.beta())?;
    Ok([a, parent - a])
}

/// Uniform Lorentz-invariant three-body phase space, using a sampled two-body invariant.
pub fn three_body(parent: FourVector, m: [f64; 3], rng: &mut Rng) -> Result<[FourVector; 3]> {
    if !parent.finite()
        || parent.e <= 0.
        || parent.mass2() <= 0.
        || m.iter().any(|x| !x.is_finite() || *x < 0.)
    {
        return Err("Invalid three-body masses".into());
    }
    let big = parent.mass2().sqrt();
    if big <= m.iter().sum() {
        return Err("Three-body decay requires available phase space".into());
    }
    let lo = (m[0] + m[1]).powi(2);
    let hi = (big - m[2]).powi(2);
    // The first factor decreases with s; the second increases. Their endpoint
    // product is a rigorous, tighter envelope near thresholds such as beta decay.
    let envelope = kallen(big * big, lo, m[2] * m[2]).max(0.).sqrt()
        * kallen(hi, m[0] * m[0], m[1] * m[1]).max(0.).sqrt()
        / hi;
    for _ in 0..1_000_000 {
        let s = lo + (hi - lo) * rng.uniform();
        let w = kallen(big * big, s, m[2] * m[2]).max(0.).sqrt()
            * kallen(s, m[0] * m[0], m[1] * m[1]).max(0.).sqrt()
            / s;
        if rng.uniform() * envelope > w {
            continue;
        }
        let [cluster, c] = two_body(parent, [s.sqrt(), m[2]], rng)?;
        let [a, b] = two_body(cluster, [m[0], m[1]], rng)?;
        return Ok([a, b, c]);
    }
    Err("Phase-space rejection sampler did not converge".into())
}
