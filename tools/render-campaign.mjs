// Offline, data-driven momentum-space animation. No generated imagery or physics.
// Usage: node tools/render-campaign.mjs RESULT_JSON FRAME_DIRECTORY [FRAME_COUNT]
import fs from 'node:fs';
import path from 'node:path';
import { gunzipSync } from 'node:zlib';
import { Resvg } from '@resvg/resvg-js';

const [input, directory, framesArg = '180'] = process.argv.slice(2);
if (!input || !directory) throw new Error('Provide result.json and an output directory');
const data = fs.readFileSync(input);
const run = JSON.parse(input.endsWith('.gz') ? gunzipSync(data).toString('utf8') : data.toString('utf8'));
const frames = Number(framesArg);
if (!Number.isInteger(frames) || frames < 2 || !run.snapshots?.length) throw new Error('Invalid frames or receipt');
fs.mkdirSync(directory, { recursive: true });
const width = 960, height = 600;
const cyan = '#67e8d6', amber = '#ffb56b', ink = '#eef5ff', muted = '#91a8c1';
const esc = s => String(s).replace(/[<>&"]/g, c => ({ '<': '&lt;', '>': '&gt;', '&': '&amp;', '"': '&quot;' })[c]);
const text = (x,y,s,size=16,fill=ink,weight=400) => `<text x="${x}" y="${y}" font-size="${size}" fill="${fill}" font-weight="${weight}">${esc(s)}</text>`;
const line = (a,b,color,opacity=1,w=1) => `<line x1="${a[0].toFixed(2)}" y1="${a[1].toFixed(2)}" x2="${b[0].toFixed(2)}" y2="${b[1].toFixed(2)}" stroke="${color}" stroke-opacity="${opacity.toFixed(3)}" stroke-width="${w}" stroke-linecap="round"/>`;
const dot = (p,r,color,opacity=1) => `<circle cx="${p[0].toFixed(2)}" cy="${p[1].toFixed(2)}" r="${r}" fill="${color}" fill-opacity="${opacity.toFixed(3)}"/>`;
const short = n => n >= 1e12 ? `${(n/1e12).toFixed(2)} trillion` : n >= 1e9 ? `${(n/1e9).toFixed(1)} billion` : `${(n/1e6).toFixed(1)} million`;

for (let f=0; f<frames; f++) {
  const phase=f/(frames-1), elapsed=run.gpu_seconds*Math.min(1,phase/0.9);
  const snapshot=[...run.snapshots].reverse().find(s=>s.gpu_seconds<=elapsed) ?? run.snapshots[0];
  const yaw=0.65+phase*Math.PI*2, pitch=0.27;
  const project=([x,y,z])=>{
    const a=x*Math.cos(yaw)+z*Math.sin(yaw), b=-x*Math.sin(yaw)+z*Math.cos(yaw);
    const c=y*Math.cos(pitch)-b*Math.sin(pitch), d=y*Math.sin(pitch)+b*Math.cos(pitch);
    const scale=159*3.6/(3.6-d*0.22);
    return [310+a*scale,314-c*scale,d];
  };
  const origin=project([0,0,0]);
  let svg=`<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}"><defs><linearGradient id="bg" x2="1" y2="1"><stop stop-color="#07101e"/><stop offset="1" stop-color="#10273b"/></linearGradient><radialGradient id="halo"><stop stop-color="#64e5d5" stop-opacity=".11"/><stop offset="1" stop-color="#64e5d5" stop-opacity="0"/></radialGradient></defs><rect width="960" height="600" fill="url(#bg)"/><g font-family="DejaVu Sans,Arial,sans-serif">`;
  svg+=text(36,44,'floops',30,ink,650)+text(771,40,'CUDA  /  FP64',15,cyan,600);
  svg+=text(36,89,'Gluon scattering',32,ink,500);
  svg+=text(36,117,`g g → g g     •     ${run.request.energy/1000} TeV     •     αs = ${run.request.alpha}`,16,muted);
  svg+='<circle cx="310" cy="314" r="220" fill="url(#halo)"/>';
  // Spherical momentum reference grid; this is not a detector geometry.
  for(let ring=0;ring<12;ring++){
    let prior;
    for(let k=0;k<=80;k++){
      const a=k/80*Math.PI*2, b=ring/12*Math.PI;
      const p=project([Math.cos(a)*Math.cos(b),Math.sin(a),Math.cos(a)*Math.sin(b)]);
      if(prior)svg+=line(prior,p,'#7491ae',p[2]>0?.20:.07,.65);
      prior=p;
    }
  }
  for(const latitude of [-Math.PI/3,-Math.PI/6,0,Math.PI/6,Math.PI/3]){
    let prior;
    for(let k=0;k<=100;k++){
      const a=k/100*Math.PI*2, r=Math.cos(latitude);
      const p=project([r*Math.cos(a),Math.sin(latitude),r*Math.sin(a)]);
      if(prior)svg+=line(prior,p,'#7491ae',p[2]>0?.20:.07,.65);
      prior=p;
    }
  }
  svg+=line(project([0,0,-1.78]),project([0,0,1.78]),'#8b9fc6',.36,1);
  svg+=text(project([0,0,1.9])[0]-20,project([0,0,1.9])[1]-8,'beam',13,muted);
  const events=snapshot.sample.filter((_,i)=>i%4===0);
  const maxWeight=Math.max(...events.map(e=>e.weight_pb));
  const primitives=[];
  for(let i=0;i<events.length;i++){
    const event=events[i], clock=(phase*5+i*0.61803398875)%1;
    const reach=Math.max(0,(clock-.15)/.85), alpha=(.12+.58*Math.sqrt(event.weight_pb/maxWeight))*(1-.35*reach);
    for(let k=2;k<4;k++){
      const m=event.momenta[k], v=[m.px/m.e,m.py/m.e,m.pz/m.e];
      const tip=project(v.map(x=>x*reach*1.13)), tail=project(v.map(x=>x*Math.max(0,reach-.32)*1.13));
      const color=k===2?cyan:amber;
      primitives.push({z:tip[2],s:line(origin,tip,color,alpha*.28,.75)+line(tail,tip,color,alpha,1.25)+dot(tip,1.65,color,alpha)});
    }
  }
  primitives.sort((a,b)=>a.z-b.z);
  svg+=primitives.map(x=>x.s).join('');
  for(const sign of [-1,1])for(let j=0;j<7;j++){
    const t=(phase*5+j/7)%1,p=project([0,0,sign*(1.78*(1-t))]);
    svg+=dot(p,1.6,'#d6e8ff',.8);
  }
  svg+=dot(origin,6,cyan,.08)+dot(origin,2.2,'#ffffff',.9);
  svg+=text(36,506,`Momentum space · ${events.length} sampled pairs`,14,muted);
  svg+=text(36,527,'Directions come from the recorded four-momenta.',13,muted);
  svg+='<path d="M608 140V525" stroke="#294157" stroke-width="1"/>';
  svg+=text(640,161,'WEIGHTED EVENTS',12,muted,600)+text(640,198,short(snapshot.events),30,ink,600);
  svg+=text(640,229,`${(snapshot.gpu_seconds/60).toFixed(1)} min GPU time`,17,cyan);
  svg+=text(640,255,`${(run.resident_event_bytes/2**30).toFixed(1)} GiB resident event data`,15,muted);
  svg+=text(640,301,'Angular cross section',17,ink,500);
  const left=644,top=326,cw=273,ch=117,max=Math.max(...run.reference_histogram)*1.1;
  for(let i=0;i<60;i++){
    const bh=snapshot.histogram[i]/max*ch;
    svg+=`<rect x="${left+i*cw/60}" y="${top+ch-bh}" width="${cw/60-.8}" height="${bh}" fill="${cyan}" fill-opacity=".72"/>`;
  }
  const theory=run.reference_histogram.map((y,i)=>`${left+(i+.5)*cw/60},${top+ch-y/max*ch}`).join(' ');
  svg+=`<polyline points="${theory}" fill="none" stroke="${amber}" stroke-width="1.7"/>`;
  svg+=line([left,top+ch],[left+cw,top+ch],'#69839d',.65);
  svg+=text(left,top+ch+21,`−${run.request.cos_cut}`,12,muted)+text(left+cw/2-19,top+ch+21,'cos θ',12,muted)+text(left+cw-24,top+ch+21,run.request.cos_cut,12,muted);
  svg+=text(640,491,`σ = ${snapshot.cross_section_pb.toFixed(4)} pb`,19,ink,500);
  svg+=text(640,515,`MC ± ${snapshot.standard_error_pb.toFixed(4)} pb`,13,cyan)+text(796,515,'― reference',13,amber);
  svg+='<rect x="36" y="549" width="888" height="3" rx="1.5" fill="#284158"/>';
  svg+=`<rect x="36" y="549" width="${888*snapshot.gpu_seconds/run.gpu_seconds}" height="3" rx="1.5" fill="${cyan}"/>`;
  svg+=text(36,578,'Tree-level partons · no showers, hadronization or detector response',12,muted);
  svg+=text(754,578,'Recorded GPU campaign',12,muted);
  svg+='</g></svg>';
  const base=path.join(directory,`frame-${String(f).padStart(4,'0')}`);
  const image=new Resvg(svg,{font:{loadSystemFonts:true,defaultFontFamily:'DejaVu Sans'}}).render().asPng();
  fs.writeFileSync(base+'.png',image);
  if(f===0||f===Math.floor(frames/2)||f===frames-1)fs.writeFileSync(base+'.svg',svg);
}
fs.writeFileSync(path.join(directory,'render.json'),JSON.stringify({input,frames,width,height,fps:15,source_events:run.events,source_gpu_seconds:run.gpu_seconds,backend:run.backend},null,2)+'\n');
console.log(`Rendered ${frames} data-derived frames to ${directory}`);
