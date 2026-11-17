import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
export type Momentum = { e: number; px: number; py: number; pz: number };
export type Track = {
  a: THREE.Vector3;
  b: THREE.Vector3;
  color: number;
  start: number;
  end: number;
};
export const speciesColor = (id: number) =>
  Math.abs(id) <= 6 || Math.abs(id) > 100
    ? 0xffad91
    : id === 21
      ? 0xada0f2
      : Math.abs(id) === 12 || Math.abs(id) === 14 || Math.abs(id) === 16
        ? 0xa4bac9
        : id === 22
          ? 0xf3da8e
          : 0x72dbed;
export class Chamber {
  scene = new THREE.Scene();
  camera = new THREE.PerspectiveCamera(38, 1, 0.1, 100);
  renderer: THREE.WebGLRenderer;
  controls: OrbitControls;
  content = new THREE.Group();
  tracks: Track[] = [];
  dots?: THREE.InstancedMesh;
  field?: THREE.InstancedMesh;
  frame = 0;
  frames: number[][] = [];
  length = 4;
  playing = false;
  time = 0.65;
  mode = "events";
  last = 0;
  onTime: (t: number) => void = () => {};
  dummy = new THREE.Object3D();
  color = new THREE.Color();
  constructor(public el: HTMLElement) {
    this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
    this.renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
    this.renderer.setClearColor(0x142a41);
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    el.prepend(this.renderer.domElement);
    this.renderer.domElement.setAttribute(
      "aria-label",
      "Interactive 3D event view. Drag to orbit, scroll to zoom; numerical results are below.",
    );
    this.renderer.domElement.setAttribute("tabindex", "0");
    this.camera.position.set(13, 8, 15);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.minDistance = 5;
    this.controls.maxDistance = 40;
    this.controls.target.set(0, 0, 0);
    this.scene.add(new THREE.AmbientLight(0xffffff, 2));
    const light = new THREE.DirectionalLight(0xffffff, 3);
    light.position.set(5, 8, 9);
    this.scene.add(light);
    this.scene.add(this.content);
    new ResizeObserver(() => {
      const { width, height } = el.getBoundingClientRect();
      this.renderer.setSize(width, height);
      this.camera.aspect = width / height;
      this.camera.updateProjectionMatrix();
    }).observe(el);
    this.renderer.domElement.addEventListener("keydown", (e) => {
      const sign = e.key === "ArrowLeft" ? -1 : e.key === "ArrowRight" ? 1 : 0;
      if (sign) {
        this.camera.position.applyAxisAngle(
          new THREE.Vector3(0, 1, 0),
          sign * 0.1,
        );
        e.preventDefault();
      }
      if (e.key === "Home") this.resetCamera();
    });
    this.renderer.setAnimationLoop((t) => this.animate(t));
  }
  resetCamera() {
    this.camera.position.set(13, 8, 15);
    this.controls.target.set(0, 0, 0);
    this.controls.update();
  }
  clear() {
    for (const o of [...this.content.children]) {
      this.content.remove(o);
      o.traverse((x) => {
        if (x instanceof THREE.Mesh || x instanceof THREE.Line) {
          x.geometry.dispose();
          const mats = Array.isArray(x.material) ? x.material : [x.material];
          mats.forEach((m) => m.dispose());
        }
      });
    }
    this.dots = undefined;
    this.field = undefined;
    this.tracks = [];
    this.frames = [];
  }
  line(a: THREE.Vector3, b: THREE.Vector3, color: number, opacity = 0.4) {
    const geo = new THREE.BufferGeometry().setFromPoints([a, b]);
    const mesh = new THREE.Line(
      geo,
      new THREE.LineBasicMaterial({ color, transparent: true, opacity }),
    );
    this.content.add(mesh);
  }
  instrument() {
    const grid = new THREE.GridHelper(18, 18, 0x37536b, 0x253e56);
    grid.position.y = -4;
    this.content.add(grid);
    for (const x of [-4, -2, 0, 2, 4]) {
      const pts = [];
      for (let k = 0; k <= 80; k++) {
        const a = (k / 80) * Math.PI * 2;
        pts.push(new THREE.Vector3(x, 3.2 * Math.cos(a), 3.2 * Math.sin(a)));
      }
      this.content.add(
        new THREE.Line(
          new THREE.BufferGeometry().setFromPoints(pts),
          new THREE.LineBasicMaterial({
            color: 0x41627b,
            transparent: true,
            opacity: 0.2,
          }),
        ),
      );
    }
    this.line(
      new THREE.Vector3(-8, 0, 0),
      new THREE.Vector3(8, 0, 0),
      0x7794ae,
      0.4,
    );
  }
  vector(p: Momentum) {
    return new THREE.Vector3(p.pz, p.py, p.px).normalize();
  }
  setScatter(
    events: { momenta: Momentum[]; species: number[] }[],
    selected = 0,
  ) {
    this.clear();
    this.mode = "events";
    this.instrument();
    events.forEach((ev, index) => {
      for (let i = 0; i < 4; i++) {
        const v = this.vector(ev.momenta[i]);
        const incoming = i < 2;
        const a = incoming ? v.clone().multiplyScalar(-7) : new THREE.Vector3();
        const b = incoming
          ? new THREE.Vector3()
          : v.clone().multiplyScalar(6.5);
        const color = speciesColor(ev.species[i]);
        this.line(a, b, color, index === selected ? 0.95 : 0.055);
        this.tracks.push({
          a,
          b,
          color,
          start: incoming ? 0 : 0.25,
          end: incoming ? 0.25 : 1,
        });
      }
    });
    this.makeDots();
    this.time = 0.7;
  }
  setDecay(
    nodes: {
      pdg: number;
      parent: number | null;
      momentum: Momentum;
      birth: number[];
      end: number[];
      status: string;
    }[],
  ) {
    this.clear();
    this.mode = "events";
    this.instrument();
    const depth = (index: number): number =>
      nodes[index].parent === null ? 0 : 1 + depth(nodes[index].parent!);
    const positions: THREE.Vector3[] = [];
    nodes.forEach((n, i) => {
      const d = depth(i);
      const a =
        n.parent === null
          ? new THREE.Vector3(-4, 0, 0)
          : positions[n.parent].clone();
      const v = this.vector(n.momentum);
      if (v.lengthSq() === 0) v.set(1, 0, 0);
      const b = a
        .clone()
        .add(v.multiplyScalar(n.status === "decayed" ? 1.5 : 3.2));
      positions.push(b);
      const color = speciesColor(n.pdg);
      this.line(a, b, color, 0.85);
      this.tracks.push({
        a,
        b,
        color,
        start: d * 0.14,
        end: Math.min(1, (d + 1) * 0.14 + 0.15),
      });
    });
    this.makeDots();
    this.time = 0.75;
  }
  setAmplitude(momenta: Momentum[]) {
    const e = momenta.map((p) =>
      p.e < 0 ? { e: -p.e, px: -p.px, py: -p.py, pz: -p.pz } : p,
    );
    this.clear();
    this.mode = "events";
    this.instrument();
    momenta.forEach((p, i) => {
      const v = this.vector(e[i]);
      const incoming = p.e < 0;
      const a = incoming ? v.clone().multiplyScalar(-7) : new THREE.Vector3();
      const b = incoming ? new THREE.Vector3() : v.clone().multiplyScalar(6.5);
      this.line(a, b, 0xada0f2, 0.9);
      this.tracks.push({
        a,
        b,
        color: 0xada0f2,
        start: incoming ? 0 : 0.25,
        end: incoming ? 0.25 : 1,
      });
    });
    this.makeDots();
  }
  makeDots() {
    this.dots = new THREE.InstancedMesh(
      new THREE.SphereGeometry(0.055, 8, 6),
      new THREE.MeshBasicMaterial(),
      this.tracks.length,
    );
    this.tracks.forEach((t, i) =>
      this.dots!.setColorAt(i, new THREE.Color(t.color)),
    );
    this.content.add(this.dots);
  }
  setLattice(frames: number[][], length: number) {
    this.clear();
    this.mode = "lattice";
    this.frames = frames;
    this.length = length;
    this.time = 1;
    const count = length ** 3;
    this.field = new THREE.InstancedMesh(
      new THREE.SphereGeometry(0.14, 12, 8),
      new THREE.MeshStandardMaterial({ roughness: 0.4 }),
      count,
    );
    this.content.add(this.field);
    const step = 7 / (length - 1);
    const pos = (x: number, y: number, z: number) =>
      new THREE.Vector3(
        (x - (length - 1) / 2) * step,
        (y - (length - 1) / 2) * step,
        (z - (length - 1) / 2) * step,
      );
    for (let z = 0; z < length; z++)
      for (let y = 0; y < length; y++)
        for (let x = 0; x < length; x++) {
          const a = pos(x, y, z);
          if (x + 1 < length) this.line(a, pos(x + 1, y, z), 0x5c7790, 0.32);
          if (y + 1 < length) this.line(a, pos(x, y + 1, z), 0x5c7790, 0.32);
          if (z + 1 < length) this.line(a, pos(x, y, z + 1), 0x5c7790, 0.32);
        }
    this.updateField();
  }
  updateField() {
    if (!this.field || !this.frames.length) return;
    this.frame = Math.min(
      this.frames.length - 1,
      Math.floor(this.time * this.frames.length),
    );
    const values = this.frames[this.frame];
    const l = this.length;
    values.forEach((v, i) => {
      const x = i % l,
        y = Math.floor(i / l) % l,
        z = Math.floor(i / l / l);
      this.dummy.position.set(
        ((x - (l - 1) / 2) * 7) / (l - 1),
        ((y - (l - 1) / 2) * 7) / (l - 1),
        ((z - (l - 1) / 2) * 7) / (l - 1),
      );
      this.dummy.scale.setScalar(1);
      this.dummy.updateMatrix();
      this.field!.setMatrixAt(i, this.dummy.matrix);
      const t = Math.max(0, Math.min(1, (v + 0.5) / 1.5));
      this.color.set(0xada0f2).lerp(new THREE.Color(0x72dbed), t);
      this.field!.setColorAt(i, this.color);
    });
    this.field.instanceMatrix.needsUpdate = true;
    if (this.field.instanceColor) this.field.instanceColor.needsUpdate = true;
  }
  animate(now: number) {
    const dt = Math.min(0.05, (now - this.last) / 1000);
    this.last = now;
    if (this.playing && !document.hidden) {
      this.time = (this.time + dt * 0.16) % 1;
      this.onTime(this.time);
    }
    if (this.mode === "lattice") this.updateField();
    if (this.dots) {
      this.tracks.forEach((t, i) => {
        const p = Math.max(
          0,
          Math.min(1, (this.time - t.start) / (t.end - t.start)),
        );
        this.dummy.position.copy(t.a).lerp(t.b, p);
        this.dummy.scale.setScalar(this.time < t.start ? 0 : 1);
        this.dummy.updateMatrix();
        this.dots!.setMatrixAt(i, this.dummy.matrix);
      });
      this.dots.instanceMatrix.needsUpdate = true;
    }
    this.controls.update();
    this.renderer.render(this.scene, this.camera);
  }
}
