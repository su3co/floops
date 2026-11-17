"""Integration check of the installed native extension; no Python physics implementation."""
import json
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import floops as physics
assert physics.__version__ == "0.1.0"

scatter = json.loads(physics.scatter('{"events":10000,"display":4,"seed":42}'))
assert scatter["cross_section_pb"] > 0
assert len(scatter["events"]) == 4
decay = json.loads(physics.decay('{"pdg":13,"momentum":0,"events":1000,"display":4}'))
assert decay["max_conservation_error"] < 1e-10
lattice = json.loads(physics.lattice('{"length":2,"warmup":2,"sweeps":3,"chains":2}'))
assert len(lattice["chains"]) == 2
assert max(c["max_unitarity_error"] for c in lattice["chains"]) < 1e-10
amplitude = json.loads(physics.amplitude((Path(__file__).parent / "amplitude.json").read_text()))
assert amplitude["norm_squared"] > 0
assert amplitude["four_gluon_spin_color_averaged"] > 0
assert len(json.loads(physics.catalogue())["particles"]) == 44
assert len(json.loads(physics.diagrams('{"legs":6}'))) == 220
diagrams = json.loads(physics.diagrams('{"process":"gluon_gluon"}'))
assert {d["channel"] for d in diagrams} == {"s", "t", "u", "contact"}
assert all(d["svg"].startswith("<svg") for d in diagrams)
for request in ['{"legs":7}', '{"process":"invalid"}', '{"legs":4,"process":"gluon_gluon"}']:
    try:
        physics.diagrams(request)
    except ValueError:
        pass
    else:
        raise AssertionError("Invalid diagram request was accepted")
with ThreadPoolExecutor(2) as pool:
    parallel = list(pool.map(physics.scatter, ['{"events":10000,"seed":42}'] * 2))
assert json.loads(parallel[0])["histogram"] == json.loads(parallel[1])["histogram"]
try:
    physics.scatter('{"energy":-1}')
except ValueError:
    pass
else:
    raise AssertionError("Invalid parameters were accepted")
print("Python native extension: scattering, decays, lattice, amplitude, diagrams, catalogue, concurrency and errors passed.")
