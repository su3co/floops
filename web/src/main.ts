import "@fontsource/ibm-plex-sans/400.css";
import "@fontsource/ibm-plex-sans/500.css";
import "@fontsource/ibm-plex-sans/600.css";
import "@fontsource/ibm-plex-serif/400.css";
import "@fontsource/ibm-plex-serif/400-italic.css";
import "./style.css";
import { Chamber, type Momentum } from "./chamber";

type Mode = "scattering" | "decays" | "amplitudes" | "lattice";
type Particle = {
  pdg: number;
  name: string;
  mass: number;
  lifetime_s: number | null;
  channels: { branching: number }[];
};
type Tree =
  { kind: "Leaf"; children: number } | { kind: "Vertex"; children: Tree[] };
const $ = <T extends HTMLElement = HTMLElement>(s: string) =>
  document.querySelector<T>(s)!;
const escape = (s: unknown) =>
  String(s).replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        c
      ]!,
  );
const number = (n: number, d = 3) =>
  Number.isFinite(n)
    ? Math.abs(n) >= 1e5 || (Math.abs(n) > 0 && Math.abs(n) < 0.001)
      ? n.toExponential(d)
      : n.toLocaleString(undefined, { maximumFractionDigits: d })
    : "—";
const post = async (path: string, body: unknown) => {
  const r = await fetch("/api/" + path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const raw = await r.text();
  let data;
  try {
    data = JSON.parse(raw);
  } catch {
    throw new Error(
      raw.slice(0, 250) || "The server returned an empty response.",
    );
  }
  if (!r.ok) throw new Error(data.error || "Calculation failed.");
  return data;
};
let mode: Mode = "scattering",
  processes: { id: string; label: string }[] = [],
  particles: Particle[] = [],
  requestId = 0,
  lastResult: any = null,
  lastDiagrams: { svg: string; channel: string; mediator: string }[] = [];
let chamber: Chamber | undefined;
$("#app").innerHTML =
  `<header class="masthead"><a href="/" class="wordmark" aria-label="unnamed home"><span class="symbol">∿</span>unnamed</a><span class="descriptor">Particle laboratory</span><a class="source-link" href="https://github.com/su3-labs/unnamed" target="_blank" rel="noreferrer">Source & documentation</a></header>
<main><div class="intro"><div><h1>Particle experiments</h1><p>Scattering, decays and SU(3) gauge fields.</p></div><div class="engine" id="engine">Connecting to the Rust engine…</div></div>
<nav class="modes" aria-label="Experiments"><button data-mode="scattering" aria-pressed="true">Scattering</button><button data-mode="decays" aria-pressed="false">Decays</button><button data-mode="amplitudes" aria-pressed="false">Amplitudes</button><button data-mode="lattice" aria-pressed="false">SU(3) lattice</button></nav>
<section class="bench" aria-label="Experiment bench"><aside class="parameters"><div class="panel-heading"><h2 id="experiment-title">Scattering experiment</h2><span id="model-tag">Tree level</span></div><form id="parameters"><div id="fields"></div><button class="run" type="submit" id="run">Run simulation <span>↗</span></button></form><div class="status" id="status" role="status" aria-live="polite">Preparing the experiment.</div><p class="scope" id="scope"></p></aside>
<div class="visual"><div class="chamber" id="chamber"><div class="scene-heading"><span id="scene-title">Parton collision</span><span id="scene-detail">Center-of-mass frame</span></div><div class="scene-legend" id="legend"><span class="quark">Quarks</span><span class="gluon">Gluons</span><span class="lepton">Leptons</span></div><div class="orbit-hint">Drag to orbit · Scroll to zoom</div><button class="camera" id="camera" aria-label="Reset camera">Reset view</button></div><div class="timeline"><button id="play" aria-label="Play animation">Play</button><label for="timeline" id="timeline-label">Event phase</label><input type="range" id="timeline" min="0" max="1" step="0.001" value="0.7"><span id="time-value">70%</span></div><div class="scene-caption" id="scene-caption">Straight trajectories from computed four-momenta. Chamber geometry is a visual reference.</div></div></section>
<section class="analysis" aria-label="Calculation results"><div class="result-summary"><div class="panel-heading"><h2>Calculation</h2><button class="text-button" id="export" disabled>Export JSON</button></div><div id="numbers" class="numbers"><p>Results will appear after the first run.</p></div><div id="event-picker"></div></div><div class="plot-panel"><div class="panel-heading"><h2 id="plot-title">Angular distribution</h2><button class="text-button" id="export-csv" disabled>Export CSV</button></div><div id="plot" class="plot"></div><p class="plot-note" id="plot-note">Weighted events, integrated within the angular cut.</p></div></section>
<section class="diagram-panel"><div class="panel-heading"><h2 id="diagram-title">Feynman diagrams</h2><span id="diagram-note">Contributing tree diagrams</span></div><div id="diagrams" class="diagrams"></div></section>
<details class="physics-notes"><summary>Physics, units and limitations</summary><div id="physics-notes"></div><p>References: <a href="https://pdg.lbl.gov/2025/reviews/rpp2025-rev-cross-section-formulae.pdf">PDG cross sections</a>, <a href="https://pdg.lbl.gov/2025/reviews/rpp2025-rev-kinematics.pdf">relativistic kinematics</a>, <a href="https://arxiv.org/abs/1101.2414">spinor-helicity amplitudes</a>, <a href="https://www.damtp.cam.ac.uk/user/tong/gaugetheory.html">lattice gauge theory</a>.</p></details>
</main><footer><span>Built around the calculation.</span><span>Rust physics · Three.js visualization · Reproducible seeds</span></footer>`;

const field = (
  id: string,
  label: string,
  value: number,
  min: number,
  max: number,
  step: number | string = 1,
  help = "",
) =>
  `<label class="field" for="${id}"><span>${label}</span><input id="${id}" name="${id}" type="number" value="${value}" min="${min}" max="${max}" step="${step}" required>${help ? `<small>${help}</small>` : ""}</label>`;
const select = (
  id: string,
  label: string,
  options: [string | number, string][],
  value: string | number,
) =>
  `<label class="field" for="${id}"><span>${label}</span><select id="${id}" name="${id}">${options.map(([v, l]) => `<option value="${escape(v)}" ${v == value ? "selected" : ""}>${escape(l)}</option>`).join("")}</select></label>`;
const pair = (a: string, b: string) => `<div class="field-pair">${a}${b}</div>`;
function setMode(m: Mode) {
  requestId++;
  mode = m;
  lastResult = null;
  lastDiagrams = [];
  $("#export").setAttribute("disabled", "");
  $("#export-csv").setAttribute("disabled", "");
  $("#numbers").innerHTML =
    "<p>Run this experiment to calculate its observables.</p>";
  $("#plot").innerHTML = "";
  $("#diagrams").innerHTML = "";
  $("#event-picker").innerHTML = "";
  $("#status").textContent = "Ready.";
  $("#plot-title").textContent = "Results";
  $("#plot-note").textContent = "";
  $("#physics-notes").textContent =
    "Run this experiment to see its calculation conventions.";
  $("#scene-caption").textContent =
    "Run the experiment to populate the 3D view.";
  document
    .querySelectorAll<HTMLButtonElement>("[data-mode]")
    .forEach((b) =>
      b.setAttribute("aria-pressed", String(b.dataset.mode === mode)),
    );
  const titles = {
    scattering: ["Scattering experiment", "Tree level"],
    decays: ["Particle decays", "Selected PDG modes"],
    amplitudes: ["Gluon amplitudes", "Complex MHV"],
    lattice: ["Gauge-field experiment", "Pure SU(3)"],
  };
  $("#experiment-title").textContent = titles[m][0];
  $("#model-tag").textContent = titles[m][1];
  if (m === "scattering") {
    $("#fields").innerHTML =
      select(
        "process",
        "Interaction",
        processes.map((p) => [p.id, p.label]),
        "gluon_gluon",
      ) +
      pair(
        field("energy", "√s / GeV", 200, 1, 1e7, "any"),
        field("alpha", "αs", 0.118, 0.0001, 1, "any"),
      ) +
      field(
        "events",
        "Events",
        100000,
        2,
        5000000,
        1,
        "All events contribute to the distribution.",
      ) +
      pair(
        field("cos_cut", "|cos θ| cut", 0.9, 0.01, 0.999, "any"),
        field("seed", "Seed", 42, 0, 2147483647),
      ) +
      `<p class="field-note">Initial spins and colors are averaged. Final colors and spins are summed.</p>`;
    $("#scope").textContent =
      "Massless, fixed-coupling partons. No proton PDFs, showers, hadronization or detector simulation.";
    $("#scene-title").textContent = "Parton collision";
    $("#scene-detail").textContent = "Center-of-mass frame";
    $("#process").addEventListener("change", () => {
      const qed =
        ($("#process") as HTMLSelectElement).value === "electron_muon";
      ($("#alpha") as HTMLInputElement).value = qed
        ? String(1 / 137.035999084)
        : ".118";
      $("#alpha").previousElementSibling!.textContent = qed
        ? "α electromagnetic"
        : "αs";
    });
  }
  if (m === "decays") {
    $("#fields").innerHTML =
      select(
        "pdg",
        "Parent particle",
        particles
          .filter((p) => p.channels.length > 0)
          .map((p) => [p.pdg, `${p.name} · ${number(p.mass, 5)} GeV`]),
        321,
      ) +
      field("momentum", "Parent momentum / GeV", 2, 0, 1000, "any") +
      field("events", "Decay chains", 20000, 1, 200000) +
      pair(
        field("depth", "Maximum depth", 6, 1, 8),
        field("seed", "Seed", 42, 0, 2147483647),
      ) +
      `<p class="field-note">Proper lifetimes are exponentially sampled. Muon and leptonic tau decays use an unpolarized V−A kernel.</p>`;
    $("#scope").textContent =
      "Other channels use phase space. Missing branching fractions remain unmodeled. No spin correlations between successive decays.";
    $("#scene-title").textContent = "Decay cascade";
    $("#scene-detail").textContent = "Directions from four-momenta";
  }
  if (m === "amplitudes") {
    $("#fields").innerHTML =
      pair(
        field("energy", "√s / GeV", 200, 1, 1e7, "any"),
        field("legs", "External gluons", 4, 4, 6),
      ) +
      pair(
        field("negative1", "Negative leg i", 1, 1, 12),
        field("negative2", "Negative leg j", 2, 1, 12),
      ) +
      field("coupling", "Gauge coupling g", 1.2177, 0, 10, "any") +
      `<details class="custom-momenta"><summary>Custom four-momenta</summary><p>Optional JSON array of 4–12 massless momenta. All outgoing: incoming energies are negative. Sum must be zero.</p><textarea id="momenta" aria-label="Custom all-outgoing four-momenta" placeholder='[{"e":-100,"px":0,"py":0,"pz":-100}, …]'></textarea></details><p class="formula">Aₙ = i gⁿ⁻² ⟨ij⟩⁴ / ∏⟨k,k+1⟩</p>`;
    $("#scope").textContent =
      "Tree MHV amplitudes with exactly two negative helicities. Fixed cyclic ordering, stripped color. This is a complex amplitude, not a color-summed probability.";
    $("#scene-title").textContent = "Gluon kinematics";
    $("#scene-detail").textContent = "All-outgoing convention";
  }
  if (m === "lattice") {
    $("#fields").innerHTML =
      pair(
        field("length", "Lattice side L", 4, 2, 10),
        field("beta", "β = 6 / g²", 5.7, 0, 30, "any"),
      ) +
      pair(
        field("sweeps", "Measurement sweeps", 30, 1, 300),
        field("warmup", "Warmup sweeps", 20, 0, 300),
      ) +
      pair(
        field("chains", "Independent chains", 2, 1, 16),
        field("step", "Proposal step", 0.35, 0.01, 3, "any"),
      ) +
      select(
        "hot",
        "Starting field",
        [
          [0, "Cold · identity links"],
          [1, "Hot · Haar-distributed links"],
        ],
        0,
      ) +
      field("seed", "Seed", 7, 0, 2147483647);
    $("#scope").textContent =
      "Periodic, four-dimensional Euclidean Wilson lattice. Parallel independent chains; no dynamical quarks. Monte Carlo sweeps are not physical time.";
    $("#scene-title").textContent = "SU(3) gauge field";
    $("#scene-detail").textContent = "Spatial slice at Euclidean t = 0";
  }
  $("#run").innerHTML =
    (m === "amplitudes" ? "Compute amplitude" : "Run simulation") +
    " <span>↗</span>";
  $("#run").removeAttribute("disabled");
  $("#timeline-label").textContent =
    m === "lattice" ? "Saved configuration" : "Event phase";
  $("#legend").innerHTML =
    m === "lattice"
      ? '<span class="gluon">Low plaquette</span><span class="lepton">High plaquette</span>'
      : '<span class="quark">Quarks / hadrons</span><span class="gluon">Gluons</span><span class="lepton">Leptons</span>';
  $("#diagram-title").textContent =
    m === "lattice"
      ? "SU(3) observables"
      : m === "decays"
        ? "Decay channels"
        : "Feynman diagrams";
  $("#diagram-note").textContent =
    m === "amplitudes"
      ? "Full gluon tree topologies; MHV amplitudes use a fixed color ordering"
      : m === "scattering"
        ? "Amplitudes interfere; individual diagrams are not probabilities"
        : "";
  if (chamber) {
    chamber.playing = false;
    chamber.clear();
    $("#play").textContent = "Play";
    $("#play").setAttribute("aria-label", "Play animation");
  }
  $("#status").classList.remove("error");
}
function read(id: string) {
  return Number(($(`#${id}`) as HTMLInputElement).value);
}
function rows(values: [string, string][], formula = "") {
  $("#numbers").innerHTML =
    (formula ? `<div class="result-formula">${formula}</div>` : "") +
    `<dl>${values.map(([a, b]) => `<div><dt>${escape(a)}</dt><dd>${escape(b)}</dd></div>`).join("")}</dl>`;
}
function plot(
  values: number[],
  xLabel: string,
  yLabel: string,
  errors?: number[],
  line = false,
) {
  const w = Math.max(340, Math.min(660, $("#plot").clientWidth)),
    h = 220,
    left = 66,
    right = 20,
    top = 16,
    bottom = 42;
  const high =
    Math.max(...values.map((v, i) => v + (errors?.[i] || 0)), 1e-12) * 1.1;
  const plotW = w - left - right,
    plotH = h - top - bottom;
  const base = h - bottom;
  let svg = `<svg viewBox="0 0 ${w} ${h}" role="img" aria-label="${escape(yLabel)} versus ${escape(xLabel)}"><title>${escape(yLabel)} versus ${escape(xLabel)}</title>`;
  for (let i = 0; i <= 4; i++) {
    const val = (high * i) / 4,
      y = base - (plotH * i) / 4;
    svg += `<line x1="${left}" y1="${y}" x2="${w - right}" y2="${y}" stroke="#d9e1e9"/><text x="${left - 9}" y="${y + 4}" text-anchor="end" font-size="10" fill="#536a80">${number(val, 2)}</text>`;
  }
  if (line) {
    svg += `<polyline fill="none" stroke="#356689" stroke-width="2" points="${values.map((v, i) => `${left + (i / Math.max(1, values.length - 1)) * plotW},${base - (v / high) * plotH}`).join(" ")}"/>`;
  } else
    values.forEach((v, i) => {
      const x = left + (i / values.length) * plotW,
        bw = plotW / values.length - 1,
        y = base - (v / high) * plotH;
      svg += `<rect x="${x}" y="${y}" width="${bw}" height="${Math.max(0, base - y)}" fill="#597f9e"><title>Bin ${i + 1}: ${number(v, 6)}${errors ? " ± " + number(errors[i], 6) : ""}</title></rect>`;
      if (errors) {
        const mid = x + bw / 2,
          dy = (errors[i] / high) * plotH;
        svg += `<path d="M${mid},${y - dy} V${y + dy}" stroke="#17293d" stroke-width="1"/>`;
      }
    });
  svg += `<text x="${left}" y="${h - 18}" font-size="11" fill="#536a80">${escape(xLabel)}</text><text x="${left}" y="12" font-size="10" fill="#536a80">${escape(yLabel)}</text></svg>`;
  $("#plot").innerHTML = svg;
}
function download(name: string, content: string, type = "application/json") {
  const url = URL.createObjectURL(new Blob([content], { type }));
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
function renderDiagrams(diagrams: typeof lastDiagrams) {
  lastDiagrams = diagrams;
  $("#diagrams").innerHTML = diagrams
    .map(
      (d, i) =>
        `<figure>${d.svg}<figcaption><span>${escape(d.channel === "contact" ? "Four-gluon vertex" : d.channel + " channel · " + d.mediator)}</span><button data-svg="${i}" class="text-button">SVG</button></figcaption></figure>`,
    )
    .join("");
  document
    .querySelectorAll<HTMLButtonElement>("[data-svg]")
    .forEach(
      (b) =>
        (b.onclick = () =>
          download(
            `diagram-${b.dataset.svg}.svg`,
            diagrams[Number(b.dataset.svg)].svg,
            "image/svg+xml",
          )),
    );
}
function treeSvg(tree: Tree) {
  let lines = "",
    labels = "",
    count = 0;
  const leaves = (t: Tree): number =>
    t.kind === "Leaf" ? 1 : t.children.reduce((s, c) => s + leaves(c), 0);
  const total = leaves(tree);
  const walk = (t: Tree, depth: number): [number, number] => {
    if (t.kind === "Leaf") {
      const x = 20 + ((++count - 0.5) / total) * 200;
      labels += `<text x="${x}" y="154" text-anchor="middle" font-size="12" fill="#17293d">g${t.children + 1}</text>`;
      return [x, 136];
    }
    const kids = t.children.map((c) => walk(c, depth + 1));
    const x = kids.reduce((s, c) => s + c[0], 0) / kids.length,
      y = 18 + depth * 26;
    for (const p of kids)
      lines += `<path d="M${x},${y} L${p[0]},${p[1]}" stroke="#637aa3" fill="none"/>`;
    if (kids.length > 2 || depth > 0)
      lines += `<circle cx="${x}" cy="${y}" r="2" fill="#254d75"/>`;
    return [x, y];
  };
  walk(tree, 0);
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 240 172" role="img"><title>Gluon tree topology; lines denote gluons</title>${lines}${labels}</svg>`;
}
function setPicker(
  count: number,
  action: (i: number) => void,
  label = "Displayed event",
) {
  if (count < 1) return;
  $("#event-picker").innerHTML =
    `<label for="event-index">${label} <output id="event-number">1 / ${count}</output></label><input id="event-index" type="range" min="0" max="${count - 1}" value="0" step="1">`;
  $("#event-index").oninput = () => {
    const i = read("event-index");
    $("#event-number").textContent = `${i + 1} / ${count}`;
    action(i);
    if (chamber) {
      ($("#timeline") as HTMLInputElement).value = String(chamber.time);
      $("#time-value").textContent = Math.round(chamber.time * 100) + "%";
    }
  };
}
function drawDecay(data: any, i: number) {
  const e = data.events[i];
  chamber?.setDecay(e.nodes);
  $("#scene-detail").textContent =
    `Event ${e.id + 1} · ${e.nodes.length} particles`;
  $("#scene-caption").textContent =
    "Directions are physical; segment lengths and playback timing are schematic so short and long lifetimes remain visible. Export contains lab times (s) and vertices (mm).";
}
function amplitudeMomenta(n: number, energy: number): Momentum[] {
  const p: Momentum[] = [
    { e: -energy / 2, px: 0, py: 0, pz: -energy / 2 },
    { e: -energy / 2, px: 0, py: 0, pz: energy / 2 },
  ];
  for (let k = 0; k < n - 2; k++) {
    const phi = 0.4 + (k / (n - 2)) * 2 * Math.PI;
    const e = energy / (n - 2);
    p.push({ e, px: e * Math.cos(phi), py: e * Math.sin(phi), pz: 0 });
  }
  return p;
}
async function run() {
  const id = ++requestId,
    current = mode;
  lastResult = null;
  $("#export").setAttribute("disabled", "");
  $("#export-csv").setAttribute("disabled", "");
  $("#numbers").innerHTML = "<p>Calculating…</p>";
  $("#plot").innerHTML = "";
  $("#diagrams").innerHTML = "";
  $("#event-picker").innerHTML = "";
  chamber?.clear();
  $("#run").setAttribute("disabled", "");
  $("#status").textContent = "Computing in Rust…";
  $("#status").classList.remove("error");
  const started = performance.now();
  try {
    let data: any;
    if (current === "scattering") {
      const request = {
        process: ($("#process") as HTMLSelectElement).value,
        energy: read("energy"),
        alpha: read("alpha"),
        events: read("events"),
        cos_cut: read("cos_cut"),
        seed: read("seed"),
        display: 256,
      };
      data = await post("scatter", request);
      const diagrams = await post("diagrams", { process: request.process });
      if (id !== requestId) return;
      chamber?.setScatter(data.events);
      rows(
        [
          [
            "Integrated cross section",
            `${number(data.cross_section_pb, 5)} ± ${number(data.standard_error_pb, 4)} pb`,
          ],
          ["Events calculated", number(request.events, 0)],
          [
            "Events displayed",
            String(data.events.length) + " weighted samples",
          ],
          [
            "Rust throughput",
            number(data.events_per_second / 1e6, 2) + " million events/s",
          ],
          ["Calculation time", number(data.elapsed_ms, 1) + " ms"],
        ],
        `${escape(processes.find((p) => p.id === request.process)?.label)}`,
      );
      plot(
        data.histogram,
        `cos θ from −${request.cos_cut} to +${request.cos_cut}`,
        "Cross section per bin / pb",
        data.histogram_error,
      );
      $("#plot-title").textContent = "Angular distribution";
      $("#plot-note").textContent =
        "Error bars show Monte Carlo standard errors. Integrating all bins gives the cross section; display samples are uniformly drawn, not unweighted events.";
      $("#scene-caption").textContent =
        "Straight trajectories from computed four-momenta. 256 weighted events overlaid; the highlighted event is selected below. The chamber is a visual reference.";
      setPicker(data.events.length, (i) => chamber?.setScatter(data.events, i));
      renderDiagrams(diagrams.diagrams);
    }
    if (current === "decays") {
      const request = {
        pdg: read("pdg"),
        momentum: read("momentum"),
        events: read("events"),
        depth: read("depth"),
        seed: read("seed"),
        display: 64,
      };
      data = await post("decay", request);
      if (id !== requestId) return;
      drawDecay(data, 0);
      const p = particles.find((p) => p.pdg === request.pdg)!;
      rows(
        [
          ["Decay chains", number(request.events, 0)],
          [
            "Mean proper lifetime",
            number(data.mean_root_proper_time_s, 4) + " s",
          ],
          ["Reference lifetime", number(p.lifetime_s!, 4) + " s"],
          [
            "Max four-momentum residual",
            number(data.max_conservation_error, 2) + " GeV",
          ],
          ["Calculation time", number(data.elapsed_ms, 1) + " ms"],
        ],
        escape(p.name) + " decay",
      );
      plot(
        data.energy_histogram,
        "x = 2E / parent mass, from 0 to 1",
        "First daughter counts",
      );
      $("#plot-title").textContent = "Daughter energy in parent rest frame";
      $("#plot-note").textContent =
        "The first listed daughter of each modeled root decay contributes. Missing modes are counted separately below.";
      setPicker(data.events.length, (i) => drawDecay(data, i));
      $("#diagrams").innerHTML =
        `<table class="channel-table"><thead><tr><th>Root decay channel</th><th>Sampled chains</th><th>Fraction</th></tr></thead><tbody>${Object.entries(
          data.channels,
        )
          .map(
            ([name, n]) =>
              `<tr><td>${escape(name)}</td><td>${number(n as number, 0)}</td><td>${number(((n as number) / request.events) * 100, 2)}%</td></tr>`,
          )
          .join("")}</tbody></table>`;
    }
    if (current === "amplitudes") {
      const custom = ($("#momenta") as HTMLTextAreaElement).value.trim();
      const momenta = custom
        ? JSON.parse(custom)
        : amplitudeMomenta(read("legs"), read("energy"));
      const request = {
        momenta,
        negative: [read("negative1") - 1, read("negative2") - 1],
        coupling: read("coupling"),
      };
      data = await post("amplitude", request);
      data.request = request;
      const trees =
        momenta.length <= 6
          ? await post("diagrams", { legs: momenta.length })
          : null;
      if (id !== requestId) return;
      chamber?.setAmplitude(momenta);
      rows(
        [
          ["Real A", number(data.real, 8)],
          ["Imaginary A", number(data.imaginary, 8)],
          ["|A|² (color stripped)", number(data.norm_squared, 8)],
          [
            "Spin/color averaged |M|²",
            data.four_gluon_spin_color_averaged === null
              ? "Available for four gluons"
              : number(data.four_gluon_spin_color_averaged, 6),
          ],
          ["External legs", String(momenta.length)],
          [
            "Negative helicities",
            `${request.negative[0] + 1}, ${request.negative[1] + 1}`,
          ],
        ],
        "Parke–Taylor MHV",
      );
      $("#plot-title").textContent = "External four-momenta";
      $("#plot").innerHTML =
        `<div class="table-scroll"><table><thead><tr><th>Leg</th><th>E</th><th>px</th><th>py</th><th>pz</th></tr></thead><tbody>${momenta.map((p: Momentum, i: number) => `<tr><td>g${i + 1}</td>${[p.e, p.px, p.py, p.pz].map((v) => `<td>${number(v, 3)}</td>`).join("")}</tr>`).join("")}</tbody></table></div>`;
      $("#plot-note").textContent =
        "GeV, metric (+−−−). Negative energies mark incoming legs. A has mass dimension 4−n. Helicity indices follow the all-outgoing convention.";
      $("#scene-caption").textContent =
        "Kinematics of the evaluated color-ordered amplitude. Complex phases depend on spinor conventions; squared magnitudes are invariant under little-group phases.";
      if (trees) {
        data.topologies = trees;
        $("#diagram-note").textContent =
          `${trees.count} complete tree topologies from 3- and 4-gluon vertices. MHV uses one cyclic color ordering.`;
        const all = trees.trees.map((t: Tree) => ({
          svg: treeSvg(t),
          channel: "topology",
          mediator: "gluon",
        }));
        renderDiagrams(all.slice(0, 8));
        if (all.length > 8) {
          const b = document.createElement("button");
          b.className = "text-button more-diagrams";
          b.textContent = `Show all ${all.length} topologies`;
          b.onclick = () => renderDiagrams(all);
          $("#diagrams").append(b);
        }
      } else {
        $("#diagrams").innerHTML =
          "<p>Amplitudes support up to 12 legs; diagram enumeration is capped at 6 to bound combinatorial growth.</p>";
      }
    }
    if (current === "lattice") {
      const request = {
        length: read("length"),
        beta: read("beta"),
        sweeps: read("sweeps"),
        warmup: read("warmup"),
        chains: read("chains"),
        step: read("step"),
        hot: read("hot") === 1,
        seed: read("seed"),
      };
      data = await post("lattice", request);
      if (id !== requestId) return;
      const showChain = (index: number) => {
        const c = data.chains[index];
        chamber?.setLattice(c.frames, request.length);
        const last = c.measurements.at(-1);
        rows(
          [
            [
              "Four-dimensional volume",
              `${request.length}⁴ = ${request.length ** 4} sites`,
            ],
            [`Mean plaquette · chain ${index + 1}`, number(last.plaquette, 6)],
            ["Acceptance · last sweep", number(last.acceptance * 100, 1) + "%"],
            [
              "SU(3) unitarity residual",
              number(
                Math.max(...data.chains.map((c: any) => c.max_unitarity_error)),
                2,
              ),
            ],
            ["Calculation time", number(data.elapsed_ms, 1) + " ms"],
          ],
          "S = β ∑ₚ (1 − Re Tr Uₚ / 3)",
        );
        plot(
          c.measurements.map((m: any) => m.plaquette),
          `Measurement sweeps 1–${request.sweeps}`,
          "Mean plaquette",
          undefined,
          true,
        );
        $("#plot-title").textContent = `Plaquette history · chain ${index + 1}`;
      };
      showChain(0);
      $("#plot-note").textContent =
        `After ${request.warmup} discarded warmup sweeps. This trace does not establish equilibration or independent samples; analyze autocorrelations before precision inference.`;
      $("#scene-caption").textContent =
        "Spatial slice at Euclidean t = 0. Each site color is the mean real normalized spatial plaquette at that site, a gauge-invariant observable. Links show lattice connectivity.";
      setPicker(data.chains.length, showChain, "Independent chain");
      $("#diagrams").innerHTML =
        `<table><thead><tr><th>Chain</th><th>Final plaquette</th><th>Wilson loop 2×2</th><th>Polyakov loop (Re, Im)</th></tr></thead><tbody>${data.chains
          .map((c: any, i: number) => {
            const m = c.measurements.at(-1);
            return `<tr><td>${i + 1}</td><td>${number(m.plaquette, 6)}</td><td>${number(m.wilson_2x2, 6)}</td><td>${number(m.polyakov[0], 5)}, ${number(m.polyakov[1], 5)}</td></tr>`;
          })
          .join("")}</tbody></table>`;
    }
    if (id !== requestId) return;
    lastResult = data;
    $("#export").removeAttribute("disabled");
    $("#export-csv").removeAttribute("disabled");
    $("#physics-notes").textContent = data.note || data.convention;
    $("#status").textContent =
      `Complete in ${number((performance.now() - started) / 1000, 2)} s. Parameters are included in the export.`;
    if (chamber) {
      ($("#timeline") as HTMLInputElement).value = String(chamber.time);
      $("#time-value").textContent = Math.round(chamber.time * 100) + "%";
    }
  } catch (e) {
    if (id === requestId) {
      $("#numbers").innerHTML =
        "<p>No result. Adjust the parameters and run again.</p>";
      $("#status").textContent = e instanceof Error ? e.message : String(e);
      $("#status").classList.add("error");
    }
  } finally {
    if (id === requestId) $("#run").removeAttribute("disabled");
  }
}

$("#parameters").onsubmit = (e) => {
  e.preventDefault();
  void run();
};
document
  .querySelectorAll<HTMLButtonElement>("[data-mode]")
  .forEach((b) => (b.onclick = () => setMode(b.dataset.mode as Mode)));
$("#play").onclick = () => {
  if (!chamber) return;
  chamber.playing = !chamber.playing;
  $("#play").textContent = chamber.playing ? "Pause" : "Play";
  $("#play").setAttribute(
    "aria-label",
    chamber.playing ? "Pause animation" : "Play animation",
  );
};
$("#timeline").oninput = () => {
  if (chamber) {
    chamber.time = read("timeline");
    chamber.playing = false;
    $("#play").textContent = "Play";
    $("#play").setAttribute("aria-label", "Play animation");
  }
  $("#time-value").textContent = Math.round(read("timeline") * 100) + "%";
};
$("#camera").onclick = () => chamber?.resetCamera();
$("#export").onclick = () =>
  lastResult &&
  download(`unnamed-${mode}.json`, JSON.stringify(lastResult, null, 2));
$("#export-csv").onclick = () => {
  if (!lastResult) return;
  let csv = "";
  if (mode === "scattering") {
    csv =
      "cos_min,cos_max,cross_section_pb,standard_error_pb\n" +
      lastResult.histogram
        .map(
          (v: number, i: number) =>
            `${-lastResult.request.cos_cut + (i * 2 * lastResult.request.cos_cut) / 60},${-lastResult.request.cos_cut + ((i + 1) * 2 * lastResult.request.cos_cut) / 60},${v},${lastResult.histogram_error[i]}`,
        )
        .join("\n");
  }
  if (mode === "decays")
    csv =
      "x_min,x_max,count\n" +
      lastResult.energy_histogram
        .map((v: number, i: number) => `${i / 60},${(i + 1) / 60},${v}`)
        .join("\n");
  if (mode === "lattice")
    csv =
      "chain,sweep,plaquette,acceptance,wilson_2x2,polyakov_re,polyakov_im\n" +
      lastResult.chains
        .flatMap((c: any, i: number) =>
          c.measurements.map(
            (m: any) =>
              `${i},${m.sweep},${m.plaquette},${m.acceptance},${m.wilson_2x2},${m.polyakov[0]},${m.polyakov[1]}`,
          ),
        )
        .join("\n");
  if (mode === "amplitudes")
    csv =
      "leg,e,px,py,pz\n" +
      lastResult.request.momenta
        .map(
          (p: Momentum, i: number) => `${i + 1},${p.e},${p.px},${p.py},${p.pz}`,
        )
        .join("\n");
  download(`unnamed-${mode}.csv`, csv, "text/csv");
};
try {
  chamber = new Chamber($("#chamber"));
  chamber.onTime = (t) => {
    ($("#timeline") as HTMLInputElement).value = String(t);
    $("#time-value").textContent = Math.round(t * 100) + "%";
  };
} catch (e) {
  const p = document.createElement("p");
  p.className = "webgl-error";
  p.textContent =
    "3D rendering is unavailable in this browser. Calculations, tables, diagrams and exports still work.";
  $("#chamber").append(p);
}
async function init() {
  try {
    const [health, catalogue, ps] = await Promise.all([
      fetch("/api/health").then((r) => r.json()),
      fetch("/api/catalogue").then((r) => r.json()),
      fetch("/api/processes").then((r) => r.json()),
    ]);
    particles = catalogue.particles;
    processes = ps;
    $("#engine").innerHTML =
      `<span class="live-dot"></span>Rust engine ready<br><small>${health.threads} worker threads · ${particles.length} catalogue entries</small>`;
    setMode("scattering");
    await run();
  } catch {
    $("#status").textContent =
      "Rust engine unavailable. Start cargo run --release -p unnamed-server, then reload.";
    $("#status").classList.add("error");
    $("#engine").textContent = "Engine offline";
  }
}
void init();
