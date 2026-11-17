# Interfaces

The executable serves the UI and JSON API on loopback port 8631 by default. It has no authentication; place authentication and resource controls in front of it before exposing it to an untrusted network. Request bodies are capped at 512 KB and at most two calculations run concurrently. Invalid physics requests return HTTP 400; structural JSON errors use Axum's 4xx responses; a busy engine returns 503.

| Endpoint | Input |
| --- | --- |
| GET `/api/health` | Version and worker count |
| GET `/api/catalogue` | Particle model with masses, lifetimes and channels |
| GET `/api/processes` | Supported process IDs and external species |
| POST `/api/scatter` | [Example](../examples/scatter.json) |
| POST `/api/decay` | [Example](../examples/decay.json) |
| POST `/api/lattice` | [Example](../examples/lattice.json) |
| POST `/api/amplitude` | [Example](../examples/amplitude.json) |
| POST `/api/diagrams` | `{"process":"gluon_gluon"}` for SVGs or `{"legs":6}` for tree JSON |

Scattering, decay and lattice requests have defaults; supply only the fields to override. `display` caps returned event records while every simulated event contributes to the statistics. Up to five million scattering events or 200,000 decay chains are accepted per call. Streaming statistics bound memory use independently of batch size. `RAYON_NUM_THREADS` controls CPU workers. Results are deterministic at fixed seed and parameters on the same build/platform, independent of worker count; cross-platform floating-point bit identity is not promised.

Rust entry points are `scattering::simulate`, `decay::simulate`, `amplitude::evaluate`, `lattice::simulate`, and `diagram::{process_diagrams,gluon_trees}` in `unnamed-physics`. Lower-level matrix and kinematics modules are also available. Numerical units and conventions are in [PHYSICS.md](PHYSICS.md).

The PyO3 module exposes `scatter`, `decay`, `lattice`, `amplitude` and `catalogue`. Input/output uses JSON strings for a small, dependency-free Python surface. Invalid inputs raise `ValueError`. CPU batch functions release the interpreter while native Rust and Rayon execute. The example [python_smoke.py](../examples/python_smoke.py) exercises every binding and concurrent calls. Python contains no physics implementation.

## Custom decay models

Start from the catalogue response and replace/add particles, then pass it as the `model` field in a decay request. Each particle has:

```json
{
  "pdg": 9000001,
  "name": "X",
  "mass": 2.0,
  "charge_thirds": 0,
  "lifetime_s": 1e-12,
  "channels": [
    {"daughters": [11, -11], "branching": 0.7, "model": "phase_space"},
    {"daughters": [13, -13], "branching": 0.3, "model": "phase_space"}
  ]
}
```

The model object requires `particles` and `source`; all referenced daughters must be present. There are no automatic extra antiparticles for custom models. Validation checks unique IDs, finite masses/lifetimes, thresholds, charge conservation and total branching probability ≤1. Channels support two or three daughters. `phase_space` is constant matrix element; `v_a` requires three daughters ordered charged lepton, antineutrino, neutrino, with the last two massless. A model is a user-supplied physical assumption, not automatically inferred from particle names or quantum numbers.

## Development

Run the Rust server, then `cd web && npm run dev`. Vite proxies `/api` to the Rust server. The production server serves `web/dist`; override with `serve --web PATH`. Dependencies are locked in Cargo.lock and package-lock.json. No CDN or external service is required by the application.

Native builds and Python wheels are checked in CI on Linux, macOS and Windows; the web build has its own job. The browser uses WebGL through Three.js and provides tables/exports even if WebGL is unavailable. Play is opt-in, camera controls support mouse/touch and Left/Right/Home keys, and controls have visible keyboard focus.
