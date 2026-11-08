//! The 3D page, as lyre_harp.py held it: browser code, kept byte for
//! byte. lyre_harp.rs fills __DATA__.

pub const RENDER_PAGE: &str = r###"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Lyre-Harp Frame</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Barlow+Semi+Condensed:wght@500;600&family=IBM+Plex+Mono:wght@400;500&display=swap">
<style>
:root{
  --ground:#e9edf0; --panel:#f6f8f9; --ink:#1d2328; --muted:#5b6770; --rule:#cdd5db;
  --accent:#2f6b8a; --stage:#dfe5ea; --focus:#2f6b8a;
  --ply:#d8b98b; --ply-edge:#a9835a; --ply-dark:#c39f6f;
}
@media (prefers-color-scheme: dark){
  :root:not([data-theme="light"]){
    --ground:#14181b; --panel:#1b2024; --ink:#e3e8ec; --muted:#93a0a9; --rule:#2c343a;
    --accent:#7fb3cf; --stage:#101316; --focus:#7fb3cf;
  }
}
:root[data-theme="dark"]{
  --ground:#14181b; --panel:#1b2024; --ink:#e3e8ec; --muted:#93a0a9; --rule:#2c343a;
  --accent:#7fb3cf; --stage:#101316; --focus:#7fb3cf;
}
*{box-sizing:border-box}
body{background:var(--ground);color:var(--ink);font:15px/1.5 "IBM Plex Mono",ui-monospace,Menlo,monospace;}
.wrap{max-width:1180px;margin:0 auto;padding-inline:20px;padding-block:22px 36px;display:grid;gap:18px}
header h1{font:600 clamp(26px,4vw,38px)/1.05 "Barlow Semi Condensed","Arial Narrow",system-ui,sans-serif;letter-spacing:.01em;margin:0;text-wrap:balance}
header p{margin:6px 0 0;color:var(--muted);max-width:62ch;font-size:13.5px}
.grid{display:grid;grid-template-columns:minmax(0,1fr) 300px;gap:18px;align-items:start}
@media (max-width:860px){.grid{grid-template-columns:minmax(0,1fr)}}
.stage{position:relative;background:var(--stage);border:1px solid var(--rule);border-radius:6px;overflow:hidden;aspect-ratio:4/5;max-height:78vh;width:100%;touch-action:none;cursor:grab}
.stage:active{cursor:grabbing}
.stage canvas{display:block;width:100%;height:100%}
.hint{position:absolute;left:12px;bottom:10px;font-size:12px;color:var(--muted);pointer-events:none}
.side{display:grid;gap:14px}
.controls{display:flex;flex-wrap:wrap;gap:8px}
.controls label,.controls button{font:500 13px "IBM Plex Mono",ui-monospace,monospace;color:var(--ink);background:var(--panel);border:1px solid var(--rule);border-radius:4px;padding:7px 10px;display:inline-flex;align-items:center;gap:7px;cursor:pointer}
.controls input{accent-color:var(--accent);margin:0}
.controls :focus-visible{outline:2px solid var(--focus);outline-offset:2px}
.spec{background:var(--panel);border:1px solid var(--rule);border-radius:6px;padding:14px 16px}
.spec h2{font:600 17px/1.2 "Barlow Semi Condensed","Arial Narrow",system-ui,sans-serif;letter-spacing:.04em;text-transform:uppercase;margin:0 0 8px;color:var(--muted)}
.spec dl{margin:0;display:grid;grid-template-columns:auto 1fr;gap:6px 14px;font-size:13px}
.spec dt{color:var(--muted)}
.spec dd{margin:0;text-align:right;font-variant-numeric:tabular-nums}
.spec .note{margin:10px 0 0;font-size:12px;color:var(--muted);line-height:1.45}
.key{display:flex;gap:12px;flex-wrap:wrap;font-size:12px;color:var(--muted)}
.key span{display:inline-flex;align-items:center;gap:6px}
.sw{width:12px;height:12px;border-radius:2px;display:inline-block;border:1px solid var(--rule)}
/* ?embed: framed on the writeup page, which carries the title and the notes itself */
.embed body{background:transparent;overflow:hidden}
.embed .wrap{max-width:none;padding:0;gap:0}
.embed header,.embed .key,.embed .spec{display:none}
.embed .grid{display:flex;flex-direction:column;gap:10px}
.embed .stage{aspect-ratio:auto;height:calc(100vh - 64px);max-height:none}
</style>
<script>
/* Follow the theme the site's pages store, so a framed viewer matches the page around it. */
(function () {
  var r = document.documentElement;
  function apply() { var t = null; try { t = localStorage.getItem('ctg-theme'); } catch (e) {}
    if (t === 'light' || t === 'dark') r.setAttribute('data-theme', t); else r.removeAttribute('data-theme'); }
  apply(); addEventListener('storage', function (e) { if (e.key === 'ctg-theme') apply(); });
  if (/[?&]embed\b/.test(location.search)) r.classList.add('embed');
})();
</script>
</head>
<body>
<div class="wrap">
  <header>
    <h1>Lyre-Harp Frame</h1>
    <p>One closed duct, 30&nbsp;mm deep, in 3&nbsp;mm birch ply. Built from the same geometry as the cut files: two cheeks, 49 wall panels, and laminated blocks at both ends for the hitch and tuning pins.</p>
  </header>
  <div class="grid">
    <div class="stage" id="stage" aria-label="3D view of the lyre-harp frame. Drag to turn; right-drag, shift-drag or two fingers to pan; scroll or pinch to zoom; double-click a part to turn round it.">
      <span class="hint">Drag to turn &middot; right-drag to pan &middot; scroll or pinch to zoom &middot; double-click a part to turn round it</span>
    </div>
    <aside class="side">
      <div class="controls">
        <label for="t-front"><input type="checkbox" id="t-front" checked> Front cheek</label>
        <label for="t-back"><input type="checkbox" id="t-back" checked> Back cheek</label>
        <label for="t-walls"><input type="checkbox" id="t-walls" checked> Walls</label>
        <label for="t-block"><input type="checkbox" id="t-block" checked> Pin blocks</label>
        <button type="button" id="b-front">Front view</button>
        <button type="button" id="b-reset">Reset view</button>
      </div>
      <div class="key">
        <span><i class="sw" style="background:#d8b98b"></i>Cheeks</span>
        <span><i class="sw" style="background:#b98f5e"></i>Outer wall</span>
        <span><i class="sw" style="background:#8f6a44"></i>String-hole wall</span>
        <span><i class="sw" style="background:#c9a574"></i>Pin blocks</span>
      </div>
      <section class="spec">
        <h2>As cut</h2>
        <dl id="spec"></dl>
        <p class="note">Tabs, slots and engraving are left off the model. The front cheek carries the knot, and the back cheek is plain.</p>
      </section>
    </aside>
  </div>
</div>

<script src="https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js"></script>
<script>
const D = __DATA__;
(function(){
  const spec = [
    ['Outside', D.size[0].toFixed(0)+' × '+D.size[1].toFixed(0)+' mm'],
    ['Duct depth', D.bore+' mm'],
    ['Upper duct', D.bore+' × '+D.bore+' mm'],
    ['Ply', D.t+' mm'],
    ['String hole', D.hole_w.toFixed(0)+' mm wide'],
    ['Resonator', D.reso.toFixed(0)+' mm along axis'],
    ['Sound hole', '7-bight knot, r30'],
    ['Knot centre', D.knot_up.toFixed(1)+' mm up (2/3)'],
    ['Hitch block', D.hitch_layers+' × '+D.t+' = '+(D.hitch_layers*D.t)+' mm'],
    ['Hitch pins', D.pins.length+' × Ø'+D.pin[0]+', '+D.pin[1]+' mm apart'],
    ['Tuning block', D.tune_layers+' × '+D.t+' = '+(D.tune_layers*D.t)+' mm'],
    ['Tuning pins', D.tune_pins.length+', in line with the hitch pins'],
    ['Parts', '2 cheeks + '+D.panels.length+' panels + '+D.hitch_layers+' layers'],
  ];
  const dl = document.getElementById('spec');
  for (const [k,v] of spec){
    const dt=document.createElement('dt'); dt.textContent=k;
    const dd=document.createElement('dd'); dd.textContent=v;
    dl.append(dt,dd);
  }

  const stage = document.getElementById('stage');
  if (!window.THREE){ stage.insertAdjacentHTML('beforeend','<p style="padding:20px">The 3D library did not load. Reload the page to try again.</p>'); return; }

  const renderer = new THREE.WebGLRenderer({antialias:true, alpha:true});
  renderer.setPixelRatio(Math.min(window.devicePixelRatio||1, 2));
  stage.prepend(renderer.domElement);
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(32, 1, 1, 5000);

  scene.add(new THREE.HemisphereLight(0xf4f1ea, 0x3a4550, 0.85));
  const key = new THREE.DirectionalLight(0xffffff, 0.75); key.position.set(250, 400, 520); scene.add(key);
  const rim = new THREE.DirectionalLight(0xbcd3e0, 0.35); rim.position.set(-400, -200, -300); scene.add(rim);

  // centre the part on the origin
  const xs = D.rim.map(p=>p[0]), ys = D.rim.map(p=>p[1]);
  const cx = (Math.min(...xs)+Math.max(...xs))/2, cy = (Math.min(...ys)+Math.max(...ys))/2;
  const P = p => new THREE.Vector2(p[0]-cx, p[1]-cy);

  const frame = new THREE.Group(); scene.add(frame);
  const half = D.bore/2, t = D.t;

  function edges(mesh, color, opacity){
    const e = new THREE.LineSegments(new THREE.EdgesGeometry(mesh.geometry, 25),
      new THREE.LineBasicMaterial({color, transparent:true, opacity}));
    mesh.add(e); return e;
  }

  function cheek(withKnot){
    const shape = new THREE.Shape(D.rim.map(P));
    shape.holes.push(new THREE.Path(D.hole.map(P)));
    if (withKnot) for (const loop of D.knot) shape.holes.push(new THREE.Path(loop.map(P)));
    const g = new THREE.ExtrudeGeometry(shape, {depth:t, bevelEnabled:false, curveSegments:1});
    const m = new THREE.Mesh(g, new THREE.MeshStandardMaterial({color:0xd8b98b, roughness:.85, metalness:0}));
    edges(m, 0x6d5233, .55);
    return m;
  }
  const front = cheek(true);  front.position.z = half;
  const back  = cheek(false); back.position.z = -half - t;
  frame.add(front, back);

  // the hitch-pin block: laminations glued up against the front cheek
  const block = new THREE.Group(); frame.add(block);
  function laminate(loop, layers){
    for (let k = 0; k < layers; k++){
      const g = new THREE.ExtrudeGeometry(new THREE.Shape(loop.map(P)),
        {depth:t*0.985, bevelEnabled:false, curveSegments:1});
      const m = new THREE.Mesh(g, new THREE.MeshStandardMaterial({color: k % 2 ? 0xc9a574 : 0xd6b688, roughness:.85}));
      m.position.z = half - (k + 1) * t;
      edges(m, 0x5e4528, .5);
      block.add(m);
    }
  }
  laminate(D.hitch, D.hitch_layers);
  laminate(D.tune, D.tune_layers);
  // hitch pins: through the front cheek into the block, standing proud
  const pins = new THREE.Group(); frame.add(pins);
  const pinMat = new THREE.MeshStandardMaterial({color:0xb8bec4, roughness:.35, metalness:.8});
  for (const q of D.pins.concat(D.tune_pins)){
    const g = new THREE.CylinderGeometry(D.pin[0]/2, D.pin[0]/2, 22, 16);
    const m = new THREE.Mesh(g, pinMat);
    m.rotation.x = Math.PI/2;
    m.position.set(q[0]-cx, q[1]-cy, half + t + 11 - 14);
    pins.add(m);
  }
  const walls = new THREE.Group(); frame.add(walls);
  const matO = new THREE.MeshStandardMaterial({color:0xb98f5e, roughness:.8});
  const matI = new THREE.MeshStandardMaterial({color:0x8f6a44, roughness:.8});
  D.panels.forEach((q,i)=>{
    const d = D.panels_dir[i];
    const g = new THREE.BoxGeometry(q.len, t, D.bore);
    const m = new THREE.Mesh(g, q.wall==='outer'?matO:matI);
    m.position.set(q.x-cx, q.y-cy, 0);
    m.rotation.z = Math.atan2(d.dy, d.dx);
    edges(m, 0x4a3822, .45);
    walls.add(m);
  });

  // view state: the frame turns about piv, a point in its own coordinates
  // that sits at the middle of the view; panning and zooming slide piv
  const home = {yaw:-0.62, pitch:0.38, dist:760, piv:new THREE.Vector3()};
  let yaw=home.yaw, pitch=home.pitch, dist=home.dist;
  const piv = new THREE.Vector3();
  const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  function place(){
    frame.rotation.set(pitch, yaw, 0);
    frame.position.copy(piv).applyEuler(frame.rotation).negate();
    camera.position.set(0, 0, dist); camera.lookAt(0,0,0);
  }
  function size(){
    const r = stage.getBoundingClientRect();
    renderer.setSize(r.width, r.height, false);
    camera.aspect = r.width/Math.max(r.height,1); camera.updateProjectionMatrix();
    render();
  }
  function render(){ place(); renderer.render(scene, camera); }
  new ResizeObserver(size).observe(stage);

  const ray = new THREE.Raycaster(), ndc = new THREE.Vector2();
  const shown = o=>{ for (; o; o=o.parent) if (!o.visible) return false; return true; };
  function under(cx, cy, partOnly){   // the part under the pointer, else where its ray meets the plane through the middle
    const b = renderer.domElement.getBoundingClientRect();
    ndc.set((cx-b.left)/b.width*2-1, -(cy-b.top)/b.height*2+1);
    ray.setFromCamera(ndc, camera);
    const hit = ray.intersectObject(frame, true).find(h=>h.object.isMesh && shown(h.object));
    if (hit || partOnly) return hit ? hit.point : null;
    return ray.ray.intersectPlane(new THREE.Plane(new THREE.Vector3(0,0,1), 0), new THREE.Vector3());
  }
  const slide = v=>piv.add(v.applyQuaternion(frame.quaternion.clone().invert()));   // the model moves by -v on screen
  function pan(dx, dy){   // the model follows the pointer
    const k = 2*dist*Math.tan(camera.fov*Math.PI/360)/Math.max(renderer.domElement.clientHeight,1);
    slide(new THREE.Vector3(-dx*k, dy*k, 0));
  }
  function zoom(s, cx, cy){   // scale the view about the point under the pointer, which stays where it is on screen
    const nd = Math.max(30, Math.min(1800, dist*s)), p = under(cx, cy);
    if (p) slide(p.multiplyScalar(1-nd/dist));
    dist = nd;
  }
  const pts = new Map(); let pinch0 = 0, mid0 = null, panning = false;
  const two = ()=>{ const [a,b]=[...pts.values()]; return {h:Math.hypot(a.x-b.x,a.y-b.y), x:(a.x+b.x)/2, y:(a.y+b.y)/2}; };
  stage.addEventListener('contextmenu', e=>e.preventDefault());
  stage.addEventListener('pointerdown', e=>{ stage.setPointerCapture(e.pointerId); pts.set(e.pointerId,{x:e.clientX,y:e.clientY});
    panning = e.button===1 || e.button===2 || e.shiftKey;
    if (pts.size===2){ mid0=two(); pinch0=mid0.h; } });
  stage.addEventListener('pointermove', e=>{
    if (!pts.has(e.pointerId)) return;
    const prev = pts.get(e.pointerId); pts.set(e.pointerId,{x:e.clientX,y:e.clientY});
    const dx = e.clientX-prev.x, dy = e.clientY-prev.y;
    if (pts.size===1){
      if (panning) pan(dx, dy);
      else { yaw += dx*0.008; pitch = Math.max(-1.45, Math.min(1.45, pitch + dy*0.008)); }
    } else if (pts.size===2 && pinch0){   // two fingers: pinch to zoom, move together to pan
      const m=two(); pan(m.x-mid0.x, m.y-mid0.y); render();
      if (m.h) zoom(pinch0/m.h, m.x, m.y); mid0=m; pinch0=m.h;
    }
    render();
  });
  const up = e=>{ pts.delete(e.pointerId); if (pts.size<2) pinch0=0; };
  stage.addEventListener('pointerup', up); stage.addEventListener('pointercancel', up);
  stage.addEventListener('wheel', e=>{ e.preventDefault(); zoom(1+Math.sign(e.deltaY)*0.08, e.clientX, e.clientY); render(); }, {passive:false});

  function goTo(target){
    const t = target.piv || home.piv;
    if (reduce){ ({yaw,pitch,dist}=target); piv.copy(t); render(); return; }
    const s={yaw,pitch,dist,piv:piv.clone()}, t0=performance.now();
    (function step(now){
      const k=Math.min(1,(now-t0)/450), e=k<.5?2*k*k:1-Math.pow(-2*k+2,2)/2;
      yaw=s.yaw+(target.yaw-s.yaw)*e; pitch=s.pitch+(target.pitch-s.pitch)*e; dist=s.dist+(target.dist-s.dist)*e;
      piv.lerpVectors(s.piv, t, e);
      render(); if(k<1) requestAnimationFrame(step);
    })(t0);
  }
  stage.addEventListener('dblclick', e=>{   // turn round the part under the pointer from now on
    const p = under(e.clientX, e.clientY, true);
    if (p) goTo({yaw, pitch, dist, piv:frame.worldToLocal(p)});
  });
  document.getElementById('b-reset').addEventListener('click', ()=>goTo(home));
  document.getElementById('b-front').addEventListener('click', ()=>goTo({yaw:0,pitch:0,dist:720}));
  const bind=(id,obj)=>document.getElementById(id).addEventListener('change',e=>{obj.visible=e.target.checked; render();});
  bind('t-front',front); bind('t-back',back); bind('t-walls',walls); bind('t-block',block); bind('t-block',pins);
  size();
})();
</script>
</body>
</html>
"###;
