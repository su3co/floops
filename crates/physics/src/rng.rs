//! Independent SplitMix64 streams make parallel event batches reproducible.
#[derive(Clone)]
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn event(seed: u64, index: usize) -> Self {
        let mut r = Self(seed ^ (index as u64).wrapping_mul(0xd2b74407b1ce6e93));
        Self(r.next_u64())
    }
    pub fn next_u64(&mut self) -> u64 {
        crate::random_bits::next_u64(&mut self.0)
    }
    pub fn uniform(&mut self) -> f64 {
        // Strictly inside (0, 1), including after floating-point rounding.
        ((self.next_u64() >> 12) as f64 + 0.5) * (1.0 / 4503599627370496.0)
    }
    pub fn direction(&mut self) -> [f64; 3] {
        let z = 2.0 * self.uniform() - 1.0;
        let phi = std::f64::consts::TAU * self.uniform();
        let r = (1.0 - z * z).sqrt();
        [r * phi.cos(), r * phi.sin(), z]
    }
    pub fn normal(&mut self) -> f64 {
        (-2.0 * self.uniform().ln()).sqrt() * (std::f64::consts::TAU * self.uniform()).cos()
    }
}
