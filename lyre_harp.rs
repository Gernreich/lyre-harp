//! A lyre-harp frame as one closed duct whose width changes round the loop.
//! Rust since 2026-11-07 (lyre_harp.py until then; the move changed no number
//! in any file).
//!
//!     target/release/lyre_harp --no-write
//!     target/release/lyre_harp --out=DIR/lyre-harp-....svg
//!     target/release/lyre_harp --drawing=DIR/lyre-harp-drawing.svg
//!     target/release/lyre_harp --render=DIR/lyre-harp-....html
//!
//! ribbon_bore builds every duct as two walls offset a fixed bore either side
//! of ONE centreline, so its section is the same all the way round. This one
//! is not. The walls are two independent closed outlines, drawn from the
//! author's sketch as redrawn and approved on 2026-09-16:
//!
//!   * the OUTER outline, a stadium 400mm long -- a half-circle at each end
//!     and two parallel sides;
//!   * the HOLE, where the strings go: a half-circle concentric with the
//!     outer one, parallel sides, and a bottom of two tangent corners with a
//!     small bump rising into the hole between them.
//!
//! Between them runs the air. Over the arch and down the parallel sides the
//! two walls stand a bore apart, so the duct there is BORE x BORE, 30 x 30mm.
//! Below the hole the walls part: the duct is still BORE deep, cheek to cheek,
//! but as wide as the gap between the hole's bottom and the outer bottom --
//! the resonator, about 200mm across.
//!
//! Everything that is a part -- the panel with its teeth, the mortice, the
//! engraved number, the packing, the sheet and its colours, the ring cheek cut
//! as two contours -- is ribbon_bore's own (swept-curve, cloned beside this
//! one), used and not copied. What is new is only what a single centreline
//! could not describe: the two outlines, which way is out from each, and the
//! checks that assumed a constant bore.
use lasermade_tools::py::{self, g, matan2 as atan2, mcos as cos, msin as sin, mtan as tan};
use lasermade_tools::re;
use lasermade_tools::turn3d::{Num::{F, I}, J};
use ribbon_bore::{
    basename, bbox, contours, fmax, fmin, fsum, in_poly, inside, label, offset, pack, path, pt_seg, pymod, seg_gap,
    seglen, splitext, Cfg, Ink, Item, Part, Written, BED_H, BED_W, CUT, INNER, MARGIN_S, MARK, P, R,
};
use std::f64::consts::PI;
use std::rc::Rc;

#[path = "lyre_harp_page.rs"]
mod lyre_harp_page;

const LENGTH: f64 = 400.0; // outer outline, end to end
// KERF, the full width the laser takes out, for THIS instrument only. 0.13 is
// the MEASURED beam, ribbon_bore's own figure from 2026-09-09. It is not a fit
// knob and nothing here may move it to chase a joint.
//
// FIT is the fit knob, and it is the whole history of this instrument in one
// number. The test came off the bed with LOOSE FINGER JOINTS, so the kerf was
// raised to 0.17, then to 0.20 (2026-09-20, both on the author's instruction)
// -- which tightened the joints because an overstated kerf draws tabs fat and
// mortices thin, but it also drew every OTHER line in the design 0.07 out,
// outlines oversize and holes undersize, the band, the duct and the knot cuts
// with them. So the kerf goes back to the truth and the interference moves to
// the two settings that exist for it, leaving the joints exactly where 0.20
// put them. FIT is the kerf the JOINTS are cut as if, and nothing else reads it.
//
// 0.20 -> 0.25 the same day, on the author's instruction: they are after a
// FRICTION FIT, one that needs a rubber mallet to close, and are walking the
// test piece up to it. That is +0.19mm of interference along the tooth and
// +0.12 across the ply, against +0.09 and +0.07 at 0.20. Use --fit= to try a
// value; move this default when one is to be cut, so the sheets on disk and
// the number in the source never disagree about what is on the bench.
const KERF: f64 = 0.13;
const FIT: f64 = 0.25;
// Why two settings, and not one. Write D = FIT - KERF. Along the tooth BOTH
// sides of the joint are drawn, so overstating the kerf fattens the tab by D
// and thins the notch by D: 2D of interference. Across the ply only the notch
// is drawn -- the other half of that joint is the plywood, which no kerf can
// grow -- so the same D buys D. One lever cannot pay 2:1 and 1:1 at once.
// slot_tighten comes off the notch in both directions, so it settles the
// depth; the width is then short by D, and half of that comes off the
// clearance a side. It takes the play NEGATIVE, at -0.01, and that is the
// honest reading and not a trick: there is no clearance left in this joint,
// there is interference, and a negative clearance per side is what that
// sentence means in a number. The clearance this reduces is ribbon_bore's own
// play_unmeasured, read from its defaults rather than written out again.
const BORE: f64 = 30.0; // duct depth everywhere; duct width over the arch and sides
// The approved drawing, as proportions. Width against length is the drawing's
// 391 : 831. The hole's bottom is scaled on the hole's own half-width, not on
// the length: the band is a fixed 36mm, so the hole does not shrink in step
// with the outline, and scaling its corners on the length left them 10mm
// apart with no room for the bump between them.
const WIDTH_OF_LENGTH: f64 = 391.0 / 830.9;
const HOLE_STRAIGHT: f64 = 175.3 / 159.5;
const HOLE_CORNER: f64 = 98.9 / 159.5;
const HOLE_BUMP: f64 = 10.3 / 159.5;
const FACET: f64 = 15.0; // the most an arc turns per panel
const MIN_PANEL: f64 = 14.0; // but no finer than this along an arc: a tooth needs 10
const NARROW: bool = true;
// The sound hole: a knotwork rosette from the knotwork-soundholes repository,
// used as its generator wrote it. Its centre sits on the long axis, KNOT_UP of
// the way from the bottom of the resonator's air to the top of it -- the outer
// wall's face at the far end, and the hole wall's face at the bump. It goes in
// ONE cheek, the front, so the two cheeks stop being one part cut twice.
const KNOT: &str = "LaserMadeMusic/GIT/knotwork-soundholes/2-lead_7-bight_knot_radius30mm.svg";
const KNOT_UP: f64 = 2.0 / 3.0;
// The hitch-pin block: the strings' loop ends go over pins driven through the
// bottom wall, and 3mm of ply will not hold a pin under string tension, so a
// block is laminated inside the bottom end for the pins to bite into. Each
// layer is one flat part; HITCH_LAYERS of them stack HITCH_LAYERS x THICK
// deep. In plan the block sits against the outer wall's face round the bottom
// end, HITCH_WIDTH wide from that face towards the middle, and runs
// HITCH_FACETS facets' worth along it, centred on the long axis.
const HITCH_FACETS: usize = 8;
const HITCH_WIDTH: f64 = 30.0;
const HITCH_LAYERS: usize = 6;
// The hitch pins, one per string, driven through the front cheek into the
// block. The strings run parallel to the long axis, so the pins stand
// PIN_SPACING apart across it, centred on it. Each sits half the block's width
// in from the wall, so the row follows the curve of the bottom end. They are
// ENGRAVED on the front cheek -- a PIN_DIA circle and a cross through it -- as
// drilling marks, not cut: a hole cut in the cheek alone would not guide a
// drill on into the block.
const PINS: usize = 7;
const PIN_SPACING: f64 = 16.0;
const PIN_DIA: f64 = 2.0;
// The tuning-pin block, the hitch block's opposite number at the arch. It
// fills the duct there -- BORE wide, touching the outer wall's face and the
// string hole's -- and carries one tuning pin per string, each in line across
// the instrument with its hitch pin below. It runs TUNE_EXTRA past the
// outermost pin at least, for wood to hold a pin that is turned; the span is
// then taken out to the next facet midpoint, as the hitch block's is.
const TUNE_LAYERS: usize = 6;
const TUNE_EXTRA: f64 = 15.0;
// --test cuts a piece of the arch to try the joints on before committing a
// whole frame: the string-hole wall's panels TEST_FROM..TEST_TO, every outer
// panel beside them, and a sector of the cheek for each side carrying their
// mortices. TEST_MARGIN of cheek is left beyond the outermost mortice so the
// rim has something to hold on to.
const TEST_FROM: &str = "4";
const TEST_TO: &str = "A";
const TEST_MARGIN: f64 = 10.0;
// --ladder: the same joint cut at each of these, to find the fit by hand
// instead of by arithmetic. Spanning the crossover at 0.18, where the joint
// has neither clearance nor interference, so the sheet holds rungs either side
// of the point where it stops going together by thumb.
const LADDER: [f64; 5] = [0.15, 0.18, 0.21, 0.24, 0.27];
// --kerf-test: the machine's own number, not this instrument's. Two ways of
// asking on one sheet. KERF_CUTS slices give KERF_CUTS + 1 pieces and the
// pieces are short by KERF_CUTS kerfs all together, so the caliper's own error
// divides by that many -- which is the whole point of slicing ten times rather
// than measuring one cut. KERF_SQ is the cross-check: a square hole and the
// piece that fell out of it differ by exactly two kerfs, one off each edge.
const KERF_LEN: f64 = 120.0;
const KERF_WIDE: f64 = 15.0;
const KERF_CUTS: usize = 10;
const KERF_SQ: f64 = 30.0;
const LADDER_MARGIN: f64 = 10.0; // cheek left past the outermost mortice on a coupon
const LADDER_COUPON_H: f64 = 16.0; // across the coupon; the mortices run down its middle

extern "C" {
    fn asin(x: f64) -> f64;
}
/// math.asin: libm, through a pointer the compiler cannot fold.
fn masin(x: f64) -> f64 {
    let f: unsafe extern "C" fn(f64) -> f64 = asin;
    unsafe { std::hint::black_box(f)(x) }
}

type Rect = Vec<P>;

/// The design numbers, y up, the arch centre at the origin.
#[allow(non_snake_case)]
struct Geo {
    A: f64,
    a: f64,
    Hs: f64,
    hs: f64,
    rc: f64,
    cx: f64,
    sag: f64,
    Rb: f64,
    bc: P,
    band: f64,
}

/// The ribbon_bore settings this frame cuts at, with a fit.
fn cfg_for(fit: f64) -> Cfg {
    let mut b = Cfg::default();
    let play0 = b.play_unmeasured;
    b.bore = BORE;
    b.narrow = NARROW;
    b.port = false;
    b.port_at = None;
    b.burn = KERF;
    // The fit, off the notch only. play_by_bore is emptied rather than given
    // an entry for 30: it is ribbon_bore's table of what has been cut on the
    // TRUMPET bores, this instrument has measured its own.
    b.slot_tighten = fit - KERF;
    b.play_by_bore = vec![];
    b.play_unmeasured = play0 - (fit - KERF) / 2.0;
    b
}

struct Lyre {
    fit: f64,
    b: Cfg,
    out: Option<String>,
}

#[allow(non_snake_case)]
fn geometry(b: &Cfg) -> R<Geo> {
    let t = b.thick;
    let band = BORE + 2.0 * t;
    let A = LENGTH * WIDTH_OF_LENGTH / 2.0; // outer half-width
    let a = A - band; // hole half-width
    let Hs = LENGTH - 2.0 * A; // outer straight sides
    let (hs, rc, sag) = (HOLE_STRAIGHT * a, HOLE_CORNER * a, HOLE_BUMP * a);
    let cx = a - rc; // corner centres at +-cx, -hs
    let apex = -hs - rc + sag;
    let d = apex + hs;
    // the bump: a circle below the hole, tangent to both corners from outside
    let Rb = (cx * cx + d * d - rc * rc) / (2.0 * (d + rc));
    if Rb <= 0.0 {
        return Err("the hole corners leave no room for the bump between them".into());
    }
    Ok(Geo { A, a, Hs, hs, rc, cx, sag, Rb, bc: (0.0, apex - Rb), band })
}

fn facets(sweep: f64, r: f64) -> usize {
    let n = 1.0f64.max((py::degrees(sweep).abs() / FACET).ceil());
    1.0f64.max(n.min(ribbon_bore::floordiv(sweep.abs() * r, MIN_PANEL).trunc())) as usize
}

/// Vertices on a circle from angle a0 to a1, both included: ON the circle
/// rather than inscribed with tangent chords, as fine as FACET and no finer
/// than MIN_PANEL along the arc.
fn arc(cx: f64, cy: f64, r: f64, a0: f64, a1: f64) -> Vec<P> {
    let sweep = a1 - a0;
    let n = facets(sweep, r);
    (0..=n)
        .map(|i| {
            let t = a0 + sweep * i as f64 / n as f64;
            (cx + r * cos(t), cy + r * sin(t))
        })
        .collect()
}

/// Vertices whose chords TOUCH the circle, for the outer wall's ends. arc()
/// cuts inside the circle by r(1 - cos(step/2)), which on the outer wall moves
/// the wall INTO the air: the duct over the arch came out 29.72mm against 30.
/// The end vertices fall on the straight sides, so none is returned.
fn tangent_arc(cx: f64, cy: f64, r: f64, a0: f64, a1: f64) -> Vec<P> {
    let sweep = a1 - a0;
    let n = facets(sweep, r);
    let step = sweep / n as f64;
    let rr = r / cos(step / 2.0);
    (0..n)
        .map(|i| {
            let t = a0 + step * (i as f64 + 0.5);
            (cx + rr * cos(t), cy + rr * sin(t))
        })
        .collect()
}

fn join(runs: Vec<Vec<P>>) -> Vec<P> {
    let mut out: Vec<P> = Vec::new();
    for run in runs {
        if out.is_empty() {
            out.extend(run);
        } else {
            out.extend(run.into_iter().skip(1));
        }
    }
    out
}

/// Long axis along the bed: 400 x 188mm fits the bed only lying down.
fn lay(pts: &[P]) -> Vec<P> {
    pts.iter().map(|&(x, y)| (-y, x)).collect()
}

/// How far a closed wall turns at vertex k; the seam is a vertex too.
fn turn(poly: &[P], k: i64) -> f64 {
    let n = poly.len() as i64 - 1;
    let (p0, p1, p2) = (poly[(k - 1).rem_euclid(n) as usize], poly[k.rem_euclid(n) as usize], poly[(k + 1).rem_euclid(n) as usize]);
    let h1 = atan2(p1.1 - p0.1, p1.0 - p0.0);
    let h2 = atan2(p2.1 - p1.1, p2.0 - p1.0);
    (pymod(h2 - h1 + PI, 2.0 * PI) - PI).abs()
}

fn length(r: &[P]) -> f64 {
    fsum(r.windows(2).map(|w| seglen(w[0], w[1])))
}

fn edges(poly: &[P]) -> Vec<(P, P)> {
    poly.windows(2).map(|w| (w[0], w[1])).collect()
}

/// min(... for ...) of a run, the first of equals
fn near(q: P, es: &[(P, P)]) -> f64 {
    fmin(es.iter().map(|&(u, v)| pt_seg(q, u, v)))
}

/// Where a point stands round the arch: 0 on the long axis, + towards +y.
/// Laid down, the arch's centre is the origin and the arch itself is at
/// negative x, so the axis points that way.
fn arch_angle(p: P) -> f64 {
    atan2(p.1, -p.0)
}

/// Where the ray from the arch's centre at angle phi crosses a face.
fn ray_hit(poly: &[P], phi: f64) -> R<P> {
    let (dx, dy) = (-cos(phi), sin(phi));
    for w in poly.windows(2) {
        let (a, b) = (w[0], w[1]);
        let (ux, uy) = (b.0 - a.0, b.1 - a.1);
        let den = dx * uy - dy * ux;
        if den.abs() < 1e-12 {
            continue;
        }
        let t = (dy * a.0 - dx * a.1) / den;
        if (-1e-12..=1.0 + 1e-12).contains(&t) {
            let q = (a.0 + ux * t, a.1 + uy * t);
            if q.0 * dx + q.1 * dy > 0.0 {
                return Ok(q);
            }
        }
    }
    Err(format!("no face at {:.1} degrees round the arch", py::degrees(phi)))
}

/// A run of a face between two angles round the arch, a duplicated vertex
/// dropped (a zero-length run has no direction to offset along).
fn arch_run(face: &[P], lo: f64, hi: f64, strict: bool) -> R<Vec<P>> {
    let mut inside: Vec<P> = face[..face.len() - 1]
        .iter()
        .copied()
        .filter(|&q| {
            let a = arch_angle(q);
            (if strict { lo < a && a < hi } else { a.abs() < hi }) && q.0 < 0.0
        })
        .collect();
    inside.sort_by(|p, q| arch_angle(*p).partial_cmp(&arch_angle(*q)).unwrap());
    let mut edge = vec![ray_hit(face, lo)?];
    edge.extend(inside);
    edge.push(ray_hit(face, hi)?);
    Ok(dedupe(edge))
}

fn dedupe(edge: Vec<P>) -> Vec<P> {
    let mut out: Vec<P> = Vec::new();
    for (k, &q) in edge.iter().enumerate() {
        if k == 0 || seglen(edge[k - 1], q) > 1e-9 {
            out.push(q);
        }
    }
    out
}

fn seg_pts(poly: &[P]) -> Vec<P> {
    let per = 6;
    let mut out = Vec::new();
    for w in poly.windows(2) {
        let (a, b) = (w[0], w[1]);
        for t in 0..per {
            out.push((a.0 + (b.0 - a.0) * t as f64 / per as f64, a.1 + (b.1 - a.1) * t as f64 / per as f64));
        }
    }
    out
}

/// 'M x,y L x,y ...' back to its points.
fn pts_of(d: &str) -> Vec<P> {
    d.replace("M ", "")
        .split(" L ")
        .map(|t| {
            let (a, b) = t.trim().split_once(',').unwrap();
            (a.parse().unwrap(), b.parse().unwrap())
        })
        .collect()
}

fn grp(ds: &[String], col: &str, gid: &str) -> String {
    let body: Vec<String> = ds.iter().map(|d| format!("    <path d=\"{d}\"/>")).collect();
    format!("  <g id=\"{gid}\" fill=\"none\" stroke=\"{col}\" stroke-width=\"0.2\">\n{}\n  </g>\n", body.join("\n"))
}

fn report(ok: &[(bool, String, String)]) -> usize {
    let mut bad = 0;
    for (good, what, detail) in ok {
        println!("  {}  {what:<44} {detail}", if *good { "pass" } else { "FAIL" });
        bad += !good as usize;
    }
    bad
}

fn write(path: &str, body: &str) -> R<()> {
    std::fs::write(path, body).map_err(|e| format!("{path}: {e}"))
}

/// textwrap.wrap(text, width) with its defaults, by its own word splitter.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let ws = r"[\t\n\x0b\x0c\r ]";
    let nws = r"[^\t\n\x0b\x0c\r ]";
    let wp = r#"[\w!"'&.,?]"#;
    let lt = r"[^\d\W]";
    let pat = format!(
        r"({ws}+|(?<={wp})-{{2,}}(?=\w)|{nws}+?(?:-(?:(?<={lt}{{2}}-)|(?<={lt}-{lt}-))(?={lt}-?{lt})|(?={ws}|\z)|(?<={wp})(?=-{{2,}}\w)))"
    );
    let rx = re::rx(&pat);
    let text: String = text.chars().map(|c| if "\t\n\x0b\x0c\r".contains(c) { ' ' } else { c }).collect();
    let mut chunks: Vec<String> = Vec::new();
    let mut at = 0;
    for m in re::all(&rx, &text) {
        let m = m.get(0).unwrap();
        if m.start() > at {
            chunks.push(text[at..m.start()].to_string());
        }
        chunks.push(m.as_str().to_string());
        at = m.end();
    }
    if at < text.len() {
        chunks.push(text[at..].to_string());
    }
    chunks.retain(|c| !c.is_empty());
    chunks.reverse();
    let mut lines: Vec<String> = Vec::new();
    while !chunks.is_empty() {
        let mut cur: Vec<String> = Vec::new();
        let mut cur_len = 0usize;
        if chunks.last().unwrap().trim().is_empty() && !lines.is_empty() {
            chunks.pop();
        }
        while let Some(c) = chunks.last() {
            let l = c.chars().count();
            if cur_len + l <= width {
                cur_len += l;
                cur.push(chunks.pop().unwrap());
            } else {
                break;
            }
        }
        if let Some(c) = chunks.last() {
            if c.chars().count() > width {
                let space = if width < 1 { 1 } else { width - cur_len };
                let head: String = c.chars().take(space).collect();
                let rest: String = c.chars().skip(space).collect();
                cur.push(head);
                *chunks.last_mut().unwrap() = rest;
            }
        }
        if cur.last().map(|c| c.trim().is_empty()).unwrap_or(false) {
            cur.pop();
        }
        if !cur.is_empty() {
            lines.push(cur.concat());
        }
    }
    lines
}

impl Lyre {
    fn new(fit: f64) -> Lyre {
        Lyre { fit, b: cfg_for(fit), out: None }
    }

    fn geo(&self) -> Geo {
        geometry(&self.b).expect("geometry")
    }

    /// (outer, hole) closed polylines, each moved `off` into the duct. off =
    /// THICK gives the two faces the air touches; off = 0 is the drawing's
    /// outline. The outer arcs are tangent_arc(), the hole's arc().
    fn outlines(&self, off: f64) -> R<(Vec<P>, Vec<P>)> {
        let g = geometry(&self.b)?;
        let ro = g.A - off;
        let top = tangent_arc(0.0, 0.0, ro, 0.0, PI);
        let mut outer = top.clone();
        outer.extend(tangent_arc(0.0, -g.Hs, ro, PI, 2.0 * PI));
        outer.push(top[0]);
        // The hole grows by `off`: every radius about its own centre, and the
        // bump's the other way, because its centre lies outside the hole.
        let (ri, rcc, rb) = (g.a + off, g.rc + off, g.Rb - off);
        let (bx, by) = g.bc;
        let (hs, cx) = (g.hs, g.cx);
        let theta_l = atan2(by + hs, cx); // left corner -> bump centre
        let theta_r = atan2(by + hs, -cx); // right corner -> bump centre
        let phi_l = atan2(-hs - by, -cx); // bump -> left corner centre
        let phi_r = atan2(-hs - by, cx); // bump -> right corner centre
        let mut hole = join(vec![
            vec![(ri, -hs), (ri, 0.0)],
            arc(0.0, 0.0, ri, 0.0, PI),
            vec![(-ri, 0.0), (-ri, -hs)],
            arc(-cx, -hs, rcc, PI, 2.0 * PI + theta_l),
            arc(bx, by, rb, phi_l, phi_r),
            arc(cx, -hs, rcc, theta_r, 0.0),
            vec![(ri, -hs)],
        ]);
        let n = hole.len();
        hole[n - 1] = hole[0];
        Ok((outer, hole))
    }

    /// The two wall centrelines and every panel on them. The FACES are drawn,
    /// and the walls follow from them: the face is the section.
    fn build(&self) -> R<(Vec<P>, Vec<P>, Vec<Part>)> {
        let b = &self.b;
        let (o, i) = self.outlines(b.thick)?;
        let (face_o, face_i) = (lay(&o), lay(&i));
        let outer = self.away(&face_o, b.thick / 2.0, true);
        let hole = self.away(&face_i, b.thick / 2.0, false);
        let mut parts = Vec::new();
        let mut seq = 0;
        for (name, poly) in [("inner", &hole), ("outer", &outer)] {
            for (k, w) in poly.windows(2).enumerate() {
                let i = k + 1;
                let (a, bb) = (w[0], w[1]);
                // ribbon_bore's trim: two neighbours meeting at a mitre jam
                // on the concave side unless each is shortened
                let e0 = b.thick / 2.0 * tan(turn(poly, i as i64 - 1) / 2.0);
                let e1 = b.thick / 2.0 * tan(turn(poly, i as i64) / 2.0);
                let l = seglen(a, bb) - e0 - e1;
                if l < b.tooth + 2.0 * b.shoulder {
                    return Err(format!(
                        "{name} panel {i} is {l:.2}mm and a tooth needs {}mm. Raise MIN_PANEL.",
                        g(b.tooth + 2.0 * b.shoulder)
                    ));
                }
                let ang = atan2(bb.1 - a.1, bb.0 - a.0);
                let mid = ((a.0 + bb.0) / 2.0 + (e0 - e1) / 2.0 * cos(ang), (a.1 + bb.1) / 2.0 + (e0 - e1) / 2.0 * sin(ang));
                // OUT means away from the air, asked of the polygon
                let (mut nx, mut ny) = (-sin(ang), cos(ang));
                if in_poly(poly, mid.0 + nx, mid.1 + ny) != (name == "inner") {
                    (nx, ny) = (-nx, -ny);
                }
                seq += 1;
                parts.push(Part {
                    wall: name.into(),
                    n: i,
                    len: l,
                    mid,
                    ang,
                    out: (nx, ny),
                    tag: format!("{seq:X}"),
                    ..Default::default()
                });
            }
        }
        for q in parts.iter_mut() {
            q.teeth = Some(b.teeth(q.len));
        }
        Ok((outer, hole, parts))
    }

    /// poly offset by d, towards whichever side makes it bigger or smaller.
    fn away(&self, poly: &[P], d: f64, bigger: bool) -> Vec<P> {
        let (p, q) = (offset(poly, d), offset(poly, -d));
        if (length(&p) > length(&q)) == bigger {
            p
        } else {
            q
        }
    }

    /// Both rims, as one list the way ribbon_bore's cheek() writes a ring.
    fn cheek(&self, outer: &[P], hole: &[P]) -> Vec<P> {
        let e = self.b.cheek_off() - self.b.wall_off();
        let mut rim_o = self.away(outer, e, true);
        let mut rim_i = self.away(hole, e, false);
        let (no, ni) = (rim_o.len(), rim_i.len());
        rim_o[no - 1] = rim_o[0];
        rim_i[ni - 1] = rim_i[0];
        rim_o.extend(rim_i);
        rim_o
    }

    /// The air's own boundary: each wall's face on the duct side.
    fn faces(&self, outer: &[P], hole: &[P]) -> (Vec<P>, Vec<P>) {
        (self.away(outer, self.b.thick / 2.0, false), self.away(hole, self.b.thick / 2.0, true))
    }

    /// Where the knot's centre goes, laid down, and the span it was placed in.
    fn knot_centre(&self) -> (P, (f64, f64)) {
        let g = self.geo();
        let t = self.b.thick;
        let bottom = -(g.Hs + g.A - t); // outer wall face, far end
        let top = -g.hs - g.rc + g.sag - t; // hole wall face at the bump
        let y = bottom + KNOT_UP * (top - bottom);
        (lay(&[(0.0, y)])[0], (bottom, top))
    }

    /// (edge, outline) of one hitch-block lamination, laid down, true size.
    /// The span is centred on the axis and ends half way along a facet at each
    /// end, which keeps both ends of the block off the panel joints.
    fn hitch_block(&self, face_o: &[P]) -> (Vec<P>, Vec<P>) {
        let n = face_o.len() - 1;
        // the facet crossing the long axis at the resonator end
        let mut best: Option<(f64, usize)> = None;
        for i in 0..n {
            if (face_o[i].1 > 0.0) != (face_o[i + 1].1 > 0.0) {
                let v = py::max(face_o[i].0, face_o[i + 1].0);
                if best.map(|(bv, _)| v >= bv).unwrap_or(true) {
                    best = Some((v, i));
                }
            }
        }
        let c = best.unwrap().1 as i64;
        let reach = HITCH_FACETS as f64 / 2.0;
        let whole = (reach - 0.5).floor() as i64;
        let frac = reach - 0.5 - whole as f64;
        let at = |i: i64, t: f64| -> P {
            let (a, b) = (face_o[i.rem_euclid(n as i64) as usize], face_o[(i + 1).rem_euclid(n as i64) as usize]);
            (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
        };
        let (lo, hi) = (c - whole - 1, c + whole + 1);
        let mut edge = vec![at(lo, 1.0 - frac)];
        edge.extend((lo..hi).map(|i| face_o[(i + 1).rem_euclid(n as i64) as usize]));
        edge.push(at(hi, frac));
        let edge = dedupe(edge);
        let mut inner = offset(&edge, HITCH_WIDTH);
        let probe = inner[inner.len() / 2];
        if !in_poly(face_o, probe.0, probe.1) {
            inner = offset(&edge, -HITCH_WIDTH);
        }
        let mut outline = edge.clone();
        outline.extend(inner.iter().rev());
        outline.push(edge[0]);
        (edge, outline)
    }

    /// Where each hitch pin goes, laid down: across the axis, on the block's
    /// mid-line, half HITCH_WIDTH in from the wall face.
    fn pin_spots(&self, face_o: &[P]) -> R<Vec<P>> {
        let (edge, _) = self.hitch_block(face_o);
        let mut mid = offset(&edge, HITCH_WIDTH / 2.0);
        let m = mid[mid.len() / 2];
        if !in_poly(face_o, m.0, m.1) {
            mid = offset(&edge, -HITCH_WIDTH / 2.0);
        }
        let mut spots = Vec::new();
        'pins: for k in 0..PINS {
            let u = (k as f64 - (PINS - 1) as f64 / 2.0) * PIN_SPACING; // across the axis
            for w in mid.windows(2) {
                let (a, b) = (w[0], w[1]);
                if (a.1 - u) * (b.1 - u) <= 0.0 && a.1 != b.1 {
                    let t = (u - a.1) / (b.1 - a.1);
                    spots.push((a.0 + (b.0 - a.0) * t, u));
                    continue 'pins;
                }
            }
            return Err(format!(
                "hitch pin {}, {}mm off the axis, falls beyond the block, which does not reach that far across. Fewer pins, closer spacing, or more HITCH_FACETS.",
                k + 1,
                g(u)
            ));
        }
        Ok(spots)
    }

    /// Half the tuning block's angle: past the outermost pin, out to a facet mid.
    fn tune_span(&self) -> f64 {
        let r_mid = self.geo().A - self.b.thick - BORE / 2.0;
        let u = (PINS - 1) as f64 / 2.0 * PIN_SPACING;
        let need = masin(u / r_mid) + TUNE_EXTRA / r_mid;
        let step = py::radians(FACET);
        (need / step - 1e-9).ceil() * step
    }

    /// One lamination of the tuning-pin block: both edges are the walls' own
    /// faces over the span, the ends radial.
    fn tuning_block(&self, face_o: &[P], face_i: &[P]) -> R<(Vec<P>, Vec<P>)> {
        let phi = self.tune_span();
        let outer_edge = arch_run(face_o, -phi, phi, false)?;
        let inner_edge = arch_run(face_i, -phi, phi, false)?;
        let mut block = outer_edge.clone();
        block.extend(inner_edge.iter().rev());
        block.push(outer_edge[0]);
        Ok((outer_edge, block))
    }

    /// Each tuning pin: across the axis like its hitch pin, mid-band.
    fn tune_spots(&self) -> Vec<P> {
        let r_mid = self.geo().A - self.b.thick - BORE / 2.0;
        (0..PINS)
            .map(|k| {
                let u = (k as f64 - (PINS - 1) as f64 / 2.0) * PIN_SPACING;
                (-(py::powf(r_mid, 2.0) - py::powf(u, 2.0)).sqrt(), u)
            })
            .collect()
    }

    fn checks(&self, outer: &[P], hole: &[P], parts: &[Part], cheekpoly: &[P], written: &[Written], ink: &[Ink],
              cut_slots: &[(Vec<P>, String)]) -> R<Vec<(bool, String, String)>> {
        let b = &self.b;
        let mut res = Vec::new();
        let mut note = |ok: bool, what: &str, detail: String| res.push((ok, what.to_string(), detail));
        let (face_o, face_i) = self.faces(outer, hole);
        let rims = contours(cheekpoly);
        let rim: Vec<(P, P)> = rims.iter().flat_map(|r| edges(r)).collect();
        let mortices: Vec<Rect> = parts.iter().flat_map(|q| b.slots_for(q)).collect();
        let (eo, ei) = (edges(&face_o), edges(&face_i));

        // --- the section: the narrowest gap between the two duct faces
        let narrow = fmin(seg_pts(&face_i).iter().map(|&p| near(p, &eo)));
        note(narrow >= BORE - 1e-6, "the duct is never narrower than the bore",
             format!("narrowest {narrow:.4}mm wall face to wall face, against {}", g(BORE)));

        // along the long axis: the hole wall's face to the outer wall's at the
        // far end, which is the resonator's length
        let on_axis = |poly: &[P]| {
            fmax(poly.windows(2).filter(|w| (w[0].1 > 0.0) != (w[1].1 > 0.0)).map(|w| {
                let (u, v) = (w[0], w[1]);
                u.0 + (v.0 - u.0) * (0.0 - u.1) / (v.1 - u.1)
            }))
        };
        let reso = on_axis(&face_o) - on_axis(&face_i);
        note(reso > 2.0 * BORE, "the resonator opens out below the hole",
             format!("{reso:.1}mm along the axis from the hole wall to the outer wall, {:.1} bores", reso / BORE));

        let off = mortices.iter().flatten().filter(|&&pt| !(inside(cheekpoly, pt.0, pt.1) || near(pt, &rim) <= b.burn / 2.0)).count();
        note(off == 0 && !mortices.is_empty(), "every slot corner is inside its cheek",
             format!("{} corners on {} slots, {off} outside", 4 * mortices.len(), mortices.len()));

        let gap = |a: &Rect, q: &Rect| {
            fmin((0..4).flat_map(|i| (0..4).map(move |j| (i, j))).map(|(i, j)| seg_gap(a[i], a[(i + 1) % 4], q[j], q[(j + 1) % 4])))
        };
        let mut npairs = 0;
        let mut worst = f64::NAN;
        for i in 0..mortices.len() {
            for j in i + 1..mortices.len() {
                let gp = gap(&mortices[i], &mortices[j]);
                worst = if npairs == 0 { gp } else { py::min(worst, gp) };
                npairs += 1;
            }
        }
        note(worst - b.burn >= b.min_feature - 1e-9, "the ply between two holes survives the kerf",
             format!("{npairs} pairs, narrowest {worst:.3}mm drawn, {:.3}mm after the kerf, against {}", worst - b.burn, g(b.min_feature)));

        let plan_rect = |q: &Part| -> Rect {
            let (hl, ht) = (q.len / 2.0, b.thick / 2.0);
            let (ca, sa) = (cos(q.ang), sin(q.ang));
            let (mx, my) = q.mid;
            [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
                .iter()
                .map(|&(u, v): &(f64, f64)| (mx + u * hl * ca - v * ht * sa, my + u * hl * sa + v * ht * ca))
                .collect()
        };
        let overlap = |pp: &Rect, qq: &Rect| -> bool {
            for r in [pp, qq] {
                for k in 0..4 {
                    let (ex, ey) = (r[(k + 1) % 4].0 - r[k].0, r[(k + 1) % 4].1 - r[k].1);
                    let a: Vec<f64> = pp.iter().map(|&(x, y)| -ey * x + ex * y).collect();
                    let c: Vec<f64> = qq.iter().map(|&(x, y)| -ey * x + ex * y).collect();
                    let (mna, mxa) = (fmin(a.iter().copied()), fmax(a.iter().copied()));
                    let (mnc, mxc) = (fmin(c.iter().copied()), fmax(c.iter().copied()));
                    if mna >= mxc - 1e-7 || mnc >= mxa - 1e-7 {
                        return false;
                    }
                }
            }
            true
        };
        let rects: Vec<Rect> = parts.iter().map(plan_rect).collect();
        let (mut np, mut jam) = (0, 0);
        for i in 0..rects.len() {
            for j in i + 1..rects.len() {
                np += 1;
                jam += overlap(&rects[i], &rects[j]) as usize;
            }
        }
        note(np > 0 && jam == 0, "no two wall panels share plan area", format!("{np} pairs across both walls, {jam} jamming"));

        let short = fmin(parts.iter().map(|q| q.len));
        note(short >= b.tooth + 2.0 * b.shoulder, "the shortest panel still holds a tooth",
             format!("{short:.2}mm against {}mm needed", g(b.tooth + 2.0 * b.shoulder)));

        let bad = ink.iter().filter(|(x, y, owner, _)| !inside(owner, *x, *y)).count();
        note(!ink.is_empty() && bad == 0, "every engraved point is on its own part",
             format!("{} points, {bad} off the material", ink.len()));
        // Inside a hole means deeper than half a kerf: the knot file ends its
        // crossing marks ON the edges of its waste, to three decimals, and two
        // land 0.0007 and 0.0018mm over the line, points the cut burns away.
        let in_hole = |x: f64, y: f64, sl: &[P]| {
            inside(sl, x, y) && fmin((0..sl.len()).map(|i| pt_seg((x, y), sl[(i + sl.len() - 1) % sl.len()], sl[i]))) > b.burn / 2.0
        };
        let over = ink
            .iter()
            .filter(|(x, y, _, f)| cut_slots.iter().any(|(sl, f2)| f2 == f && in_hole(*x, *y, sl)))
            .count();
        let (knot_cut, _) = knot_paths()?;
        let want = 2 * mortices.len() + knot_cut.len();
        note(over == 0 && cut_slots.len() == want, "no engraving lands in a slot",
             format!("{} points against {} holes ({} mortices on each of 2 cheeks plus {} knot cuts, so {want} expected), {over} inside one",
                     ink.len(), cut_slots.len(), mortices.len(), knot_cut.len()));

        // --- the sound hole, wholly over the resonator's air with ply to spare
        let ((kx, ky), (lo, hi)) = self.knot_centre();
        let pts: Vec<P> = knot_cut.iter().flatten().map(|&(x, y)| (x + kx, y + ky)).collect();
        let both: Vec<(P, P)> = eo.iter().chain(&ei).copied().collect();
        let clear = fmin(pts.iter().map(|&q| near(q, &both)));
        let inside_air = pts.iter().all(|&q| in_poly(&face_o, q.0, q.1) && !in_poly(&face_i, q.0, q.1));
        note(inside_air && clear >= b.min_feature + b.burn, "the sound hole lies over the resonator",
             format!("{} cuts, nearest {clear:.1}mm from a wall face, against {}mm", knot_cut.len(), g(b.min_feature + b.burn)));
        let up = (-kx - lo) / (hi - lo);
        // against the request, 2/3, written here: a check that reads the
        // number it is checking cannot fail
        note((up - 2.0 / 3.0).abs() < 1e-9, "the sound hole sits where it was asked",
             format!("centre {:.1}% of the way up the resonator's {:.1}mm, {:.1}mm from its bottom", up * 100.0, hi - lo, -kx - lo));

        let crosses = |a: P, bb: P, c: P, d: P| {
            let (d1, d2) = ((bb.0 - a.0, bb.1 - a.1), (d.0 - c.0, d.1 - c.1));
            let den = d1.0 * d2.1 - d1.1 * d2.0;
            if den.abs() < 1e-12 {
                return false;
            }
            let u = ((c.0 - a.0) * d2.1 - (c.1 - a.1) * d2.0) / den;
            let v = ((c.0 - a.0) * d1.1 - (c.1 - a.1) * d1.0) / den;
            1e-9 < u && u < 1.0 - 1e-9 && 1e-9 < v && v < 1.0 - 1e-9
        };
        let mut xing = 0;
        for i in 0..rim.len() {
            for j in i + 1..rim.len() {
                xing += crosses(rim[i].0, rim[i].1, rim[j].0, rim[j].1) as usize;
            }
        }
        note(xing == 0, "the cheek outline does not cross itself", format!("{} edges on {} contours, {xing} crossing", rim.len(), rims.len()));

        let flush = fmax(mortices.iter().map(|sl| fmin(sl.iter().flat_map(|&p| rim.iter().map(move |&(u, v)| pt_seg(p, u, v))))));
        note(flush <= b.burn / 2.0, "the rim is flush with every mortice",
             format!("furthest of {} mortices {flush:.4}mm from the rim", mortices.len()));

        // no black line through the air
        let in_air = |p: P| in_poly(&face_o, p.0, p.1) && !in_poly(&face_i, p.0, p.1) && near(p, &both) > 1e-6;
        let across = rim
            .iter()
            .flat_map(|&(u, v)| [0.25, 0.5, 0.75].map(|t| (u.0 + (v.0 - u.0) * t, u.1 + (v.1 - u.1) * t)))
            .filter(|&p| in_air(p))
            .count();
        note(across == 0, "no cut line crosses the airway", format!("{} cheek contour(s), {across} point(s) inside the duct", rims.len()));

        // --- the hitch-pin block
        let (edge, block) = self.hitch_block(&face_o);
        let on_wall = fmax(seg_pts(&edge).iter().map(|&q| near(q, &eo)));
        let n = face_o.len() - 1;
        let facet = fmin((0..n).map(|i| seglen(face_o[i], face_o[i + 1])));
        let span = length(&edge);
        // against the request -- 8 facets, 30mm -- not the settings that drew it
        note(on_wall < 1e-6 && (span / facet - 8.0).abs() < 1e-6, "the hitch block lies against the bottom wall",
             format!("{span:.1}mm of the wall face, {} facets of {facet:.2}mm, furthest {}mm off it", g(span / facet), py::e(on_wall, 1)));
        let mut inner_run: Vec<P> = block[edge.len()..block.len() - 1].to_vec();
        inner_run.push(inner_run[inner_run.len() - 1]);
        let widths: Vec<f64> = seg_pts(&inner_run).iter().map(|&q| near(q, &eo)).collect();
        let wmin = fmin(widths.iter().copied());
        note((wmin - 30.0).abs() < 1e-6, "the hitch block is as wide as asked",
             format!("{wmin:.4}mm from the wall face at its narrowest, against {}", g(HITCH_WIDTH)));
        let bedges = edges(&block);
        let to_knot = fmin(pts.iter().map(|&q| near(q, &bedges)));
        let in_res = block.iter().all(|&q| in_poly(&face_o, q.0, q.1) || near(q, &eo) < 1e-6);
        note(in_res && to_knot > 1.5 && !pts.iter().any(|&q| in_poly(&block, q.0, q.1)), "the hitch block stays clear of the sound hole",
             format!("inside the resonator, {to_knot:.1}mm from the nearest knot cut"));
        // --- the hitch pins, against the request: 7 of them, 16mm apart, 2mm
        let spots = self.pin_spots(&face_o)?;
        let gaps: Vec<f64> = spots.windows(2).map(|w| seglen((0.0, w[0].1), (0.0, w[1].1))).collect();
        note(spots.len() == 7 && gaps.iter().all(|&gp| (gp - 16.0).abs() < 1e-9) && fsum(spots.iter().map(|q| q.1)).abs() < 1e-9,
             "the hitch pins are where they were asked",
             format!("{} pins, {} to {}mm apart across the axis, centred on it", spots.len(), g(fmin(gaps.iter().copied())), g(fmax(gaps.iter().copied()))));
        let reach = PIN_DIA; // the cross arms reach this far from the centre
        let in_block = fmin(spots.iter().map(|&q| near(q, &bedges)));
        note(spots.iter().all(|&q| in_poly(&block, q.0, q.1)) && in_block >= reach + 5.0, "every hitch pin lands in the block",
             format!("nearest pin centre {in_block:.1}mm from the block's edge, so a {}mm pin has {:.1}mm of wood round it", g(PIN_DIA), in_block - PIN_DIA / 2.0));
        // --- the tuning-pin block
        let (t_edge, t_block) = self.tuning_block(&face_o, &face_i)?;
        let inner_edge: Vec<P> = t_block[t_edge.len()..t_block.len() - 1].to_vec();
        let on_o = fmax(seg_pts(&t_edge).iter().map(|&q| near(q, &eo)));
        let on_i = fmax(seg_pts(&inner_edge).iter().map(|&q| near(q, &ei)));
        note(py::max(on_o, on_i) < 1e-6, "the tuning block touches both walls",
             format!("{}mm off the outer wall face, {}mm off the string hole's", py::e(on_o, 1), py::e(on_i, 1)));
        let ie = edges(&inner_edge);
        let across = fmin(seg_pts(&t_edge).iter().map(|&q| near(q, &ie)));
        note((across - BORE).abs() < 1e-6, "the tuning block is the duct's width",
             format!("{across:.4}mm wall face to wall face, against {}", g(BORE)));
        let t_spots = self.tune_spots();
        let aligned = t_spots.iter().zip(&spots).all(|(a, c)| (a.1 - c.1).abs() < 1e-9);
        note(t_spots.len() == PINS && aligned, "every tuning pin lines up with its hitch pin",
             format!("{} pins, each within 1e-9mm across the axis of the pin below it", t_spots.len()));
        let t_edges = edges(&t_block);
        let hold = fmin(t_spots.iter().map(|&q| near(q, &t_edges)));
        let ends = [t_block[0], t_edge[t_edge.len() - 1]];
        let past = fmin([t_spots[0], t_spots[t_spots.len() - 1]].iter().flat_map(|&q| ends.map(|e| seglen(q, e))));
        note(t_spots.iter().all(|&q| in_poly(&t_block, q.0, q.1)) && past >= TUNE_EXTRA, "the tuning block runs past the outermost pins",
             format!("{past:.1}mm from the end pin to the nearest end, against {}; {hold:.1}mm of wood round the tightest pin", g(TUNE_EXTRA)));
        let t_deep = TUNE_LAYERS as f64 * b.thick;
        note(t_deep <= BORE, "the tuning block fits between the cheeks",
             format!("{TUNE_LAYERS} layers x {}mm = {}mm in a {}mm duct", g(b.thick), g(t_deep), g(BORE)));
        let deep = HITCH_LAYERS as f64 * b.thick;
        note(deep <= BORE, "the laminated block fits between the cheeks",
             format!("{HITCH_LAYERS} layers x {}mm = {}mm in a {}mm duct", g(b.thick), g(deep), g(BORE)));

        let big = written.iter().filter(|w| w.1 > BED_W || w.2 > BED_H).count();
        note(!written.is_empty() && big == 0, "every sheet fits the P2S bed",
             format!("{} sheet(s), largest {:.0} x {:.0}mm against {} x {}", written.len(),
                     fmax(written.iter().map(|w| w.1)), fmax(written.iter().map(|w| w.2)), g(BED_W), g(BED_H)));
        Ok(res)
    }

    /// One block's laminations on a sheet of their own, numbered 1 to layers,
    /// drawn BURN/2 outside the true outline as ribbon_bore draws every part.
    #[allow(clippy::too_many_arguments)]
    fn lamination_sheet(&self, tag: &str, block: &[P], layers: usize, spot: P, note: &str, what: &str, out_path: &str,
                        write_: bool, ink: &mut Vec<Ink>) -> R<Vec<Written>> {
        let b = &self.b;
        let mut cut = self.away(block, b.burn / 2.0, true);
        let n = cut.len();
        cut[n - 1] = cut[0];
        let items: Vec<Item> = (1..=layers)
            .map(|k| Item {
                outline: cut.clone(),
                slots: vec![],
                marks: Box::new(move |dx: f64, dy: f64| label(&format!("{k:X}"), spot.0 + dx, spot.1 + dy, 5.0, PI / 2.0)),
                note: String::new(),
            })
            .collect();
        let (stem, ext) = splitext(out_path);
        let name = format!("{stem}-{tag}-cut-files{ext}");
        let mut written = Vec::new();
        let refs: Vec<&Item> = items.iter().collect();
        for (k, placed) in pack(&refs, MARGIN_S, 4.0)?.into_iter().enumerate() {
            let n = k + 1;
            let path_here = if n == 1 { name.clone() } else { name.replace("-cut-files", &format!("-sheet{n}-cut-files")) };
            let (mut marks, mut cuts) = (Vec::new(), Vec::new());
            for &(it, dx, dy) in &placed {
                let here: Vec<P> = it.outline.iter().map(|q| (q.0 + dx, q.1 + dy)).collect();
                cuts.push(path(&here, true));
                for d in (it.marks)(dx, dy) {
                    for (a, c) in pts_of(&d) {
                        ink.push((a, c, here.clone(), path_here.clone()));
                    }
                    marks.push(d);
                }
            }
            let w = fmax(placed.iter().map(|&(it, dx, _)| bbox(&it.outline).2 + dx)) + MARGIN_S;
            let h = fmax(placed.iter().map(|&(it, _, dy)| bbox(&it.outline).3 + dy)) + MARGIN_S;
            let body = format!(
                "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}mm\" height=\"{h:.2}mm\" viewBox=\"0 0 {w:.2} {h:.2}\">\n<title>Lyre-harp frame, {note}</title>\n<desc>1 user unit = 1mm. {note}. {layers} identical laminations of {}mm ply, {}mm glued up. {what} Drawn {}mm oversize for a {}mm kerf, joints cut to the fit of {}. Blue #0000ff engraves, black #000000 cuts.</desc>\n{}{}</svg>\n",
                g(b.thick), g(layers as f64 * b.thick), g(b.burn / 2.0), g(b.burn), g(self.fit),
                grp(&marks, MARK, "numbers"), grp(&cuts, CUT, "outlines")
            );
            if write_ {
                write(&path_here, &body)?;
            }
            written.push((basename(&path_here).to_string(), w, h, placed.len(), note.to_string()));
        }
        Ok(written)
    }

    /// Both laminated blocks, a sheet each.
    fn blocks_sheets(&self, face_o: &[P], face_i: &[P], out_path: &str, write_: bool, ink: &mut Vec<Ink>) -> R<Vec<Written>> {
        let (_, hitch) = self.hitch_block(face_o);
        let spot = (fmax(hitch.iter().map(|q| q.0)) - HITCH_WIDTH / 2.0, 0.0);
        let mut out = self.lamination_sheet(
            "hitch-block", &hitch, HITCH_LAYERS, spot,
            &format!("the hitch-pin block - {HITCH_LAYERS} laminations, glue them in number order"),
            &format!("The long curved edge lies against the inside of the bottom wall over {HITCH_FACETS} facets; the block is {}mm wide from it. The {PINS} hitch pins go through the front cheek into it, {}mm apart.",
                     g(HITCH_WIDTH), g(PIN_SPACING)),
            out_path, write_, ink)?;
        let (_, tune) = self.tuning_block(face_o, face_i)?;
        let spot = (-(self.geo().A - self.b.thick - BORE / 2.0), 0.0);
        out.extend(self.lamination_sheet(
            "tuning-block", &tune, TUNE_LAYERS, spot,
            &format!("the tuning-pin block - {TUNE_LAYERS} laminations, glue them in number order"),
            &format!("It fills the duct at the arch, touching the outer wall and the string hole's wall, {}mm between them, and runs {:.0} degrees round the arch. The {PINS} tuning pins go through the front cheek into it, {}mm apart, each in line across the instrument with its hitch pin.",
                     g(BORE), 2.0 * py::degrees(self.tune_span()), g(PIN_SPACING)),
            out_path, write_, ink)?);
        Ok(out)
    }

    /// The panels the test piece is made of, and the arc they cover. The inner
    /// run is named by tag; the outer is whichever panels stand beside them.
    fn test_parts(&self, parts: &[Part]) -> R<(Vec<Part>, (f64, f64))> {
        let hex = |s: &str| i64::from_str_radix(s, 16).unwrap();
        let (lo, hi) = (hex(TEST_FROM), hex(TEST_TO));
        let inner: Vec<Part> = parts.iter().filter(|q| q.wall == "inner" && lo <= hex(&q.tag) && hex(&q.tag) <= hi).cloned().collect();
        if inner.is_empty() {
            return Err(format!("no inner panel is tagged {TEST_FROM}..{TEST_TO}"));
        }
        let end = |q: &Part, sg: f64| (q.mid.0 + sg * cos(q.ang) * q.len / 2.0, q.mid.1 + sg * sin(q.ang) * q.len / 2.0);
        let ends: Vec<f64> = inner
            .iter()
            .flat_map(|q| {
                [q.mid, (q.mid.0 + cos(q.ang) * q.len / 2.0, q.mid.1 + sin(q.ang) * q.len / 2.0),
                 (q.mid.0 - cos(q.ang) * q.len / 2.0, q.mid.1 - sin(q.ang) * q.len / 2.0)]
            })
            .map(arch_angle)
            .collect();
        let (a0, a1) = (fmin(ends.iter().copied()), fmax(ends.iter().copied()));
        let spans = |q: &Part| {
            let (s, t) = (arch_angle(end(q, -1.0)), arch_angle(end(q, 1.0)));
            if t < s { (t, s) } else { (s, t) }
        };
        // every outer panel that OVERLAPS the arc, not only those whose middle
        // falls in it; angles wrap at the resonator end, so the far end is
        // kept out by where it stands
        let outer: Vec<Part> = parts
            .iter()
            .filter(|q| q.wall == "outer" && q.mid.0 < 0.0 && arch_angle(q.mid).abs() < PI / 2.0 && spans(q).0 < a1 - 1e-9 && spans(q).1 > a0 + 1e-9)
            .cloned()
            .collect();
        let mut here = inner;
        here.extend(outer);
        Ok((here, (a0, a1)))
    }

    /// A sector of the cheek over the arc, with room past the mortices.
    fn test_cheek(&self, rims: &[Vec<P>], span: (f64, f64)) -> R<Vec<P>> {
        let (a0, a1) = span;
        let r_hole = rims.iter().fold(&rims[0], |m, r| if length(r) < length(m) { r } else { m });
        let r_out = rims.iter().fold(&rims[0], |m, r| if length(r) > length(m) { r } else { m });
        let grow = TEST_MARGIN / (self.geo().A - self.b.thick); // margin, as an angle
        let (lo, hi) = (a0 - grow, a1 + grow);
        let out_edge = arch_run(r_out, lo, hi, true)?;
        let hole_edge = arch_run(r_hole, lo, hi, true)?;
        let mut s = out_edge.clone();
        s.extend(hole_edge.iter().rev());
        s.push(out_edge[0]);
        Ok(s)
    }

    /// The whole test piece on one sheet: two cheek sectors and the panels.
    fn test_sheet(&self, parts: &[Part], rims: &[Vec<P>], out_path: &str, write_: bool, ink: &mut Vec<Ink>) -> R<Vec<Written>> {
        let b = &self.b;
        let (here, span) = self.test_parts(parts)?;
        let sector = self.test_cheek(rims, span)?;
        let slots: Vec<Rect> = here.iter().flat_map(|q| b.slots_for(q)).collect();
        let mut items: Vec<Item> = Vec::new();
        let sx = fmin(sector.iter().map(|q| q.0));
        for k in ["A", "B"] {
            let here2 = here.clone();
            items.push(Item {
                outline: sector.clone(),
                slots: slots.clone(),
                marks: Box::new(move |dx: f64, dy: f64| {
                    let mut m = label(k, sx + 24.0 + dx, dy, 4.0, PI / 2.0);
                    for q in &here2 {
                        let (ox, oy) = q.out;
                        m.extend(label(&q.tag, q.mid.0 - ox * 5.5 + dx, q.mid.1 - oy * 5.5 + dy, 2.0, q.ang));
                    }
                    m
                }),
                note: String::new(),
            });
        }
        for q in &here {
            let w = q.len + b.burn;
            let h2 = (BORE + 2.0 * b.thick + b.burn) / 2.0;
            let poly: Vec<P> = b.panel(q.len, q.teeth.as_deref()).iter().map(|&(px, py_)| (px + w / 2.0, py_ + h2)).collect();
            let t = q.tag.clone();
            items.push(Item {
                outline: poly,
                slots: vec![],
                marks: Box::new(move |dx: f64, dy: f64| label(&t, w / 2.0 + dx, h2 + dy, 3.2, 0.0)),
                note: String::new(),
            });
        }
        let (stem, ext) = splitext(out_path);
        let name = format!("{stem}-test-arch-{TEST_FROM}to{TEST_TO}-cut-files{ext}");
        let n_in = here.iter().filter(|q| q.wall == "inner").count();
        let note = format!("ARCH TEST PIECE - 2 cheek sectors, {n_in} string-hole panels {TEST_FROM}-{TEST_TO} and {} outer panels", here.len() - n_in);
        let mut written = Vec::new();
        let refs: Vec<&Item> = items.iter().collect();
        let grp_ = |ds: &[String], col: &str, gid: &str| if ds.is_empty() { String::new() } else { grp(ds, col, gid) };
        for (k, placed) in pack(&refs, MARGIN_S, 4.0)?.into_iter().enumerate() {
            let n = k + 1;
            let path_here = if n == 1 { name.clone() } else { name.replace("-cut-files", &format!("-sheet{n}-cut-files")) };
            let (mut marks, mut holes, mut cuts) = (Vec::new(), Vec::new(), Vec::new());
            for &(it, dx, dy) in &placed {
                let there: Vec<P> = it.outline.iter().map(|q| (q.0 + dx, q.1 + dy)).collect();
                cuts.push(path(&there, true));
                for sl in &it.slots {
                    let moved: Vec<P> = sl.iter().map(|q| (q.0 + dx, q.1 + dy)).collect();
                    holes.push(path(&moved, true));
                }
                for d in (it.marks)(dx, dy) {
                    for (a, c) in pts_of(&d) {
                        ink.push((a, c, there.clone(), path_here.clone()));
                    }
                    marks.push(d);
                }
            }
            let w = fmax(placed.iter().map(|&(it, dx, _)| bbox(&it.outline).2 + dx)) + MARGIN_S;
            let h = fmax(placed.iter().map(|&(it, _, dy)| bbox(&it.outline).3 + dy)) + MARGIN_S;
            let body = format!(
                "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}mm\" height=\"{h:.2}mm\" viewBox=\"0 0 {w:.2} {h:.2}\">\n<title>Lyre-harp frame - {note}</title>\n<desc>1 user unit = 1mm. {note}. A slice of the arch, cut to try the finger joints before committing a whole frame: the panels stand between the two sectors exactly as they do in the instrument, {}mm apart. Sector A and sector B are the same part. {}mm ply, slots for a {}mm sheet at {}mm kerf, cut to the fit of {}. Blue #0000ff engraves, orange #ff8000 cuts the slots first, black #000000 frees the parts.</desc>\n{}{}{}</svg>\n",
                g(BORE), g(b.thick), g(b.sheet), g(KERF), g(self.fit),
                grp_(&marks, MARK, "numbers"), grp_(&holes, INNER, "slots"), grp_(&cuts, CUT, "outlines")
            );
            if write_ {
                write(&path_here, &body)?;
            }
            written.push((basename(&path_here).to_string(), w, h, placed.len(), note.clone()));
        }
        // every mortice has to sit in the sector, as on the whole cheek
        let rim = edges(&sector);
        let off = slots.iter().flatten().filter(|&&q| !(inside(&sector, q.0, q.1) || near(q, &rim) <= b.burn / 2.0)).count();
        if off > 0 {
            return Err(format!("{off} mortice corner(s) fall outside the test sector; raise TEST_MARGIN"));
        }
        Ok(written)
    }

    fn stem(&self) -> String {
        format!("lyre-harp-bore{}-{:.0}x{}mm{}", g(BORE), 2.0 * self.geo().A, g(LENGTH), if NARROW { "-narrow" } else { "" })
    }

    fn main(&self, write_: bool) -> R<i32> {
        let b = &self.b;
        let (outer, hole, parts) = self.build()?;
        let cheekpoly = self.cheek(&outer, &hole);
        let gg = geometry(b)?;
        let out_path = match &self.out {
            Some(o) => o.clone(),
            None => format!("{}/{}.svg", env!("CARGO_MANIFEST_DIR"), self.stem()),
        };
        // ribbon_bore's sheet places the cheek's '0' along the start of the
        // "centreline" it is handed; there is none here, so a short run down
        // the middle of one parallel side, in the duct, clear of the mortices
        let mid = gg.A - gg.band / 2.0;
        let lead = lay(&[(mid, -10.0), (mid, -60.0)]);
        let ((kx, ky), _) = self.knot_centre();
        let (knot_cut, knot_ink) = knot_paths()?;
        let holes: Vec<Vec<P>> = knot_cut.iter().map(|lp| lp.iter().map(|&(x, y)| (x + kx, y + ky)).collect()).collect();
        let marks: Vec<String> = knot_ink
            .iter()
            .map(|run| path(&run.iter().map(|&(x, y)| (x + kx, y + ky)).collect::<Vec<P>>(), false))
            .collect();
        let face_o = self.faces(&outer, &hole).0;
        let spots = self.pin_spots(&face_o)?;
        let t_spots = self.tune_spots();
        // ribbon_bore's items, with the one cheek made two. A, the front,
        // carries the sound hole: its waste goes in with the mortices, and its
        // crossing marks are engraved. B, the back, is the plain cheek.
        let (mut cheeks, panels) = b.items_for(&parts, &cheekpoly, &lead)?;
        if cheeks.len() != 1 {
            return Err(format!("ValueError: too many values to unpack (expected 1)"));
        }
        let ck = cheeks.remove(0);
        let ck_marks: Rc<dyn Fn(f64, f64) -> Vec<String> + '_> = Rc::from(ck.marks);
        let (m1, m2) = (ck_marks.clone(), ck_marks);
        let mut front_slots = ck.slots.clone();
        front_slots.extend(holes);
        let front = Item {
            outline: ck.outline.clone(),
            slots: front_slots,
            marks: Box::new(move |dx: f64, dy: f64| {
                let mut m = m1(dx, dy);
                for k in &marks {
                    let pts: Vec<P> = k[2..]
                        .split(" L ")
                        .map(|t| {
                            let (a, c) = t.split_once(',').unwrap();
                            (a.parse::<f64>().unwrap() + dx, c.parse::<f64>().unwrap() + dy)
                        })
                        .collect();
                    m.push(path(&pts, false));
                }
                m.extend(pin_marks(&spots, dx, dy));
                m.extend(tune_marks(&t_spots, dx, dy));
                m
            }),
            note: "cheek A, the front - CUT THIS SHEET ONCE - carries the sound hole and the marks for the hitch and tuning pins".into(),
        };
        let back = Item {
            outline: ck.outline,
            slots: ck.slots,
            marks: Box::new(move |dx: f64, dy: f64| m2(dx, dy)),
            note: "cheek B, the back - CUT THIS SHEET ONCE".into(),
        };
        let (mut written, mut ink, cut_slots) = b.sheet_items(&parts, &[front, back], &panels, &out_path, write_)?;
        let title = format!("Lyre-harp frame, {}mm bore, {:.0} x {}mm", g(BORE), 2.0 * gg.A, g(LENGTH));
        let dir = std::path::Path::new(&out_path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
        if write_ {
            // sheet() writes a ribbon duct's title and description -- constant
            // section, swept along a curve -- which is not this part
            for (name, _, _, _, note) in &written {
                let f = dir.join(name);
                let mut body = std::fs::read_to_string(&f).map_err(|e| e.to_string())?;
                body = replace_first(&body, "<title>", "</title>", &format!("<title>{title} - {note}</title>"));
                body = replace_first(&body, "<desc>", "</desc>", &format!(
                    "<desc>1 user unit = 1mm. {note}. A closed duct {}mm deep between two cheeks, {}mm wide over the arch and down the parallel sides and opening to the full gap between the hole and the outer wall below the hole, which is the resonator. {} wall panels, numbered round each wall; each cheek slot carries its panel's number. {}mm ply, slots for a {}mm sheet at {}mm kerf, joints cut to the fit of {}. Blue #0000ff engraves, orange #ff8000 cuts the slots first, black #000000 frees the parts.</desc>",
                    g(BORE), g(BORE), parts.len(), g(b.thick), g(b.sheet), g(b.burn), g(self.fit)));
                std::fs::write(&f, body).map_err(|e| e.to_string())?;
            }
        }
        let (fo, fi) = self.faces(&outer, &hole);
        written.extend(self.blocks_sheets(&fo, &fi, &out_path, write_, &mut ink)?);

        let n_in = parts.iter().filter(|q| q.wall == "inner").count();
        println!("lyre-harp frame   {}mm bore, {:.1} x {}mm", g(BORE), 2.0 * gg.A, g(LENGTH));
        println!("  outer: half-circles R{:.1} and {:.1}mm parallel sides", gg.A, gg.Hs);
        println!("  hole: half-circle R{:.1}, {:.1}mm parallel sides, corners R{:.1}, bump R{:.1} rising {:.1}mm", gg.a, gg.hs, gg.rc, gg.Rb, gg.sag);
        println!("  band {}mm over the arch and sides; resonator below the hole", g(gg.band));
        println!("  {}mm ply, {}mm kerf measured, joints cut to the fit of {}: {}mm off the notch, {}mm play a side",
                 g(b.thick), g(KERF), g(self.fit), g(b.slot_tighten), g(b.play()));
        println!("  hitch-pin block: {HITCH_LAYERS} laminations x {}mm = {}mm, {}mm wide over {HITCH_FACETS} facets of the bottom wall",
                 g(b.thick), g(HITCH_LAYERS as f64 * b.thick), g(HITCH_WIDTH));
        println!("  tuning-pin block: {TUNE_LAYERS} laminations x {}mm = {}mm, the duct's {}mm wide over {:.0} degrees of the arch",
                 g(b.thick), g(TUNE_LAYERS as f64 * b.thick), g(BORE), 2.0 * py::degrees(self.tune_span()));
        println!("  {PINS} strings: hitch and tuning pins {}mm apart, marked on cheek A", g(PIN_SPACING));
        println!("  {n_in} inner + {} outer panels + 2 cheeks + {HITCH_LAYERS} + {TUNE_LAYERS} laminations = {} parts, {} sheets",
                 parts.len() - n_in, parts.len() + 2 + HITCH_LAYERS + TUNE_LAYERS, written.len());
        for (name, w, h, k, note) in &written {
            println!("    {name:<58}{k:>3} parts  {w:.0} x {h:.0}mm  {note}");
        }
        println!();
        let bad = report(&self.checks(&outer, &hole, &parts, &cheekpoly, &written, &ink, &cut_slots)?);
        if write_ && bad > 0 {
            for (name, ..) in &written {
                let _ = std::fs::remove_file(dir.join(name));
            }
            println!("\n  {bad} check(s) failed. Nothing written.");
        } else if write_ {
            println!("\n  wrote {} file(s)", written.len());
        }
        Ok(if bad > 0 { 1 } else { 0 })
    }
}

fn replace_first(body: &str, open: &str, close: &str, with: &str) -> String {
    match body.find(open) {
        Some(i) => match body[i..].find(close) {
            Some(j) => format!("{}{with}{}", &body[..i], &body[i + j + close.len()..]),
            None => body.to_string(),
        },
        None => body.to_string(),
    }
}

/// The engraved drilling marks: a PIN_DIA circle and a cross through it.
fn pin_marks(spots: &[P], dx: f64, dy: f64) -> Vec<String> {
    let (r, arm) = (PIN_DIA / 2.0, PIN_DIA);
    let mut out = Vec::new();
    for &(x, y) in spots {
        let (x, y) = (x + dx, y + dy);
        let ring: Vec<P> = (0..25).map(|i| (x + r * cos(2.0 * PI * i as f64 / 24.0), y + r * sin(2.0 * PI * i as f64 / 24.0))).collect();
        out.push(path(&ring, false));
        out.push(path(&[(x - arm, y), (x + arm, y)], false));
        out.push(path(&[(x, y - arm), (x, y + arm)], false));
    }
    out
}

/// Drilling marks for the tuning pins: a cross, no circle -- the pin's
/// diameter is the tuner's to choose, and a ring drawn at a guess would be
/// read as the hole to drill.
fn tune_marks(spots: &[P], dx: f64, dy: f64) -> Vec<String> {
    let mut out = Vec::new();
    for &(x, y) in spots {
        let (x, y) = (x + dx, y + dy);
        out.push(path(&[(x - 3.0, y), (x + 3.0, y)], false));
        out.push(path(&[(x, y - 3.0), (x, y + 3.0)], false));
    }
    out
}

fn knot_file() -> String {
    format!("{}/{KNOT}", std::env::var("HOME").unwrap_or_default())
}

/// (cut loops, engraved polylines) from the knot file, in its own mm: 1 unit
/// = 1mm about the hole's centre. Its black group is the waste that drops
/// out, every path closed; its blue group is the crossing marks.
fn knot_paths() -> R<(Vec<Vec<P>>, Vec<Vec<P>>)> {
    let knot = knot_file();
    let body = std::fs::read_to_string(&knot).map_err(|e| format!("{knot}: {e}"))?;
    let group = |name: &str| -> R<Vec<(Vec<P>, bool)>> {
        let g_rx = re::rx(&format!(r#"(?s)<g id="{name}"[^>]*>(.*?)</g>"#));
        let m = re::first(&g_rx, &body).ok_or_else(|| format!("{knot} has no <g id=\"{name}\">"))?;
        let inner = re::g(&m, 1);
        let d_rx = re::rx(r#"\bd="([^"]*)""#);
        let pt_rx = re::rx(r"(-?[\d.]+)[ ,](-?[\d.]+)");
        let mut runs = Vec::new();
        for d in re::findall1(&d_rx, inner, 1) {
            let mut cuts: Vec<usize> = d.match_indices('M').map(|(i, _)| i).collect();
            if cuts.first() != Some(&0) {
                cuts.insert(0, 0);
            }
            cuts.push(d.len());
            for w in cuts.windows(2) {
                let sub = &d[w[0]..w[1]];
                let pts: Vec<P> = re::all(&pt_rx, sub)
                    .iter()
                    .map(|c| (re::g(c, 1).parse().unwrap(), re::g(c, 2).parse().unwrap()))
                    .collect();
                if pts.len() > 1 {
                    runs.push((pts, sub.contains('Z')));
                }
            }
        }
        Ok(runs)
    };
    let cut = group("cut")?;
    if !cut.iter().all(|(_, closed)| *closed) {
        return Err(format!("{knot}: a cut path is not closed"));
    }
    Ok((cut.into_iter().map(|(p, _)| p).collect(), group("engrave")?.into_iter().map(|(p, _)| p).collect()))
}

/// Measure the machine's kerf, in the stock the instrument is cut from. THE
/// SLICED STRIP IS ONE RECTANGLE PLUS OPEN LINES, NOT N RECTANGLES: drawn as
/// touching rectangles every interior edge is cut twice and the answer comes
/// out half.
fn kerf_test(path_out: &str) -> R<i32> {
    let (m, gap) = (MARGIN_S, 6.0);
    let (mut cuts, mut lines, mut marks): (Vec<String>, Vec<String>, Vec<String>) = (vec![], vec![], vec![]);
    let mut y = m;
    // the control: no interior cuts at all
    let ctrl = vec![(m, y), (m + KERF_LEN, y), (m + KERF_LEN, y + KERF_WIDE), (m, y + KERF_WIDE)];
    cuts.push(path(&ctrl, true));
    marks.extend(label("C", m + KERF_LEN / 2.0, y + KERF_WIDE / 2.0, 5.0, 0.0));
    // the comb: the same rectangle, sliced
    y += KERF_WIDE + gap;
    let comb = vec![(m, y), (m + KERF_LEN, y), (m + KERF_LEN, y + KERF_WIDE), (m, y + KERF_WIDE)];
    cuts.push(path(&comb, true));
    let step = KERF_LEN / (KERF_CUTS + 1) as f64;
    for i in 1..=KERF_CUTS {
        let x = m + i as f64 * step;
        lines.push(path(&[(x, y), (x, y + KERF_WIDE)], false));
    }
    for i in 0..=KERF_CUTS {
        marks.extend(label(&format!("{:X}", i + 1), m + (i as f64 + 0.5) * step, y + KERF_WIDE / 2.0, 4.0, 0.0));
    }
    // the cross-check: a square hole and the piece that drops out of it
    y += KERF_WIDE + gap;
    let pad = 12.0;
    let plate = vec![(m, y), (m + KERF_SQ + 2.0 * pad, y), (m + KERF_SQ + 2.0 * pad, y + KERF_SQ + 2.0 * pad), (m, y + KERF_SQ + 2.0 * pad)];
    cuts.push(path(&plate, true));
    let sq = vec![(m + pad, y + pad), (m + pad + KERF_SQ, y + pad), (m + pad + KERF_SQ, y + pad + KERF_SQ), (m + pad, y + pad + KERF_SQ)];
    cuts.push(path(&sq, true));
    // D for the dropout: a digit would read as a twelfth slice on the bench
    marks.extend(label("D", m + pad + KERF_SQ / 2.0, y + pad + KERF_SQ / 2.0, 6.0, 0.0));
    let w = 2.0 * m + KERF_LEN;
    let h = y + KERF_SQ + 2.0 * pad + m;
    let all_cuts: Vec<String> = cuts.iter().chain(&lines).cloned().collect();
    let body = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}mm\" height=\"{h:.2}mm\" viewBox=\"0 0 {w:.2} {h:.2}\">\n<title>KERF TEST - measure this machine, in this ply</title>\n<desc>1 user unit = 1mm. CUT IN THE STOCK THE JOB IS CUT FROM, at the same power, speed, focus and air - kerf moves with the material, so a figure from other stock is about other stock. Strip C is the control and is not sliced. The strip below it is the same rectangle with {KERF_CUTS} cuts across it, giving {} pieces numbered 1 to {:X}. Butt all {} back together against a straightedge, lightly clamped, and measure: kerf = (C - the stack) / {KERF_CUTS}. The square is a cross-check - measure the hole and the piece D that fell out of it, kerf = (hole - piece) / 2. Blue #0000ff engraves, black #000000 cuts. Do not let the laser cut any line twice.</desc>\n  <g id=\"numbers\" fill=\"none\" stroke=\"{MARK}\" stroke-width=\"0.2\">\n{}\n  </g>\n  <g id=\"outlines\" fill=\"none\" stroke=\"{CUT}\" stroke-width=\"0.2\">\n{}\n  </g>\n</svg>\n",
        KERF_CUTS + 1, KERF_CUTS + 1, KERF_CUTS + 1,
        marks.iter().map(|d| format!("    <path d=\"{d}\"/>")).collect::<Vec<_>>().join("\n"),
        all_cuts.iter().map(|d| format!("    <path d=\"{d}\"/>")).collect::<Vec<_>>().join("\n"),
    );
    let mut ok = Vec::new();
    ok.push((cuts.len() == 4 && lines.len() == KERF_CUTS, "a control, a comb, and the square".to_string(),
             format!("4 outlines and {} slicing lines", lines.len())));
    // Every slice line must appear ONCE.
    let uniq: std::collections::HashSet<&String> = lines.iter().collect();
    let dup = lines.len() - uniq.len();
    ok.push((dup == 0, "no slicing line is cut twice".into(), format!("{} lines, {dup} duplicated", lines.len())));
    ok.push((lines.iter().all(|d| d.matches(" L ").count() == 1), "each slice is one open line, not a rectangle".into(),
             format!("{} lines, 2 points each, none closed", lines.len())));
    let wid = |q: &[P]| py::round_dp(fmax(q.iter().map(|p| p.0)) - fmin(q.iter().map(|p| p.0)), 6);
    let mut widths = vec![wid(&ctrl), wid(&comb)];
    widths.sort_by(|a, b| a.partial_cmp(b).unwrap());
    widths.dedup();
    ok.push((widths.len() == 1, "control and comb are the same rectangle".into(),
             format!("both {:.3} x {}mm, so the outside kerf cancels", widths[0], g(KERF_WIDE))));
    ok.push(((step * (KERF_CUTS + 1) as f64 - KERF_LEN).abs() < 1e-9, "the slices divide the strip exactly".into(),
             format!("{} pieces of {step:.3}mm", KERF_CUTS + 1)));
    ok.push((w <= BED_W && h <= BED_H, "the sheet fits the P2S bed".into(), format!("{w:.0} x {h:.0}mm against {BED_W:.0} x {BED_H:.0}")));
    println!("KERF TEST   the machine's own number, in the job's own ply");
    println!("  strip {} x {}mm, {KERF_CUTS} cuts -> {} pieces;  square {}mm\n", g(KERF_LEN), g(KERF_WIDE), KERF_CUTS + 1, g(KERF_SQ));
    let bad = report(&ok);
    if bad > 0 {
        println!("\n  {bad} check(s) failed. Nothing written.");
        return Ok(1);
    }
    write(path_out, &body)?;
    println!("\n  wrote {}  {w:.0} x {h:.0}mm", basename(path_out));
    println!("\n  kerf = (C - the {} pieces butted up) / {KERF_CUTS}", KERF_CUTS + 1);
    println!("  kerf = (the square hole - the piece that fell out) / 2");
    println!("  currently in lyre_harp.rs: KERF = {}", g(KERF));
    Ok(0)
}

/// One sheet of the same joint at several FIT values, to find the fit by hand.
/// ONE PANEL, NOT ONE PER RUNG: panel() reads the kerf and the tooth and
/// nothing else, so the tab is identical at every fit and the mortice is the
/// whole variable. Rungs are engraved in HUNDREDTHS -- 15 is 0.15 -- because
/// the glyph table is hex digits and has no decimal point.
fn ladder(path_out: &str) -> R<i32> {
    struct Rung {
        f: f64,
        q: Part,
        pn: Vec<P>,
        sl: Vec<Rect>,
    }
    let mut rungs = Vec::new();
    for &f in &LADDER {
        let l = Lyre::new(f);
        let (_, _, parts) = l.build()?;
        let q = parts
            .iter()
            .find(|p| p.wall == "outer" && p.teeth.as_ref().map(|t| t.len()) == Some(2))
            .ok_or("StopIteration")?
            .clone();
        let mut local = q.clone();
        local.mid = (0.0, 0.0);
        local.ang = 0.0;
        let pn = l.b.panel(q.len, q.teeth.as_deref());
        let sl = l.b.slots_for(&local);
        rungs.push(Rung { f, q, pn, sl });
    }
    let tag = rungs[0].q.tag.clone();
    let l0 = rungs[0].q.len;
    // the coupon: a strip of cheek with this panel's mortices in it
    let (cw, ch) = (l0 + 2.0 * LADDER_MARGIN, LADDER_COUPON_H);
    let pan = rungs[0].pn.clone();
    let pw = fmax(pan.iter().map(|p| p.0)) - fmin(pan.iter().map(|p| p.0));
    let ph = fmax(pan.iter().map(|p| p.1)) - fmin(pan.iter().map(|p| p.1));
    let (m, gap) = (MARGIN_S, 6.0);
    let rowh = py::max(ch, ph);
    let w = 2.0 * m + cw + gap + pw;
    let h = 2.0 * m + LADDER.len() as f64 * rowh + (LADDER.len() - 1) as f64 * gap;
    let (mut cuts, mut holes, mut marks) = (Vec::new(), Vec::new(), Vec::new());
    for (i, r) in rungs.iter().enumerate() {
        let oy = m + i as f64 * (rowh + gap) + rowh / 2.0;
        let ox = m + cw / 2.0;
        let bx = vec![(ox - cw / 2.0, oy - ch / 2.0), (ox + cw / 2.0, oy - ch / 2.0), (ox + cw / 2.0, oy + ch / 2.0), (ox - cw / 2.0, oy + ch / 2.0)];
        cuts.push(path(&bx, true));
        for s in &r.sl {
            holes.push(path(&s.iter().map(|&(x, y)| (x + ox, y + oy)).collect::<Vec<P>>(), true));
        }
        marks.extend(label(&format!("{:02}", py::round(r.f * 100.0)), ox, oy + ch / 4.0, 3.0, 0.0));
    }
    // the panel, and one spare, in the right-hand column
    for i in 0..2 {
        let oy = m + i as f64 * (rowh + gap) + rowh / 2.0;
        let ox = m + cw + gap + pw / 2.0;
        let poly: Vec<P> = pan.iter().map(|&(x, y)| (x + ox, y + oy)).collect();
        cuts.push(path(&poly, true));
        marks.extend(label(&tag, ox, oy, 2.0, 0.0));
    }
    let fits: Vec<String> = LADDER.iter().map(|&f| g(f)).collect();
    let fits = fits.join(", ");
    let lines = |v: &[String]| v.iter().map(|d| format!("    <path d=\"{d}\"/>")).collect::<Vec<_>>().join("\n");
    let body = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}mm\" height=\"{h:.2}mm\" viewBox=\"0 0 {w:.2} {h:.2}\">\n<title>Lyre-harp FIT LADDER - the same joint at {} fits</title>\n<desc>1 user unit = 1mm. The same finger joint cut at FIT {fits}, to find the one that needs a mallet. Each coupon is engraved with its fit in HUNDREDTHS: 15 is 0.15. The panel is marked {tag} and is the same part at every rung - panel() reads only the kerf, so the mortice is the only thing that changes - so push the ONE panel into each coupon in turn, softest first, and take the rung that needs persuading. The second panel is a spare. {}mm ply, {}mm kerf. Blue #0000ff engraves, orange #ff8000 cuts the mortices first, black #000000 frees the parts.</desc>\n  <g id=\"numbers\" fill=\"none\" stroke=\"{MARK}\" stroke-width=\"0.2\">\n{}\n  </g>\n  <g id=\"slots\" fill=\"none\" stroke=\"{INNER}\" stroke-width=\"0.2\">\n{}\n  </g>\n  <g id=\"outlines\" fill=\"none\" stroke=\"{CUT}\" stroke-width=\"0.2\">\n{}\n  </g>\n</svg>\n",
        LADDER.len(), g(Cfg::default().thick), g(KERF), lines(&marks), lines(&holes), lines(&cuts)
    );
    let mut ok = Vec::new();
    ok.push((rungs.len() == LADDER.len() && cuts.len() == LADDER.len() + 2, "one coupon a rung, and the panel".to_string(),
             format!("{} coupons, 2 panels, {} mortices", LADDER.len(), holes.len())));
    // Read the DRAWN mortices, not the formula that placed them.
    let ext = |s: &[P], k: usize| {
        let v: Vec<f64> = s.iter().map(|p| if k == 0 { p.0 } else { p.1 }).collect();
        fmax(v.iter().copied()) - fmin(v.iter().copied())
    };
    let wide: Vec<f64> = rungs.iter().map(|r| ext(&r.sl[0], 0)).collect();
    let deep: Vec<f64> = rungs.iter().map(|r| ext(&r.sl[0], 1)).collect();
    let down = wide.windows(2).all(|p| p[0] > p[1] + 1e-9);
    let span = LADDER[LADDER.len() - 1] - LADDER[0];
    let (wl, dl) = (wide[wide.len() - 1], deep[deep.len() - 1]);
    // 2:1 and 1:1: along the tooth both faces are drawn, across the ply only one
    ok.push((down && ((wide[0] - wl) - 2.0 * span).abs() < 1e-6, "every rung is tighter than the last".into(),
             format!("mortice {:.3} down to {wl:.3}mm, {:.3} across {span:.2} of fit, 2:1", wide[0], wide[0] - wl)));
    ok.push((deep.windows(2).all(|p| p[0] > p[1] + 1e-9) && ((deep[0] - dl) - span).abs() < 1e-6, "and across the ply, at half the rate".into(),
             format!("{:.3} down to {dl:.3}mm, {:.3} across {span:.2} of fit, 1:1", deep[0], deep[0] - dl)));
    // the tab does NOT change; that is the claim the sheet is built on
    let same = rungs.iter().all(|r| r.pn == rungs[0].pn);
    ok.push((same, "the panel is the same at every rung".into(), format!("{} points, identical across {} builds", pan.len(), LADDER.len())));
    let mut off = 0;
    for r in &rungs {
        for s in &r.sl {
            for &(x, y) in s {
                if !(x.abs() < cw / 2.0 - 1e-9 && y.abs() < ch / 2.0 - 1e-9) {
                    off += 1;
                }
            }
        }
    }
    ok.push((off == 0, "every mortice sits inside its coupon".into(),
             format!("{} mortices, {off} corners out", rungs.iter().map(|r| r.sl.len()).sum::<usize>())));
    ok.push((w <= BED_W && h <= BED_H, "the sheet fits the P2S bed".into(), format!("{w:.0} x {h:.0}mm against {BED_W:.0} x {BED_H:.0}")));
    println!("lyre-harp FIT LADDER   the same joint at {} fits: {fits}", LADDER.len());
    println!("  panel {tag}, {l0:.1}mm, 2 teeth - the joint there are 22 of\n");
    let bad = report(&ok);
    if bad > 0 {
        println!("\n  {bad} check(s) failed. Nothing written.");
        return Ok(1);
    }
    write(path_out, &body)?;
    println!("\n  wrote {}  {w:.0} x {h:.0}mm", basename(path_out));
    Ok(0)
}

/// The slot numbers alone, mirrored, to engrave on cheek A's BACK face. The
/// cheek is mirror-symmetric about the long axis, so the plate turned over
/// about that axis lands back on its own footprint. FLIPPED THE OTHER WAY IT
/// WILL NOT: the string hole is not symmetric end for end. So each label is
/// reflected y -> -y and its angle negated, while label() goes on drawing
/// readable glyphs.
fn back_numbers(l: &Lyre, path_out: &str) -> R<i32> {
    let b = &l.b;
    let (outer, hole, parts) = l.build()?;
    let ckp = l.cheek(&outer, &hole);
    let (x0, x1) = (fmin(ckp.iter().map(|p| p.0)), fmax(ckp.iter().map(|p| p.0)));
    let (y0, y1) = (fmin(ckp.iter().map(|p| p.1)), fmax(ckp.iter().map(|p| p.1)));
    let m = MARGIN_S;
    let (dx, dy) = (m - x0, m - y0);
    let (w, h) = (x1 - x0 + 2.0 * m, y1 - y0 + 2.0 * m);
    let off = b.thick / 2.0 + 1.5; // ribbon_bore's own label offset, into the duct
    // Kept per panel, so the checks read the ink actually DRAWN
    let mut marks: Vec<String> = Vec::new();
    let mut drawn: Vec<(String, f64, Vec<P>, Vec<P>)> = Vec::new();
    for q in &parts {
        let (ox, oy) = q.out;
        let (lx, ly) = (q.mid.0 - ox * off, q.mid.1 - oy * off);
        let mine = label(&q.tag, lx + dx, -ly + dy, 2.0, -q.ang);
        // mine's last path is label()'s baseline tick, laid along its angle
        let tick = pts_of(&mine[mine.len() - 1]);
        let pts: Vec<P> = mine.iter().flat_map(|d| pts_of(d)).collect();
        drawn.push((q.tag.clone(), q.ang, tick, pts));
        marks.extend(mine);
    }
    let body = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}mm\" height=\"{h:.2}mm\" viewBox=\"0 0 {w:.2} {h:.2}\">\n<title>Lyre-harp cheek A - BACK FACE NUMBERS - engrave only, no cutting</title>\n<desc>1 user unit = 1mm. The {} slot numbers for cheek A, mirrored for its BACK face. ENGRAVE ONLY - there is not a cut line in this file. Cut cheek A from its own sheet first, then turn it over ABOUT ITS LONG AXIS - the 400mm one, so the sound hole stays at the same end - lay it in the same place on the bed and run this. Each number then sits beside its own mortice and reads the right way round while you drop the panels in. Turning the plate the other way, end for end, does NOT work: the string hole is not symmetric that way. Blue #0000ff engraves.</desc>\n  <g id=\"numbers\" fill=\"none\" stroke=\"{MARK}\" stroke-width=\"0.2\">\n{}\n  </g>\n</svg>\n",
        parts.len(),
        marks.iter().map(|d| format!("    <path d=\"{d}\"/>")).collect::<Vec<_>>().join("\n")
    );
    let mut ok = Vec::new();
    let ink: Vec<P> = marks.iter().flat_map(|d| pts_of(d)).collect();
    ok.push((!marks.is_empty() && drawn.len() == parts.len(), "one number for every panel".to_string(),
             format!("{} numbers for {} panels, {} strokes", drawn.len(), parts.len(), marks.len())));
    // The plate is turned over, so the mirror of the cheek is what the ink
    // has to land on: test the symmetry, do not assume it
    let key = |x: f64, y: f64| {
        let z = |v: f64| if v == 0.0 { 0u64 } else { v.to_bits() };
        (z(py::round_dp(x, 4)), z(py::round_dp(y, 4)))
    };
    let a: std::collections::HashSet<(u64, u64)> = ckp.iter().map(|&(px, py_)| key(px, -py_)).collect();
    let c: std::collections::HashSet<(u64, u64)> = ckp.iter().map(|&(px, py_)| key(px, py_)).collect();
    let sym = a == c;
    ok.push((sym, "the plate is symmetric about its long axis".into(),
             format!("{} outline points, mirror {}", ckp.len(), if sym { "present for every one" } else { "MISSING for some" })));
    // inside(), NOT in_poly(): in_poly wants the first point repeated as the
    // last, and slots_for() returns bare four-point rectangles
    let mut rings = contours(&ckp);
    let wd = |c: &Vec<P>| fmax(c.iter().map(|p| p.0)) - fmin(c.iter().map(|p| p.0));
    rings.sort_by(|p, q| wd(p).partial_cmp(&wd(q)).unwrap());
    let (inner, band) = (&rings[0], &rings[rings.len() - 1]);
    let on_material = |x: f64, y: f64| inside(band, x, y) && !inside(inner, x, y);
    let off_mat = ink.iter().filter(|&&(px, py_)| !on_material(px - dx, -(py_ - dy))).count();
    ok.push((off_mat == 0, "every number lands on the plate".into(), format!("{} ink points, {off_mat} off the material", ink.len())));
    let slots: Vec<Rect> = parts.iter().flat_map(|q| b.slots_for(q)).collect();
    let in_slot = ink.iter().filter(|&&(px, py_)| slots.iter().any(|sl| inside(sl, px - dx, -(py_ - dy)))).count();
    ok.push((in_slot == 0, "no number lands in a mortice".into(),
             format!("{} ink points against {} mortices, {in_slot} inside one", ink.len(), slots.len())));
    ok.push((body.contains("<path") && !body.contains("stroke=\"#000000\""), "engrave only, no cut line".into(), "no black stroke in the file".into()));
    // THE ONE THAT MATTERS: the mortice nearest each label, turned back over,
    // has to be one of the panel's own; the runner-up margin is reported
    let mut owner: Vec<(P, String)> = Vec::new();
    for q in &parts {
        for sl in b.slots_for(q) {
            let k = (py::round_dp(fsum(sl.iter().map(|p| p.0)) / sl.len() as f64, 4), py::round_dp(fsum(sl.iter().map(|p| p.1)) / sl.len() as f64, 4));
            match owner.iter_mut().find(|(c, _)| *c == k) {
                Some(e) => e.1 = q.tag.clone(),
                None => owner.push((k, q.tag.clone())),
            }
        }
    }
    let (mut worst, mut wtag) = (1e9f64, "None".to_string());
    let mut wrong = 0;
    for (tag, _, _, pts) in &drawn {
        let cx = fsum(pts.iter().map(|p| p.0)) / pts.len() as f64 - dx;
        let cy = -(fsum(pts.iter().map(|p| p.1)) / pts.len() as f64 - dy);
        let d2 = |c: P| py::powf(c.0 - cx, 2.0) + py::powf(c.1 - cy, 2.0);
        let mut ranked: Vec<&(P, String)> = owner.iter().collect();
        ranked.sort_by(|p, q| d2(p.0).partial_cmp(&d2(q.0)).unwrap());
        if &ranked[0].1 != tag {
            wrong += 1;
        }
        let d0 = py::dist(ranked[0].0, (cx, cy));
        if let Some(other) = ranked.iter().find(|c| &c.1 != tag) {
            let dd = py::dist(other.0, (cx, cy)) - d0;
            if dd < worst {
                worst = dd;
                wtag = tag.clone();
            }
        }
    }
    ok.push((wrong == 0, "each number names the mortice beside it".into(),
             format!("{} numbers, {wrong} nearer another panel's mortice; tightest margin {worst:.2}mm on panel {wtag}", drawn.len())));
    // A number can sit in the right place and still be laid along the wrong
    // line: the baseline tick gives the direction actually drawn
    let mut skew = 0.0f64;
    for (_, ang, tick, _) in &drawn {
        let ((ax, ay), (bx, by)) = (tick[0], tick[1]);
        let got = atan2(-(by - ay), bx - ax); // turned back over
        let d = (pymod(got - ang + PI, 2.0 * PI) - PI).abs();
        skew = py::max(skew, d);
    }
    // Half a degree: the tick is 0.44mm long and written to three decimals
    ok.push((skew < py::radians(0.5), "each number lies along its own panel".into(),
             format!("worst direction error {:.4} degrees, against 0.5 allowed for rounding", py::degrees(skew))));
    println!("lyre-harp cheek A, BACK FACE NUMBERS   engrave only, no cutting");
    println!("  {} numbers, mirrored about the long axis, {w:.0} x {h:.0}mm page\n", drawn.len());
    let bad = report(&ok);
    if bad > 0 {
        println!("\n  {bad} check(s) failed. Nothing written.");
        return Ok(1);
    }
    write(path_out, &body)?;
    println!("\n  wrote {}", basename(path_out));
    Ok(0)
}

/// Write the arch test piece on its own, without the whole frame.
fn test_piece(l: &Lyre, path_out: &str) -> R<i32> {
    let (outer, hole, parts) = l.build()?;
    let rims = contours(&l.cheek(&outer, &hole));
    let mut ink = Vec::new();
    let written = l.test_sheet(&parts, &rims, path_out, true, &mut ink)?;
    let (here, span) = l.test_parts(&parts)?;
    println!("lyre-harp arch test   panels {TEST_FROM}-{TEST_TO} of the string hole wall and the {} outer panels beside them",
             here.iter().filter(|q| q.wall == "outer").count());
    println!("  {:.0} degrees of arch, {}mm of cheek past the outermost mortice, {}mm kerf, joints cut to the fit of {}",
             py::degrees(span.1 - span.0), g(TEST_MARGIN), g(KERF), g(l.fit));
    for (name, w, h, k, _) in &written {
        println!("    {name:<58}{k:>3} parts  {w:.0} x {h:.0}mm");
    }
    Ok(0)
}

/// The drawing the design started from, redrawn from the parts as cut: every
/// line and dimension is read from the same build() the cut files come from.
/// Stood upright, arch at the top, the way the sketch was drawn. Not a cut file.
#[allow(non_snake_case)]
fn drawing(l: &Lyre, path_out: &str) -> R<i32> {
    let b = &l.b;
    let (outer, hole, parts) = l.build()?;
    let _ = parts;
    let rims = contours(&l.cheek(&outer, &hole));
    let (face_o, face_i) = l.faces(&outer, &hole);
    let gg = geometry(b)?;
    let ((kx, ky), (lo, hi)) = l.knot_centre();
    let (knot_cut, _) = knot_paths()?;
    // Laid down -> upright, arch at the top. A quarter TURN: swapping x and y
    // instead is a reflection, and drew the knot as its mirror image.
    let up = |pts: &[P]| -> Vec<P> { pts.iter().map(|&(x, y)| (-y, x)).collect() };
    let d = |pts: &[P], close: bool| path(&up(pts), close);
    let xs: Vec<f64> = rims.iter().flat_map(|r| up(r)).map(|q| q.0).collect();
    let ys: Vec<f64> = rims.iter().flat_map(|r| up(r)).map(|q| q.1).collect();
    let (x0, x1, y0, y1) = (fmin(xs.iter().copied()), fmax(xs.iter().copied()), fmin(ys.iter().copied()), fmax(ys.iter().copied()));
    let (W, L) = (x1 - x0, y1 - y0);
    let (grey, blue) = ("#6b6b6b", "#1f6fb2");
    let m = 60.0;
    let vb = (x0 - m, y0 - m, W + 2.0 * m + 70.0, L + 2.0 * m + 30.0);
    let dim = |a: P, c: P, text: &str, tx: f64, ty: f64, rotate: bool| {
        let t = if rotate { format!(" transform=\"rotate(90 {tx:.2} {ty:.2})\"") } else { String::new() };
        format!(
            "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{grey}\" stroke-width=\"0.4\" marker-start=\"url(#ar)\" marker-end=\"url(#ar)\"/><text x=\"{tx:.2}\" y=\"{ty:.2}\" text-anchor=\"middle\"{t}>{text}</text>",
            a.0, a.1, c.0, c.1
        )
    };
    // measured, upright: arch apex at the top (small y), resonator below
    let ax = 0.0; // the long axis, x = 0
    let (top_o, bot_o) = (y0, y1);
    let hole_rim = rims.iter().fold(&rims[0], |mm, r| if length(r) < length(mm) { r } else { mm });
    let h_up = up(hole_rim);
    let hx0 = fmin(h_up.iter().map(|q| q.0));
    let hole_w = fmax(h_up.iter().map(|q| q.0)) - hx0;
    let hole_top = fmin(h_up.iter().map(|q| q.1));
    let side_band = hx0 - x0;
    let top_band = hole_top - top_o;
    let k_up = up(&[(kx, ky)])[0];
    let (reso_lo, reso_hi) = (-lo, -hi); // upright y of the air's bottom, top
    let band_y = hole_top + gg.a + b.thick + 0.6 * gg.hs; // on the parallel sides

    let mut body: Vec<String> = Vec::new();
    for r in &rims {
        body.push(format!("<path d=\"{}\" fill=\"none\" stroke=\"#111\" stroke-width=\"0.9\"/>", d(r, true)));
    }
    for f in [&face_o, &face_i] {
        body.push(format!("<path d=\"{}\" fill=\"none\" stroke=\"{blue}\" stroke-width=\"0.5\" stroke-dasharray=\"1.5 2\"/>", d(f, true)));
    }
    for lp in &knot_cut {
        let moved: Vec<P> = lp.iter().map(|&(x, y)| (x + kx, y + ky)).collect();
        body.push(format!("<path d=\"{}\" fill=\"#e9e2cf\" stroke=\"#111\" stroke-width=\"0.3\"/>", d(&moved, true)));
    }
    let (_, block) = l.hitch_block(&face_o);
    body.push(format!("<path d=\"{}\" fill=\"#d8b98b\" fill-opacity=\"0.55\" stroke=\"#7a5a36\" stroke-width=\"0.5\"/>", d(&block, true)));
    let (_, tune) = l.tuning_block(&face_o, &face_i)?;
    body.push(format!("<path d=\"{}\" fill=\"#d8b98b\" fill-opacity=\"0.55\" stroke=\"#7a5a36\" stroke-width=\"0.5\"/>", d(&tune, true)));
    let mut pins = l.pin_spots(&face_o)?;
    pins.extend(l.tune_spots());
    for q in up(&pins) {
        body.push(format!("<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{}\" fill=\"#3a3f44\"/>", q.0, q.1, g(PIN_DIA / 2.0)));
    }
    body.push(format!(
        "<text x=\"{ax:.2}\" y=\"{:.2}\" text-anchor=\"middle\" font-size=\"8\">tuning-pin block, {TUNE_LAYERS} x {} = {}mm</text>",
        fmin(up(&face_i).iter().map(|q| q.1)) + 15.0, g(b.thick), g(TUNE_LAYERS as f64 * b.thick)
    ));
    // just above the block's inner edge, in the open resonator
    body.push(format!(
        "<text x=\"{ax:.2}\" y=\"{:.2}\" text-anchor=\"middle\" font-size=\"8\">hitch-pin block, {HITCH_LAYERS} x {} = {}mm</text>",
        fmax(up(&block).iter().map(|q| q.1)) - HITCH_WIDTH - 13.0, g(b.thick), g(HITCH_LAYERS as f64 * b.thick)
    ));
    body.push(dim((x0, top_o - 22.0), (x1, top_o - 22.0), &format!("{W:.0}"), (x0 + x1) / 2.0, top_o - 27.0, false));
    body.push(dim((x1 + 30.0, top_o), (x1 + 30.0, bot_o), &format!("{L:.0}"), x1 + 38.0, (top_o + bot_o) / 2.0, true));
    body.push(dim((hx0, band_y - 14.0), (hx0 + hole_w, band_y - 14.0), &format!("hole {hole_w:.0}"), ax, band_y - 19.0, false));
    body.push(dim((x0, band_y), (hx0, band_y), &format!("{side_band:.0}"), (x0 + hx0) / 2.0, band_y - 4.0, false));
    body.push(dim((x1 - side_band, band_y), (x1, band_y), &format!("{side_band:.0}"), x1 - side_band / 2.0, band_y - 4.0, false));
    body.push(dim((ax, top_o), (ax, hole_top), &format!("{top_band:.0}"), ax + 9.0, (top_o + hole_top) / 2.0 + 3.0, false));
    body.push(dim((ax - 70.0, reso_hi), (ax - 70.0, reso_lo), &format!("resonator {:.0}", reso_lo - reso_hi), ax - 78.0, (reso_hi + reso_lo) / 2.0, true));
    body.push(dim((ax + 55.0, k_up.1), (ax + 55.0, reso_lo), &format!("{:.1} = 2/3", reso_lo - k_up.1), ax + 63.0, (k_up.1 + reso_lo) / 2.0, true));
    body.push(format!(
        "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{grey}\" stroke-width=\"0.3\" stroke-dasharray=\"4 2\"/>",
        ax - 40.0, k_up.1, ax + 60.0, k_up.1
    ));
    let notes = [
        format!("Lyre-harp frame, {W:.0} x {L:.0}mm, drawn from lyre_harp.rs: the same build() as the cut files."),
        "Solid: the cheek's two rims as cut. Dotted blue: the faces the air touches.".to_string(),
        format!("The duct is {}mm deep throughout, {}mm wide over the arch and down the sides ({side_band:.0}mm outside), and opens below the hole into the resonator.", g(BORE), g(BORE)),
        "Sound hole: the 2-lead 7-bight knot, r30, front cheek only, centred on the axis 2/3 up the resonator.".to_string(),
        format!("Hitch-pin block: {HITCH_LAYERS} laminations, {}mm wide against the bottom wall over {HITCH_FACETS} facets; {PINS} {}mm hitch pins {}mm apart.",
                g(HITCH_WIDTH), g(PIN_DIA), g(PIN_SPACING)),
        format!("Tuning-pin block: {TUNE_LAYERS} laminations filling the duct at the arch, one tuning pin above each hitch pin."),
    ];
    // Wrapped to the sheet: what fits from x0 to the right edge at font-size 8
    let room = vb.0 + vb.2 - x0 - 10.0;
    let lines: Vec<String> = notes.iter().flat_map(|t| wrap(t, (room / 4.5) as usize)).collect();
    for (k, t) in lines.iter().enumerate() {
        body.push(format!("<text x=\"{x0:.2}\" y=\"{:.2}\" font-size=\"8\">{t}</text>", bot_o + 22.0 + 12.0 * k as f64));
    }
    let tall = py::max(vb.3 + 20.0, bot_o + 22.0 + 12.0 * (lines.len() as f64 - 1.0) + 12.0 - vb.1);
    let svg = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.1}mm\" height=\"{tall:.1}mm\" viewBox=\"{:.2} {:.2} {:.2} {tall:.2}\" font-family=\"Helvetica, Arial, sans-serif\" font-size=\"9\" fill=\"{grey}\">\n<title>Lyre-harp frame, {W:.0} x {L:.0}mm</title>\n<desc>Drawing for review, not a cut file. 1 unit = 1mm. Drawn from lyre_harp.rs's build(), the geometry the cut files in lyre-harp-bore30-188x400mm/ come from.</desc>\n<defs><marker id=\"ar\" viewBox=\"0 0 10 10\" refX=\"5\" refY=\"5\" markerWidth=\"4\" markerHeight=\"4\" orient=\"auto-start-reverse\"><path d=\"M0,2 L10,5 L0,8 z\" fill=\"{grey}\"/></marker></defs>\n<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{tall:.2}\" fill=\"#ffffff\"/>\n{}\n</svg>\n",
        vb.2, vb.0, vb.1, vb.2, vb.0, vb.1, vb.2, body.join("\n")
    );
    write(path_out, &svg)?;
    println!("  drew {}: {W:.1} x {L:.1}mm, hole {hole_w:.1}, sides {side_band:.1}, top {top_band:.1}, resonator {:.1}, knot {:.1} up",
             basename(path_out), reso_lo - reso_hi, reso_lo - k_up.1);
    Ok(0)
}

/// The interactive 3D page, from the same build() as the cut files, in
/// upright coordinates, y up. three.js r128 from cdnjs draws it.
fn render(l: &Lyre, path_out: &str) -> R<i32> {
    let (outer, hole, parts) = l.build()?;
    let mut rims = contours(&l.cheek(&outer, &hole));
    rims.sort_by(|p, q| (-length(p)).partial_cmp(&-length(q)).unwrap());
    let ((kx, ky), (lo, hi)) = l.knot_centre();
    let (kcut, _) = knot_paths()?;
    let (face_o, face_i) = l.faces(&outer, &hole);
    let up = |p: P| (py::round_dp(-p.1, 3), py::round_dp(-p.0, 3));
    let num = |x: f64| J::N(F(x));
    let pt = |p: P| J::A(vec![num(p.0), num(p.1)]);
    let pts = |v: &[P]| J::A(v.iter().map(|&p| pt(up(p))).collect());
    let open = |v: &[P]| v[..v.len() - 1].to_vec();
    let rim: Vec<P> = open(&rims[0]).into_iter().map(up).collect();
    let holev: Vec<P> = open(&rims[1]).into_iter().map(up).collect();
    let span = |v: &[P], k: usize| {
        let c: Vec<f64> = v.iter().map(|p| if k == 0 { p.0 } else { p.1 }).collect();
        fmax(c.iter().copied()) - fmin(c.iter().copied())
    };
    let (_, hitch) = l.hitch_block(&face_o);
    let (_, tune) = l.tuning_block(&face_o, &face_i)?;
    let data = J::obj(vec![
        ("rim", J::A(rim.iter().map(|&p| pt(p)).collect())),
        ("hole", J::A(holev.iter().map(|&p| pt(p)).collect())),
        ("knot", J::A(kcut.iter().map(|lp| J::A(lp.iter().map(|&(x, y)| pt(up((x + kx, y + ky)))).collect())).collect())),
        ("panels", J::A(parts.iter().map(|q| J::obj(vec![
            ("x", num(up(q.mid).0)),
            ("y", num(up(q.mid).1)),
            ("len", num(py::round_dp(q.len, 3))),
            ("wall", J::S(q.wall.clone())),
            ("tag", J::S(q.tag.clone())),
        ])).collect())),
        ("panels_dir", J::A(parts.iter().map(|q| J::obj(vec![
            ("dx", num(py::round_dp(-sin(q.ang), 6))),
            ("dy", num(py::round_dp(-cos(q.ang), 6))),
        ])).collect())),
        ("bore", num(BORE)),
        ("t", num(l.b.thick)),
        ("hitch", pts(&open(&hitch))),
        ("hitch_layers", J::N(I(HITCH_LAYERS as i64))),
        ("pins", pts(&l.pin_spots(&face_o)?)),
        ("tune", pts(&open(&tune))),
        ("tune_pins", pts(&l.tune_spots())),
        ("tune_layers", J::N(I(TUNE_LAYERS as i64))),
        ("pin", J::A(vec![num(PIN_DIA), num(PIN_SPACING)])),
        ("size", J::A(vec![num(py::round_dp(span(&rim, 0), 1)), num(py::round_dp(span(&rim, 1), 1))])),
        ("reso", num(py::round_dp(hi - lo, 1))),
        ("knot_up", num(py::round_dp(-kx - lo, 1))),
        ("hole_w", num(py::round_dp(span(&holev, 0), 1))),
    ]);
    let mut s = String::new();
    data.dump(&mut s);
    write(path_out, &lyre_harp_page::RENDER_PAGE.replace("__DATA__", &s))?;
    println!("  rendered {}: {} panels, {} knot cuts", basename(path_out), parts.len(), kcut.len());
    Ok(0)
}

/// An Err starting with NUL is a SystemExit: its text on stderr.
fn run(a: &[String]) -> R<i32> {
    const TAKES: [&str; 8] = ["--out=", "--drawing=", "--render=", "--test=", "--fit=", "--back-numbers=", "--ladder=", "--kerf-test="];
    for x in a {
        if !(x == "--no-write" || TAKES.iter().any(|p| x.starts_with(p))) {
            return Err(format!(
                "\x00error: {x} is not a flag this generator reads. It takes --out=, --drawing=, --render=, --test=, --fit=, --back-numbers=, --ladder=, --kerf-test= and --no-write."
            ));
        }
    }
    // --fit= is the friction knob: the kerf the JOINTS are drawn as if cut at;
    // the beam stays at the measured KERF and every other line is unmoved.
    let mut fit = FIT;
    if let Some(ft) = a.iter().filter(|x| x.starts_with("--fit=")).last() {
        fit = py::float(ft.split_once('=').unwrap().1).ok_or_else(|| format!("\x00error: {ft} is not a number."))?;
        // Below the kerf the notch would be drawn WIDER than the tooth
        if fit < KERF {
            return Err(format!(
                "\x00error: --fit={} is under the {}mm kerf, so it would loosen the joint, not tighten it. --fit={} is the neutral value.",
                g(fit), g(KERF), g(KERF)
            ));
        }
    }
    let mut l = Lyre::new(fit);
    l.out = a.iter().find(|x| x.starts_with("--out=")).map(|x| x.split_once('=').unwrap().1.to_string());
    let arg = |p: &str| a.iter().find(|x| x.starts_with(p)).map(|x| x.split_once('=').unwrap().1.to_string());
    if let Some(p) = arg("--kerf-test=") {
        return kerf_test(&p);
    }
    if let Some(p) = arg("--ladder=") {
        return ladder(&p);
    }
    if let Some(p) = arg("--back-numbers=") {
        return back_numbers(&l, &p);
    }
    if let Some(p) = arg("--test=") {
        return test_piece(&l, &p);
    }
    if let Some(p) = arg("--render=") {
        return render(&l, &p);
    }
    if let Some(p) = arg("--drawing=") {
        return drawing(&l, &p);
    }
    l.main(!a.iter().any(|x| x == "--no-write"))
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(match run(&a) {
        Ok(rc) => rc,
        Err(e) => match e.strip_prefix('\x00') {
            Some(m) => {
                eprintln!("{m}");
                1
            }
            None => {
                println!("error: {e}");
                1
            }
        },
    });
}
