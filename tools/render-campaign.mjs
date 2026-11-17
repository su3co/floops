// A seamless momentum-space loop from actual recorded events, with no dashboard.
// Usage: node tools/render-campaign.mjs RESULT_JSON[.gz] FRAME_DIRECTORY [FRAMES=250]
import fs from 'node:fs';
import path from 'node:path';
import { gunzipSync } from 'node:zlib';
import { Resvg } from '@resvg/resvg-js';

const [input, directory, framesArg = '250'] = process.argv.slice(2);
if (!input || !directory) throw new Error('Provide a campaign receipt and an output directory');
const data = fs.readFileSync(input);
const run = JSON.parse(input.endsWith('.gz') ? gunzipSync(data) : data);
const frames = Number(framesArg);
if (!Number.isInteger(frames) || frames < 2 || !run.snapshots?.at(-1)?.sample?.length) {
  throw new Error('Invalid frame count or campaign receipt');
}
fs.mkdirSync(directory, { recursive: true });

const width = 1280, height = 640, fps = 50, supersampling = 2;
const tau = 2 * Math.PI;
const colors = ['#86dfcd', '#efbc89'];
// Keep directions fixed throughout the loop: changing snapshots caused visible pops.
const events = run.snapshots.at(-1).sample.filter((_, i) => i % 8 === 0);
const maxWeight = Math.max(...events.map(e => e.weight_pb));
// Genuine LaTeX/Computer Modern outlines, so rendering needs no installed fonts.
const labelSvg = fs.readFileSync(new URL('./scattering-label.svg', import.meta.url), 'utf8');
const labelViewBox = labelSvg.match(/viewBox="([^"]+)"/)[1];
const label = labelSvg.slice(labelSvg.indexOf('>', labelSvg.indexOf('<svg')) + 1, labelSvg.lastIndexOf('</svg>'))
  .replaceAll('rgb(0%, 0%, 0%)', '#adbcca');

const smooth = (a, b, x) => {
  const t = Math.max(0, Math.min(1, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};
const coord = n => n.toFixed(3);
const line = (a, b, color, opacity, stroke = 1) =>
  `<path d="M${coord(a[0])} ${coord(a[1])}L${coord(b[0])} ${coord(b[1])}" stroke="${color}" stroke-opacity="${opacity.toFixed(4)}" stroke-width="${stroke}" stroke-linecap="round"/>`;
const dot = (p, r, color, opacity) =>
  `<circle cx="${coord(p[0])}" cy="${coord(p[1])}" r="${r}" fill="${color}" fill-opacity="${opacity.toFixed(4)}"/>`;

for (let frame = 0; frame < frames; frame++) {
  // Excluding the repeated endpoint gives the wrap the same spacing as every frame.
  const phase = frame / frames;
  const yaw = 1.08 + 0.20 * Math.sin(tau * phase);
  const pitch = 0.22 + 0.10 * Math.cos(tau * phase);
  const project = ([x, y, z]) => {
    const a = x * Math.cos(yaw) + z * Math.sin(yaw);
    const b = -x * Math.sin(yaw) + z * Math.cos(yaw);
    const c = y * Math.cos(pitch) - b * Math.sin(pitch);
    const depth = y * Math.sin(pitch) + b * Math.cos(pitch);
    const scale = 230 / (1 - depth * 0.10);
    return [width / 2 + a * scale, 290 - c * scale, depth];
  };
  const origin = project([0, 0, 0]);
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}"><rect width="${width}" height="${height}" fill="#0a101b"/>`;
  // Three quiet great circles establish depth without the old dense wireframe.
  for (let plane = 0; plane < 3; plane++) {
    let prior;
    for (let step = 0; step <= 192; step++) {
      const a = tau * step / 192, c = Math.cos(a), s = Math.sin(a);
      const point = project(plane === 0 ? [c, s, 0] : plane === 1 ? [c, 0, s] : [0, c, s]);
      if (prior) svg += line(prior, point, '#44576d', point[2] > 0 ? 0.30 : 0.13, 0.8);
      prior = point;
    }
  }
  svg += line(project([0, 0, -2.05]), project([0, 0, 2.05]), '#bccbdc', 0.13, 1);

  const tracks = [];
  for (let i = 0; i < events.length; i++) {
    const event = events[i];
    const age = (phase * 2 + i * 0.618033988749895) % 1;
    const envelope = smooth(0, 0.07, age) * (1 - smooth(0.70, 1, age));
    const weight = 0.42 + 0.58 * Math.sqrt(event.weight_pb / maxWeight);
    for (let particle = 2; particle < 4; particle++) {
      const p = event.momenta[particle];
      const direction = [p.px / p.e, p.py / p.e, p.pz / p.e];
      const at = r => project(direction.map(x => x * r));
      const head = at(age * 1.12), color = colors[particle - 2];
      const depth = 0.75 + 0.25 * (head[2] + 1.2) / 2.4;
      const alpha = envelope * weight * depth;
      let trail = line(origin, at(1.12), color, 0.055 * weight, 0.8);
      // Short fading segments form a continuous trail with no large gradient fields.
      for (let segment = 0; segment < 9; segment++) {
        const a = Math.max(0, age * 1.12 - (9 - segment) * 0.024);
        const b = Math.max(0, age * 1.12 - (8 - segment) * 0.024);
        trail += line(at(a), at(b), color, alpha * (segment + 1) / 9, 1.7);
      }
      trail += dot(head, 5.5, color, alpha * 0.07);
      trail += dot(head, 3.1, color, alpha * 0.24);
      trail += dot(head, 1.7, color, alpha);
      tracks.push({ depth: head[2], svg: trail });
    }
  }
  tracks.sort((a, b) => a.depth - b.depth);
  svg += tracks.map(t => t.svg).join('');
  for (const sign of [-1, 1]) for (let i = 0; i < 5; i++) {
    const age = (phase * 2 + i / 5) % 1;
    const alpha = smooth(0, 0.10, age) * (1 - smooth(0.92, 1, age));
    const head = project([0, 0, sign * 2.05 * (1 - age)]);
    const tail = project([0, 0, sign * Math.min(2.05, 2.05 * (1 - age) + 0.09)]);
    svg += line(tail, head, '#dce8f3', alpha * 0.45, 1.5);
    svg += dot(head, 1.7, '#edf5fc', alpha * 0.85);
  }
  svg += dot(origin, 10, '#dce8f3', 0.025) + dot(origin, 5, '#dce8f3', 0.07) + dot(origin, 2.1, '#edf5fc', 0.92);
  svg += `<svg x="550" y="583" width="180" height="28" viewBox="${labelViewBox}">${label}</svg></svg>`;

  const base = path.join(directory, `frame-${String(frame).padStart(4, '0')}`);
  const raster = new Resvg(svg, { fitTo: { mode: 'width', value: width * supersampling } }).render();
  fs.writeFileSync(base + '.png', raster.asPng());
  if ([0, Math.floor(frames / 2), frames - 1].includes(frame)) fs.writeFileSync(base + '.svg', svg);
}
fs.writeFileSync(path.join(directory, 'render.json'), JSON.stringify({
  input, frames, width, height, fps, supersampling, duration_seconds: frames / fps,
  displayed_pairs: events.length, source_snapshot: run.snapshots.length - 1,
  source_events: run.events, source_gpu_seconds: run.gpu_seconds, backend: run.backend,
  math_label: 'LaTeX Computer Modern, outlined SVG',
}, null, 2) + '\n');
console.log(`Rendered ${frames} supersampled frames to ${directory}`);
