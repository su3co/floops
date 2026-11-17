# Sources and dependencies

Original application code is MIT licensed. Dependency licenses remain their own; consult the pinned packages and their distributed notices.

- Rust, Rayon, Axum, Tokio, serde, num-complex, clap and PyO3 provide the implementation infrastructure. No PYTHIA, MadGraph or existing `feynman` Python package source is copied into this project.
- Three.js supplies WebGL rendering under its MIT license.
- NVIDIA CUDA Oxide compiles the optional Rust GPU kernels; cudarc provides CUDA driver bindings. Their versions are pinned in the Cargo manifests and lockfiles.
- resvg-js (MPL-2.0) rasterizes the data-derived benchmark animation; FFmpeg encodes the GIF. Neither is needed to use the physics library.
- The animation's math label is typeset with LaTeX's Computer Modern fonts. Its TeX source and rendered SVG outlines are included under `tools/`.
- IBM Plex Sans and IBM Plex Serif are bundled locally via Fontsource under the SIL Open Font License 1.1. Their license files are included in the installed font packages.
- Equations and reference particle values are attributed in [PHYSICS.md](PHYSICS.md). No PDG PDF or third-party figure is redistributed.
