use crate::rng::Rng;
use num_complex::Complex64 as C;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Matrix(pub [[C; 3]; 3]);
impl Matrix {
    pub fn zero() -> Self {
        Self([[C::new(0., 0.); 3]; 3])
    }
    pub fn identity() -> Self {
        let mut a = Self::zero();
        for (i, row) in a.0.iter_mut().enumerate() {
            row[i] = C::new(1., 0.);
        }
        a
    }
    pub fn dagger(self) -> Self {
        let mut a = Self::zero();
        for i in 0..3 {
            for j in 0..3 {
                a.0[i][j] = self.0[j][i].conj();
            }
        }
        a
    }
    pub fn trace(self) -> C {
        (0..3).map(|i| self.0[i][i]).sum()
    }
    pub fn det(self) -> C {
        let a = self.0;
        a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
    }
    pub fn norm(self) -> f64 {
        self.0
            .iter()
            .flatten()
            .map(|x| x.norm_sqr())
            .sum::<f64>()
            .sqrt()
    }
    pub fn unitary_error(self) -> f64 {
        (self * self.dagger() - Self::identity()).norm()
    }
    pub fn subgroup(i: usize, j: usize, axis: [f64; 3], theta: f64) -> Self {
        let mut a = Self::identity();
        let c = theta.cos();
        let s = theta.sin();
        a.0[i][i] = C::new(c, s * axis[2]);
        a.0[j][j] = C::new(c, -s * axis[2]);
        a.0[i][j] = C::new(s * axis[1], s * axis[0]);
        a.0[j][i] = C::new(-s * axis[1], s * axis[0]);
        a
    }
    /// Haar U(3) from complex Gaussian Gram-Schmidt; determinant projected to SU(3).
    pub fn haar(r: &mut Rng) -> Self {
        let mut out = Self::zero();
        for j in 0..3 {
            let mut v = [C::new(0., 0.); 3];
            for x in &mut v {
                *x = C::new(r.normal(), r.normal());
            }
            for k in 0..j {
                let dot: C = (0..3).map(|i| out.0[i][k].conj() * v[i]).sum();
                for (i, x) in v.iter_mut().enumerate() {
                    *x -= out.0[i][k] * dot;
                }
            }
            let n = v.iter().map(|z| z.norm_sqr()).sum::<f64>().sqrt();
            for (i, x) in v.iter().enumerate() {
                out.0[i][j] = x / n;
            }
        }
        let phase = out.det().conj();
        for row in &mut out.0 {
            row[2] *= phase;
        }
        out
    }
}
impl std::ops::Add for Matrix {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        let mut o = self;
        for i in 0..3 {
            for j in 0..3 {
                o.0[i][j] += b.0[i][j];
            }
        }
        o
    }
}
impl std::ops::Sub for Matrix {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        self + b * (-1.)
    }
}
impl std::ops::Mul<f64> for Matrix {
    type Output = Self;
    fn mul(self, b: f64) -> Self {
        let mut o = self;
        for row in &mut o.0 {
            for x in row {
                *x *= b;
            }
        }
        o
    }
}
impl std::ops::Mul for Matrix {
    type Output = Self;
    fn mul(self, b: Self) -> Self {
        let mut o = Self::zero();
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    o.0[i][j] += self.0[i][k] * b.0[k][j];
                }
            }
        }
        o
    }
}
pub fn generators() -> [Matrix; 8] {
    let mut t = [Matrix::zero(); 8];
    for (a, i, j) in [(0, 0, 1), (3, 0, 2), (5, 1, 2)] {
        t[a].0[i][j] = C::new(0.5, 0.);
        t[a].0[j][i] = C::new(0.5, 0.);
        t[a + 1].0[i][j] = C::new(0., -0.5);
        t[a + 1].0[j][i] = C::new(0., 0.5);
    }
    t[2].0[0][0] = C::new(0.5, 0.);
    t[2].0[1][1] = C::new(-0.5, 0.);
    for (i, row) in t[7].0.iter_mut().enumerate() {
        row[i] = C::new(if i == 2 { -1. } else { 0.5 } / 3_f64.sqrt(), 0.);
    }
    t
}
pub fn structure_f(a: usize, b: usize, c: usize) -> f64 {
    let t = generators();
    2. * ((t[a] * t[b] - t[b] * t[a]) * t[c]).trace().im
}
pub fn structure_d(a: usize, b: usize, c: usize) -> f64 {
    let t = generators();
    2. * ((t[a] * t[b] + t[b] * t[a]) * t[c]).trace().re
}
