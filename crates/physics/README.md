# floops

Scattering amplitudes, particle decays and Feynman diagrams, computed in Rust and exposed through a small Python interface. Parallel batches use Rayon; Python contains no physics implementation.

```sh
pip install floops
```

```python
import json
import floops

result = json.loads(floops.scatter('{"events":100000,"seed":42}'))
print(result["cross_section_pb"], result["standard_error_pb"])

decays = json.loads(floops.decay('{"pdg":13,"events":1000,"display":4}'))
diagrams = json.loads(floops.diagrams('{"process":"gluon_gluon"}'))
```

Python 3.10+ is supported. Binary wheels include the Rust backend. A source installation requires Rust and Cargo. Inputs and outputs are JSON strings; invalid requests raise `ValueError`.

| Function | Scope |
| --- | --- |
| `amplitude(request)` | Complex 4–12 gluon tree-level MHV amplitudes; four-gluon SU(3) color/helicity sums |
| `scatter(request)` | Nine massless tree-level QED/QCD channels; weighted events, cross sections and Monte Carlo errors |
| `decay(request)` | Two/three-body phase space, leptonic V−A decays, recursive chains and lifetimes |
| `diagrams(request)` | Process-linked SVG diagrams or all 3–6 gluon tree topologies |
| `lattice(request)` | Periodic 4D pure-gauge SU(3) Wilson lattice Monte Carlo |
| `catalogue()` | 44 model entries, including 20 decaying species |

Natural units are used: energies and masses in GeV, cross sections in pb; lifetimes are in seconds. Amplitudes use all-outgoing momenta. Batch calculations release the Python interpreter while Rust computes.

This is an experimental toolkit with selected physics models, not a full Standard Model event generator. Loop amplitudes, showers, hadronization, dynamical lattice quarks and real-time QCD are not implemented. Unmodelled decay branching fractions are reported explicitly.

The repository also provides a Three.js 3D viewer and a Rust CLI. They are run separately from source and are not bundled in the Python wheel.

[Source](https://github.com/su3co/floops) · [API](https://github.com/su3co/floops/blob/main/docs/API.md) · [Physics and conventions](https://github.com/su3co/floops/blob/main/docs/PHYSICS.md)

MIT license.
