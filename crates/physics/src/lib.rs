//! Natural units (GeV, c = hbar = 1), metric (+---). See docs/PHYSICS.md.
pub mod amplitude;
pub mod decay;
pub mod diagram;
pub mod lattice;
pub mod math;
pub mod rng;
pub mod scattering;
pub mod su3;

pub type Result<T> = std::result::Result<T, String>;
