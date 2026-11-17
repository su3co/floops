# Implementation and design

Project name: **floops**. The first release targets PyPI with a Rust backend and thin Python bindings. Crates.io publication is deferred.

The focus is scattering amplitudes, decay calculations and Feynman diagrams in Rust, with Python access and an interactive Three.js viewer. SU(3) algebra and pure-gauge lattice experiments are also included. Loop amplitudes are a future capability, not part of the initial release.

Implemented. Requirement-by-requirement evidence is recorded in [VALIDATION.md](VALIDATION.md); physical scope is documented in [PHYSICS.md](PHYSICS.md).

| Requirement | Implementation and completion evidence |
| --- | --- |
| Native parallel simulation | Rayon, deterministic per-event streams; measured release benchmark |
| Particle zoo and decays | Source-linked particle catalogue, extensible decay model, conservation and lifetime tests |
| Scattering | Tree-level QED/QCD differential cross sections, weighted event generation, complex MHV amplitudes, reference checks |
| SU(3) | Generator algebra and four-dimensional pure-gauge Wilson lattice, gauge invariance and local action tests |
| Feynman diagrams | Process-linked exchange/contact diagrams, recursive gluon tree enumeration, SVG export |
| Interactive animation | Three.js event trajectories and lattice slices; real browser checks at desktop/mobile widths |
| Python | Thin PyO3 bindings calling the same Rust calculations; installed wheel smoke test |
| Repository | Documented source, CI, reproducible commands, pushed to verified su3co organization |

## Visual plan

Color tokens: paper `#e7edf3`, ink `#17293d`, instrument blue `#254d75`, electron cyan `#72dbed`, quark coral `#ffad91`, field violet `#ada0f2`.

Typography: locally bundled IBM Plex Sans for controls, IBM Plex Serif for the experiment title and physical equations. Numeric tables use tabular figures, not a separate monospace aesthetic.

Layout: an experiment bench. The 3D scattering scene occupies the main area; a compact settings column sits at the left, with results and diagrams directly below the scene. Left-aligned text, no marketing hero or repeated metric cards.

```
floops                          SU(3) laboratory     sources
Events / Amplitudes / Lattice
+------------------+------------------------------------------+
| experiment       |                                          |
| beam energy      |              3D event chamber            |
| event count      |                                          |
| run simulation   |              timeline / orbit            |
+------------------+------------------------------------------+
| numerical result | angle distribution / Feynman diagrams     |
+------------------+------------------------------------------+
```

Review: a generic dark dashboard would obscure the educational content. Use a light instrument panel around a deep-blue chamber, reserve particle colors for physical species, and show formulas and uncertainty alongside the visualization. Motion is user-controlled and reduced-motion preferences are respected. The lattice is labeled as a Euclidean configuration, never real-time particle motion.

