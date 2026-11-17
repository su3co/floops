// Validate and archive a completed campaign without copying its large event buffer.
// Usage: node tools/summarize-campaign.mjs RESULT_JSON OUTPUT_DIRECTORY
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { gzipSync } from 'node:zlib';

const [input, directory] = process.argv.slice(2);
if (!input || !directory) throw new Error('Provide result.json and an output directory');
const raw = fs.readFileSync(input), run = JSON.parse(raw);
assert.equal(run.state, 'complete');
assert(run.gpu_seconds >= run.requested_gpu_seconds);
assert.equal(run.events, run.batches * run.resident_events);
assert.equal(run.resident_event_bytes, 64 * run.resident_events);
const final = run.snapshots.at(-1);
assert.equal(final.events, run.events);
const z = (final.cross_section_pb - run.reference_cross_section_pb) / final.standard_error_pb;
const binZ = final.histogram.map((v,i) => (v-run.reference_histogram[i])/final.histogram_error[i]);
assert(Math.abs(z) < 8 && binZ.every(v => Math.abs(v) < 8), 'Reference outside 8-sigma diagnostic bound');
let massShell = 0, conservation = 0;
for (const snapshot of run.snapshots) for (const event of snapshot.sample) {
  for (const p of event.momenta) {
    massShell = Math.max(massShell, Math.abs(p.e*p.e-p.px*p.px-p.py*p.py-p.pz*p.pz)/(p.e*p.e));
  }
  for (const field of ['e','px','py','pz']) {
    const p = event.momenta;
    conservation = Math.max(conservation,Math.abs(p[0][field]+p[1][field]-p[2][field]-p[3][field])/run.request.energy);
  }
}
assert(massShell < 1e-12 && conservation < 1e-12);
const summary = {
  state:run.state, backend:run.backend, device:run.device, revision:run.revision,
  started_unix_s:run.started_unix_s, request:run.request, events:run.events, batches:run.batches,
  gpu_seconds:run.gpu_seconds, wall_seconds:run.wall_seconds,
  resident_events:run.resident_events, resident_event_gib:run.resident_event_bytes/2**30,
  max_memory_used_mib:Math.max(...run.telemetry.map(t=>t.memory_used_mib)),
  memory_total_mib:final.telemetry.memory_total_mib,
  max_temperature_c:Math.max(...run.telemetry.map(t=>t.temperature_c)),
  median_utilization_percent:run.telemetry.map(t=>t.utilization_percent).sort((a,b)=>a-b)[Math.floor(run.telemetry.length/2)],
  cross_section_pb:final.cross_section_pb, standard_error_pb:final.standard_error_pb,
  reference_cross_section_pb:run.reference_cross_section_pb, reference_z_score:z,
  maximum_absolute_bin_z_score:Math.max(...binZ.map(Math.abs)),
  sample_max_relative_mass_shell_residual:massShell, sample_max_relative_conservation_residual:conservation,
  validation_max_relative_error:run.validation_max_relative_error,
  snapshots:run.snapshots.length, receipt_sha256:createHash('sha256').update(raw).digest('hex'), note:run.note,
};
fs.mkdirSync(directory,{recursive:true});
fs.writeFileSync(path.join(directory,'summary.json'),JSON.stringify(summary,null,2)+'\n');
fs.writeFileSync(path.join(directory,'result.json.gz'),gzipSync(raw,{level:9}));
console.log(JSON.stringify(summary,null,2));
