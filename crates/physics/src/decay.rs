use crate::{
    math::{three_body, two_body, FourVector},
    rng::Rng,
    Result,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Channel {
    pub daughters: Vec<i32>,
    pub branching: f64,
    pub model: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Particle {
    pub pdg: i32,
    pub name: String,
    pub mass: f64,
    pub charge_thirds: i32,
    pub lifetime_s: Option<f64>,
    pub channels: Vec<Channel>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    pub particles: Vec<Particle>,
    pub source: String,
}
fn channel(d: &[i32], branching: f64, model: &str) -> Channel {
    Channel {
        daughters: d.to_vec(),
        branching,
        model: model.into(),
    }
}
pub fn catalogue() -> Model {
    let mut particles = vec![];
    let mut add = |pdg, name: &str, mass, charge, lifetime_s, channels| {
        particles.push(Particle {
            pdg,
            name: name.into(),
            mass,
            charge_thirds: charge,
            lifetime_s,
            channels,
        })
    };
    add(22, "γ", 0., 0, None, vec![]);
    add(21, "g", 0., 0, None, vec![]);
    for (id, n) in [(12, "νe"), (14, "νμ"), (16, "ντ")] {
        add(id, n, 0., 0, None, vec![]);
    }
    add(11, "e⁻", 0.000510998950, -3, None, vec![]);
    add(
        13,
        "μ⁻",
        0.1056583755,
        -3,
        Some(2.1969811e-6),
        vec![channel(&[11, -12, 14], 1., "v_a")],
    );
    add(
        15,
        "τ⁻",
        1.77693,
        -3,
        Some(2.903e-13),
        vec![
            channel(&[11, -12, 16], 0.1782, "v_a"),
            channel(&[13, -14, 16], 0.1739, "v_a"),
            channel(&[-211, 16], 0.1082, "phase_space"),
        ],
    );
    add(
        111,
        "π⁰",
        0.1349768,
        0,
        Some(8.43e-17),
        vec![channel(&[22, 22], 0.98823, "phase_space")],
    );
    add(
        211,
        "π⁺",
        0.13957039,
        3,
        Some(2.6033e-8),
        vec![
            channel(&[-13, 14], 0.999877, "phase_space"),
            channel(&[-11, 12], 0.000123, "phase_space"),
        ],
    );
    add(
        321,
        "K⁺",
        0.493677,
        3,
        Some(1.2380e-8),
        vec![
            channel(&[-13, 14], 0.6356, "phase_space"),
            channel(&[211, 111], 0.2067, "phase_space"),
            channel(&[211, 211, -211], 0.05583, "phase_space"),
            channel(&[211, 111, 111], 0.01760, "phase_space"),
        ],
    );
    add(
        310,
        "K short",
        0.497611,
        0,
        Some(8.954e-11),
        vec![
            channel(&[211, -211], 0.69196, "phase_space"),
            channel(&[111, 111], 0.30687, "phase_space"),
        ],
    );
    add(
        130,
        "K long",
        0.497611,
        0,
        Some(5.116e-8),
        vec![
            channel(&[111, 111, 111], 0.1952, "phase_space"),
            channel(&[211, -211, 111], 0.1254, "phase_space"),
        ],
    );
    add(
        221,
        "η",
        0.547862,
        0,
        Some(5.024e-19),
        vec![
            channel(&[22, 22], 0.3936, "phase_space"),
            channel(&[111, 111, 111], 0.3257, "phase_space"),
            channel(&[211, -211, 111], 0.2302, "phase_space"),
        ],
    );
    add(
        113,
        "ρ⁰",
        0.77526,
        0,
        Some(4.414e-24),
        vec![channel(&[211, -211], 1., "phase_space")],
    );
    add(
        213,
        "ρ⁺",
        0.77511,
        3,
        Some(4.41e-24),
        vec![channel(&[211, 111], 1., "phase_space")],
    );
    add(
        223,
        "ω",
        0.78266,
        0,
        Some(7.76e-23),
        vec![
            channel(&[211, -211, 111], 0.892, "phase_space"),
            channel(&[111, 22], 0.0834, "phase_space"),
        ],
    );
    add(
        333,
        "φ",
        1.019461,
        0,
        Some(1.549e-22),
        vec![
            channel(&[321, -321], 0.491, "phase_space"),
            channel(&[310, 130], 0.339, "phase_space"),
        ],
    );
    add(
        443,
        "J/ψ",
        3.0969,
        0,
        Some(7.09e-21),
        vec![
            channel(&[11, -11], 0.05971, "phase_space"),
            channel(&[13, -13], 0.05961, "phase_space"),
        ],
    );
    add(2212, "p", 0.938272088, 3, None, vec![]);
    add(
        2112,
        "n",
        0.939565421,
        0,
        Some(878.4),
        vec![channel(&[2212, 11, -12], 1., "phase_space")],
    );
    // Partons appear in the catalogue for scattering; this engine does not decay or hadronize them.
    for (id, n, m, q) in [
        (1, "d", 0.0047, -1),
        (2, "u", 0.0022, 2),
        (3, "s", 0.096, -1),
        (4, "c", 1.27, 2),
        (5, "b", 4.18, -1),
        (6, "t", 172.57, 2),
    ] {
        add(id, n, m, q, None, vec![]);
    }
    let self_conjugate = [22, 21, 111, 310, 130, 221, 113, 223, 333, 443];
    let anti: Vec<_> = particles
        .iter()
        .filter(|p| !self_conjugate.contains(&p.pdg))
        .map(|p| {
            let mut a = p.clone();
            a.pdg = -p.pdg;
            a.name = match p.pdg {
                11 => "e⁺".into(),
                13 => "μ⁺".into(),
                15 => "τ⁺".into(),
                211 => "π⁻".into(),
                321 => "K⁻".into(),
                213 => "ρ⁻".into(),
                _ => format!("anti-{}", p.name),
            };
            a.charge_thirds = -p.charge_thirds;
            for c in &mut a.channels {
                for id in &mut c.daughters {
                    if !self_conjugate.contains(id) {
                        *id = -*id;
                    }
                }
            }
            a
        })
        .collect();
    particles.extend(anti);
    Model {particles,source:"Rounded PDG 2025 summary values. Selected decay modes only; missing branching probability is reported as unmodeled. Radiative corrections and spin correlations between sequential decays are omitted. Neutrinos are massless. Quark masses are reference parameters, not on-shell hadron masses. https://pdg.lbl.gov/2025/tables/contents_tables.html".into()}
}
impl Model {
    pub fn validate(&self) -> Result<()> {
        let map: HashMap<_, _> = self.particles.iter().map(|p| (p.pdg, p)).collect();
        if self.particles.is_empty()
            || self.particles.len() > 1000
            || map.len() != self.particles.len()
        {
            return Err("Model needs 1–1000 unique PDG IDs".into());
        }
        for p in &self.particles {
            if !p.mass.is_finite()
                || p.mass < 0.
                || p.lifetime_s.is_some_and(|t| !t.is_finite() || t <= 0.)
            {
                return Err(format!("Invalid mass/lifetime for {}", p.name));
            }
            if !p.channels.is_empty() && (p.mass <= 0. || p.lifetime_s.is_none()) {
                return Err("Decaying particles require positive mass and lifetime".into());
            }
            if p.channels.iter().map(|c| c.branching).sum::<f64>() > 1. + 1e-9 {
                return Err(format!("Branching fractions exceed one for {}", p.name));
            }
            for c in &p.channels {
                if !(2..=3).contains(&c.daughters.len())
                    || !c.branching.is_finite()
                    || c.branching < 0.
                    || !matches!(c.model.as_str(), "phase_space" | "v_a")
                    || c.model == "v_a" && c.daughters.len() != 3
                {
                    return Err("Channels need 2 or 3 daughters, a nonnegative branching fraction, and phase_space or v_a model".into());
                }
                let daughters: Vec<_> = c
                    .daughters
                    .iter()
                    .map(|d| {
                        map.get(d)
                            .copied()
                            .ok_or_else(|| format!("Unknown daughter {d}"))
                    })
                    .collect::<Result<_>>()?;
                if daughters.iter().map(|d| d.mass).sum::<f64>() >= p.mass {
                    return Err(format!("Below-threshold decay for {}", p.name));
                }
                if daughters.iter().map(|d| d.charge_thirds).sum::<i32>() != p.charge_thirds {
                    return Err(format!("Charge-violating decay for {}", p.name));
                }
                if c.model == "v_a" && (daughters[1].mass != 0. || daughters[2].mass != 0.) {
                    return Err("V−A kernel requires massless daughters 2 and 3".into());
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DecayRequest {
    pub pdg: i32,
    pub momentum: f64,
    pub events: usize,
    pub seed: u64,
    pub display: usize,
    pub depth: usize,
    pub model: Option<Model>,
}
impl Default for DecayRequest {
    fn default() -> Self {
        Self {
            pdg: 321,
            momentum: 2.,
            events: 20_000,
            seed: 42,
            display: 64,
            depth: 6,
            model: None,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct DecayNode {
    pub pdg: i32,
    pub parent: Option<usize>,
    pub momentum: FourVector,
    pub birth: [f64; 4],
    pub end: [f64; 4],
    pub proper_time_s: f64,
    pub status: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct DecayEvent {
    pub id: usize,
    pub nodes: Vec<DecayNode>,
    pub root_channel: String,
    pub max_conservation_error: f64,
}
#[derive(Serialize, Deserialize)]
pub struct DecayResult {
    pub request: DecayRequest,
    pub events: Vec<DecayEvent>,
    pub channels: BTreeMap<String, usize>,
    pub energy_histogram: Vec<usize>,
    pub max_conservation_error: f64,
    pub mean_root_proper_time_s: f64,
    pub elapsed_ms: f64,
    pub note: String,
}

fn generate(i: usize, r: &DecayRequest, map: &HashMap<i32, &Particle>) -> Result<DecayEvent> {
    let mut rng = Rng::event(r.seed, i);
    let root = map[&r.pdg];
    let mut nodes = vec![DecayNode {
        pdg: r.pdg,
        parent: None,
        momentum: FourVector::new(root.mass.hypot(r.momentum), 0., 0., r.momentum),
        birth: [0.; 4],
        end: [0.; 4],
        proper_time_s: 0.,
        status: String::new(),
    }];
    let mut depths = vec![0];
    let mut index = 0;
    let mut root_channel = "stable".to_string();
    let mut max_error = 0_f64;
    while index < nodes.len() {
        if nodes.len() > 4096 {
            return Err("Decay tree exceeded 4096 particles".into());
        }
        let p = map[&nodes[index].pdg];
        let momentum = nodes[index].momentum;
        if p.channels.is_empty() {
            nodes[index].status = if p.lifetime_s.is_some() {
                "unmodeled"
            } else {
                "stable"
            }
            .into();
            index += 1;
            continue;
        }
        if depths[index] >= r.depth {
            nodes[index].status = "depth_limit".into();
            index += 1;
            continue;
        }
        let proper = -rng.uniform().ln() * p.lifetime_s.unwrap();
        let dt = proper * momentum.e / p.mass;
        let birth = nodes[index].birth;
        let b = momentum.beta();
        let end = [
            birth[0] + dt,
            birth[1] + b[0] * dt * 299792458000.,
            birth[2] + b[1] * dt * 299792458000.,
            birth[3] + b[2] * dt * 299792458000.,
        ];
        nodes[index].proper_time_s = proper;
        nodes[index].end = end;
        let mut u = rng.uniform();
        let selected = p.channels.iter().find(|c| {
            u -= c.branching;
            u < 0.
        });
        let Some(ch) = selected else {
            nodes[index].status = "unmodeled".into();
            if index == 0 {
                root_channel = "unmodeled".into();
            }
            index += 1;
            continue;
        };
        if index == 0 {
            root_channel = ch
                .daughters
                .iter()
                .map(|id| map[id].name.clone())
                .collect::<Vec<_>>()
                .join(" + ");
        }
        let masses: Vec<_> = ch.daughters.iter().map(|id| map[id].mass).collect();
        let daughters = if masses.len() == 2 {
            two_body(momentum, [masses[0], masses[1]], &mut rng)?.to_vec()
        } else {
            let rest = FourVector::rest(p.mass);
            let mut found = None;
            for _ in 0..100_000 {
                let q = three_body(rest, [masses[0], masses[1], masses[2]], &mut rng)?;
                if ch.model == "v_a" {
                    let weight = rest.dot(q[1]) * q[0].dot(q[2]);
                    if rng.uniform() * p.mass.powi(4) / 16. > weight {
                        continue;
                    }
                }
                found = Some(
                    q.iter()
                        .map(|x| x.boost(momentum.beta()))
                        .collect::<Result<Vec<_>>>()?,
                );
                break;
            }
            found.ok_or("Decay matrix-element rejection sampler did not converge")?
        };
        max_error = max_error.max(
            (daughters
                .iter()
                .copied()
                .fold(FourVector::default(), |a, b| a + b)
                - momentum)
                .max_abs(),
        );
        nodes[index].status = "decayed".into();
        let depth = depths[index] + 1;
        for (id, q) in ch.daughters.iter().zip(daughters) {
            nodes.push(DecayNode {
                pdg: *id,
                parent: Some(index),
                momentum: q,
                birth: end,
                end,
                proper_time_s: 0.,
                status: String::new(),
            });
            depths.push(depth);
        }
        index += 1;
    }
    Ok(DecayEvent {
        id: i,
        nodes,
        root_channel,
        max_conservation_error: max_error,
    })
}
pub fn simulate(r: &DecayRequest) -> Result<DecayResult> {
    if !r.momentum.is_finite()
        || r.momentum < 0.
        || r.momentum > 1000.
        || r.events == 0
        || r.events > 200_000
        || r.display > 1024
        || r.depth == 0
        || r.depth > 8
    {
        return Err("Use momentum=0–1000 GeV, 1–200000 events, display ≤1024, depth=1–8".into());
    }
    let model = r.model.clone().unwrap_or_else(catalogue);
    model.validate()?;
    let map: HashMap<_, _> = model.particles.iter().map(|p| (p.pdg, p)).collect();
    let parent = map.get(&r.pdg).ok_or("Unknown parent PDG ID")?;
    if parent.mass <= 0. || parent.channels.is_empty() {
        return Err("Select a massive particle with implemented decay channels".into());
    }
    // Avoid cancellation destroying the invariant mass of extremely boosted light particles.
    if r.momentum / parent.mass > 10_000. {
        return Err("Parent boost exceeds numerical limit gamma ~10000".into());
    }
    let start = std::time::Instant::now();
    let chunks = r.events.div_ceil(256);
    let parts: Result<Vec<_>> = (0..chunks)
        .into_par_iter()
        .map(|k| {
            let mut channels = BTreeMap::<String, usize>::new();
            let mut hist = vec![0; 60];
            let mut samples = vec![];
            let mut error = 0_f64;
            let mut time = 0.;
            for i in k * 256..((k + 1) * 256).min(r.events) {
                let e = generate(i, r, &map)?;
                *channels.entry(e.root_channel.clone()).or_default() += 1;
                error = error.max(e.max_conservation_error);
                time += e.nodes[0].proper_time_s;
                if let Some(first) = e.nodes.get(1) {
                    let p = first
                        .momentum
                        .boost(e.nodes[0].momentum.beta().map(|x| -x))?;
                    let bin = ((2. * p.e / parent.mass * 60.) as usize).min(59);
                    hist[bin] += 1;
                }
                if i < r.display {
                    samples.push(e);
                }
            }
            Ok((channels, hist, samples, error, time))
        })
        .collect();
    let mut channels = BTreeMap::new();
    let mut histogram = vec![0; 60];
    let mut events = vec![];
    let mut max_error = 0_f64;
    let mut time = 0.;
    for (c, h, mut e, err, t) in parts? {
        for (k, v) in c {
            *channels.entry(k).or_default() += v;
        }
        for (a, b) in histogram.iter_mut().zip(h) {
            *a += b;
        }
        events.append(&mut e);
        max_error = max_error.max(err);
        time += t;
    }
    Ok(DecayResult {
        request: r.clone(),
        events,
        channels,
        energy_histogram: histogram,
        max_conservation_error: max_error,
        mean_root_proper_time_s: time / r.events as f64,
        elapsed_ms: start.elapsed().as_secs_f64() * 1000.,
        note: model.source,
    })
}
