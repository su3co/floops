# Validation record

Initial implementation, 2026-11-16. These are checks of the implemented models, not a claim of complete Standard Model coverage or precision experimental agreement.

| Requested capability | Evidence |
| --- | --- |
| Rust engine processing many particles | Rayon event batches; five-million-event release benchmark; exact serial/parallel histogram and momentum-sum agreement |
| Particle decays | 44-entry catalogue, 20 supported parent species; all parents tested for charge, mass-shell and four-momentum conservation; custom model accepted through HTTP |
| SU(3) simulation | Generator/Casimir/Jacobi tests; random gauge transformations preserve lattice observables; local update action matches a full lattice recomputation |
| Scattering amplitudes | Nine QED/QCD reference channels; analytic QED integral; complex MHV spinor identities and 4–12-leg scaling/cyclic tests; explicit four-gluon color/helicity sum matches the independent cross-section formula |
| Feynman diagrams | Complete 3–6-gluon tree counts (1, 4, 25, 220); 220 topologies rendered in the browser; SVG downloaded |
| 3D simulations and animation | Computed scattering, decay and lattice data displayed with Three.js; Play/Pause advances and stops the timeline; event and independent-chain selection exercised |
| Python wrapper | Native PyO3 extension built and installed; smoke script covers every API, concurrent calls and errors; physics code remains Rust |
| Reusable repository | Cargo/npm lockfiles, MIT license, CLI, API, examples, source attribution and Linux/macOS/Windows CI |

## Local checks

- 18 Rust physics tests passed in release mode.
- Formatting and Clippy checks passed for the native application.
- TypeScript checking and the Vite production build passed.
- Python 3.15 native extension installation and `examples/python_smoke.py` passed.
- HTTP checks exercised all nine scattering channels, custom decay input, complex amplitudes, six-gluon diagrams and HTTP 400 validation errors.
- Desktop Chromium and a 390×844 Chromium viewport exercised the actual application. No horizontal page overflow was observed. Play/Pause, scrubbers, event selection, independent lattice chains, complex amplitudes, all 220 six-gluon topologies and invalid helicity feedback were checked.
- JSON, CSV and SVG were downloaded through the browser. Reduced-motion emulation confirmed that animation does not autoplay.
- Expected HTTP 400 responses from the deliberate invalid-input test were distinguished from browser failures. A missing favicon found during the first pass was fixed.

Browser evidence is kept locally in the ignored `output/playwright/` directory; the README includes a clean application preview. Mobile verification is viewport-based Chromium, not a physical phone or Safari certification. Numerical limitations and reference conventions are documented in [PHYSICS.md](PHYSICS.md). Cross-platform results are reported by the repository's [Validate workflow](https://github.com/su3co/floops/actions/workflows/ci.yml).
