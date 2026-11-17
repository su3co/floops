# Physics and validation

## Conventions

Four-vectors are `(e, px, py, pz)`, in GeV, with metric `(+---)`. Natural units apply to amplitudes and phase space. Decay vertices instead explicitly use seconds and millimeters: `[time_s, x_mm, y_mm, z_mm]`. Cross sections use `1 GeV⁻² = 389379372.1 pb`.

The Rust engine implements three distinct calculations. Scattering is perturbative, decays use specified channels and matrix elements, and the lattice samples a Euclidean gauge-field ensemble. The program does **not** infer arbitrary particle decay tables from nonperturbative QCD.

## Scattering

Supported massless, leading-order processes:

- `electron_muon`: e⁻e⁺ → μ⁻μ⁺, photon exchange only;
- `quark_different`: qq′ → qq′;
- `quark_same`: qq → qq;
- `annihilation`: qq̄ → q′q̄′;
- `quark_antiquark`: qq̄ → qq̄;
- `quark_gluon`: qg → qg;
- `quark_to_gluon`: qq̄ → gg;
- `gluon_to_quark`: gg → qq̄, one final flavor;
- `gluon_gluon`: gg → gg.

The analytic squared matrix elements include interference and average over initial colors/spins. `alpha` means the electromagnetic coupling for `electron_muon`, and αs otherwise. Couplings are fixed input parameters. The catalogue's massive quark entries do not change the massless scattering approximation.

For center-of-mass energy E, `s = E²`, `t = −s(1−cosθ)/2`, `u = −s(1+cosθ)/2`. With final-state symmetry factor S,

```
dσ/dcosθ = S × |M|² / (32πs).
```

S is 1/2 for identical qq or gg final states, otherwise 1. The angular domain is the full labeled range `−cut < cosθ < cut`; dividing by 2 prevents double counting identical physical final states. This convention must be accounted for when comparing to labeled-particle differential formulae or half-domain integrals.

Events sample cosθ and azimuth uniformly. Each event carries the integration weight `2 × cut × dσ/dcosθ`; the estimator is the mean weight, with an unbiased sample standard error. Histogram bins contain integrated cross section, **not** differential density. Their sum is the total accepted cross section. Bin errors are correlated; do not combine them as independent errors to recover the total. The event display is a weighted sample and its point density is not a physical angular distribution.

Angular cuts bound the collinear poles. No PDFs, beam remnants, running coupling, electroweak Z exchange, finite scattering masses, showers, hadronization, multiparton interactions, NLO corrections or detector transport are implemented. This is not a substitute for PYTHIA, MadGraph or Geant4.

Reference: [PDG 2025 cross-section formulae, section 51.2](https://pdg.lbl.gov/2025/reviews/rpp2025-rev-cross-section-formulae.pdf).

## Complex amplitudes and diagrams

The amplitude module computes the Parke–Taylor formula for 4–12 external massless gluons, with two negative-helicity legs i,j:

```
Aₙ = i g^(n−2) ⟨ij⟩⁴ / (⟨12⟩⟨23⟩ … ⟨n1⟩).
```

The momentum convention is all outgoing; incoming four-vectors have negative energy. The negative-helicity indices refer to this convention and are zero-based in JSON/Rust/Python, one-based in the interface. Inputs must conserve four-momentum. Negative-energy spinors use the i-times-positive-energy continuation. Overall phases and little-group phases are convention-dependent. `|A|²` is a fixed-color-order result, with mass dimension `8−2n`, not a physical spin/color average.

For four gluons the engine additionally constructs the full amplitude in a two-element DDM color basis using the SU(3) structure constants. It sums the six nonzero helicity configurations and all external colors and divides by the 256 incoming spin/color states. A numerical test compares this independent calculation to the analytic gg → gg result at several angles.

The general amplitude evaluator supports MHV only: no arbitrary helicities, loop amplitudes, massive legs, generic Lagrangian parser, or arbitrary beyond-Standard-Model amplitudes. The nine 2→2 processes use analytic squared matrix elements; complex amplitudes are currently exposed for gluons.

The diagram engine recursively partitions labeled external gluons into 3- and 4-gluon vertices. It enumerates all trees through six external legs: 1, 4, 25, and 220 topologies at 3, 4, 5, and 6 legs. A two-child wrapper in the JSON roots a tree on the final external leg; it is not a physical two-valent interaction. Process SVG diagrams use fermion arrows, wavy photons and curly gluons. Topology gallery lines denote gluons without encoding a particular cyclic ordering. Diagrams are not separately observable probabilities.

Reference: [Lance Dixon, Simplifying Multi-Jet QCD Computation](https://arxiv.org/abs/1101.2414).

## Decays and particle data

The built-in catalogue contains 44 particles/antiparticles and 20 parent species with implemented modes. It includes leptons, neutrinos, light mesons, J/ψ, nucleons and quark/gluon reference entries. Custom catalogues are accepted through the same Rust, HTTP and Python API.

Masses, lifetimes and branching fractions are rounded reference inputs. Selected modes are implemented; this is **not** a complete PDG decay database. An event landing in the missing branching probability is explicitly marked `unmodeled` and retains its parent's four-momentum. It is not silently resampled into an implemented mode. `depth_limit` distinguishes truncation of a cascade. Stable and unsupported parton entries are not passed to a hadronization model.

Two-body decays are isotropic in the parent rest frame. Three-body decays use factorized Lorentz-invariant phase space, sampling the two-body invariant mass with a rejection envelope. The two factors have opposite monotonicity, allowing a rigorous envelope that remains efficient near thresholds.

Muon and leptonic tau modes use the unpolarized V−A factor `(P·p_antineutrino)(p_lepton·p_neutrino)`. The charged-lepton mass is retained; neutrinos are massless. The muon energy spectrum approaches the normalized Michel distribution `2x²(3−2x)`, with mean x = 0.7 when the electron mass is neglected. The engine tests this limit statistically. Other modes, including neutron beta decay, use uniform phase space; their detailed angular/Dalitz spectra are not precision decay models. Resonances have fixed nominal masses, not sampled line shapes.

The muon channel and charged-pion leptonic branching fractions approximate inclusive modes without explicitly generating radiative photons. No radiative corrections, oscillations, polarized production, sequential spin correlations, or amplitude interference between decay channels are included. Lifetimes are sampled exponentially in proper time and transformed into lab time and position. The visual cascade compresses lengths and time; exported vertices preserve the calculated physical values.

Sources: [PDG leptons](https://pdg.lbl.gov/2025/tables/rpp2025-sum-leptons.pdf), [mesons](https://pdg.lbl.gov/2025/tables/rpp2025-sum-mesons.pdf), [baryons and quarks](https://pdg.lbl.gov/2025/tables/contents_tables.html), and [kinematics](https://pdg.lbl.gov/2025/reviews/rpp2025-rev-kinematics.pdf). Catalogue values are illustrative rounded inputs, without experimental covariance or uncertainty propagation.

## SU(3) gauge fields

Generators are `Tᵃ = λᵃ/2`, with `Tr(TᵃTᵇ) = δᵃᵇ/2`, fundamental Casimir 4/3 and adjoint Casimir 3. Matrix multiplication, adjoints, determinants, Haar sampling and f/d structure constants are native Rust.

The lattice is periodic, hypercubic and four-dimensional. Its Wilson action is

```
S = β Σ(x, μ<ν) [1 − Re Tr Uμν(x)/3],   β = 6/g².
```

Updates use symmetric embedded SU(2) proposals in all three subgroups and a Metropolis acceptance test. A sweep updates all links and subgroups sequentially; independent chains run in parallel. Hot starts use complex-Gaussian orthogonalization and determinant projection to Haar SU(3). Cold starts use identity links.

The engine returns mean plaquettes, 2×2 Wilson loops, complex Polyakov loops, acceptance and unitarity residuals. Saved frames show the t=0 spatial slice, colored by gauge-invariant spatial plaquettes. Connectivity lines are not literal gluon trajectories. Sweep number is Monte Carlo evolution, **not physical time**.

Defaults are interactive demonstrations, not equilibrated precision ensembles. Users must assess thermalization, autocorrelation, volume effects and lattice-spacing dependence. The engine does not estimate an autocorrelation-corrected error or claim a continuum result. There are no dynamical fermions, quark propagators, hadron spectroscopy or real-time Yang–Mills dynamics. Lattice units are not assigned a physical spacing.

Reference: [David Tong, Gauge Theory, chapter 4](https://davidtong.org/teaching/gauge-theory/).

## Numerical checks

`cargo test --release -p unnamed-physics` tests:

- Lorentz-invariant masses and inverse boosts;
- on-shell two/three-body kinematics and four-momentum conservation;
- massless three-body phase-space moments;
- SU(3) generator normalization, Casimirs, Jacobi identities, Haar unitarity and determinant;
- Wilson-action local/global agreement, cold-field observables and gauge invariance;
- analytic QED angular integration, QCD reference values and crossing color factors;
- deterministic serial/parallel event reduction and Monte Carlo uncertainty;
- Parke–Taylor magnitudes, cyclic symmetry, spinor identities and the independent full color sum;
- gluon tree counts;
- all supported decay parents, charge conservation, proper lifetimes, Michel spectrum and missing modes;
- invalid-input rejection.

These tests establish the implemented mathematical checks. They are not an experimental validation, independent code review, or a precision-QCD certification. CPU batches use Rayon with deterministic per-event SplitMix64 streams and fixed reduction chunks. GPU acceleration is used by Three.js for rendering; there is no CUDA physics backend.
