# Implementation and design

Temporary project name: unnamed. `feynman` is occupied on PyPI. No PyPI publication is planned until a name is chosen.

The requested outcome is a Rust particle simulator with parallel batches, a 3D animated interface, SU(3) simulations, scattering amplitudes, Feynman diagrams, a small Python wrapper, and a repository in SU3 Labs.

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
| Repository | Documented source, CI, reproducible commands, pushed to verified su3-labs organization |

## Visual plan

Color tokens: paper `#e7edf3`, ink `#17293d`, instrument blue `#254d75`, electron cyan `#72dbed`, quark coral `#ffad91`, field violet `#ada0f2`.

Typography: locally bundled IBM Plex Sans for controls, IBM Plex Serif for the experiment title and physical equations. Numeric tables use tabular figures, not a separate monospace aesthetic.

Layout: an experiment bench. The 3D scattering scene occupies the main area; a compact settings column sits at the left, with results and diagrams directly below the scene. Left-aligned text, no marketing hero or repeated metric cards.

```
unnamed                          SU(3) laboratory     sources
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

