//! Color-ordered, all-outgoing, tree-level gluon MHV amplitudes.
use crate::{math::FourVector, Result};
use num_complex::Complex64 as C;
use serde::{Deserialize, Serialize};

pub fn spinor(p: FourVector) -> Result<[C; 2]> {
    if !p.finite() || p.e == 0. || p.mass2().abs() > 1e-8 * p.e * p.e {
        return Err("Spinors require nonzero real massless four-momenta".into());
    }
    let sign = if p.e < 0. { -1. } else { 1. };
    let q = p * sign;
    let phase = if sign < 0. {
        C::new(0., 1.)
    } else {
        C::new(1., 0.)
    };
    let a = if q.e + q.pz > 1e-12 * q.e {
        let s = (q.e + q.pz).sqrt();
        [C::new(s, 0.), C::new(q.px, q.py) / s]
    } else {
        let s = (q.e - q.pz).sqrt();
        [C::new(q.px, -q.py) / s, C::new(s, 0.)]
    };
    Ok([a[0] * phase, a[1] * phase])
}
pub fn bracket(a: [C; 2], b: [C; 2]) -> C {
    a[0] * b[1] - a[1] * b[0]
}

/// Exact four-gluon color/helicity sum using the Del Duca–Dixon–Maltoni basis.
/// Averages over the 2² * 8² incoming states, sums all outgoing states.
pub fn four_gluon_averaged(momenta: &[FourVector; 4], coupling: f64) -> Result<f64> {
    let mut amplitudes = Vec::new();
    for i in 0..4 {
        for j in i + 1..4 {
            let reordered = [momenta[0], momenta[2], momenta[1], momenta[3]];
            let map = [0, 2, 1, 3];
            amplitudes.push((
                mhv(momenta, [i, j], coupling)?,
                mhv(&reordered, [map[i], map[j]], coupling)?,
            ));
        }
    }
    let mut sum = 0.;
    // T^a = lambda^a/2. The factor 2 matches the Parke–Taylor normalization.
    for a in 0..8 {
        for b in 0..8 {
            for c in 0..8 {
                for d in 0..8 {
                    let c1: f64 = (0..8)
                        .map(|e| {
                            crate::su3::structure_f(a, b, e) * crate::su3::structure_f(e, c, d)
                        })
                        .sum();
                    let c2: f64 = (0..8)
                        .map(|e| {
                            crate::su3::structure_f(a, c, e) * crate::su3::structure_f(e, b, d)
                        })
                        .sum();
                    for (x, y) in &amplitudes {
                        sum += (2. * (c1 * x + c2 * y)).norm_sqr();
                    }
                }
            }
        }
    }
    Ok(sum / 256.)
}
/// A_n = i g^(n-2) <ij>^4 / (<12><23>...<n1>), stripped color ordering.
pub fn mhv(momenta: &[FourVector], negative: [usize; 2], coupling: f64) -> Result<C> {
    let n = momenta.len();
    if !(4..=12).contains(&n)
        || negative[0] >= n
        || negative[1] >= n
        || negative[0] == negative[1]
        || !coupling.is_finite()
        || coupling < 0.
    {
        return Err("MHV needs 4–12 legs, two distinct negative helicities, and finite nonnegative coupling".into());
    }
    let scale = momenta.iter().map(|p| p.e.abs()).sum::<f64>();
    if momenta
        .iter()
        .copied()
        .fold(FourVector::default(), |a, b| a + b)
        .max_abs()
        > 1e-8 * scale
    {
        return Err("All-outgoing momenta must sum to zero".into());
    }
    let s: Vec<_> = momenta.iter().map(|p| spinor(*p)).collect::<Result<_>>()?;
    let mut d = C::new(1., 0.);
    for k in 0..n {
        d *= bracket(s[k], s[(k + 1) % n]);
    }
    if d.norm() == 0. {
        return Err("Collinear pole in color-ordered amplitude".into());
    }
    let a = C::new(0., coupling.powi(n as i32 - 2))
        * bracket(s[negative[0]], s[negative[1]]).powu(4)
        / d;
    if !a.re.is_finite() || !a.im.is_finite() {
        return Err("Amplitude overflow near singular kinematics".into());
    }
    Ok(a)
}
#[derive(Serialize, Deserialize)]
pub struct AmplitudeRequest {
    pub momenta: Vec<FourVector>,
    pub negative: [usize; 2],
    pub coupling: f64,
}
#[derive(Serialize, Deserialize)]
pub struct AmplitudeResult {
    pub real: f64,
    pub imaginary: f64,
    pub norm_squared: f64,
    pub four_gluon_spin_color_averaged: Option<f64>,
    pub convention: String,
}
pub fn evaluate(r: &AmplitudeRequest) -> Result<AmplitudeResult> {
    let a = mhv(&r.momenta, r.negative, r.coupling)?;
    let averaged = if r.momenta.len() == 4 {
        Some(four_gluon_averaged(
            r.momenta.as_slice().try_into().unwrap(),
            r.coupling,
        )?)
    } else {
        None
    };
    Ok(AmplitudeResult {real:a.re,imaginary:a.im,norm_squared:a.norm_sqr(),four_gluon_spin_color_averaged:averaged,convention:"Parke–Taylor tree MHV, fixed cyclic ordering, all momenta outgoing, i g^(n−2). Color stripped; not a spin/color averaged matrix element.".into()})
}
