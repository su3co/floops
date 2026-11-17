//! Natural units (GeV, c = hbar = 1), metric (+---). See docs/PHYSICS.md.
pub mod amplitude;
#[cfg(feature = "cuda")]
pub mod cuda;
pub mod decay;
pub mod diagram;
pub mod lattice;
pub mod math;
mod random_bits;
pub mod rng;
pub mod scattering;
pub mod su3;
mod tree_level;

pub type Result<T> = std::result::Result<T, String>;
