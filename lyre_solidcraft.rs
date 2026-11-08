//! The frame as a parametric SolidCraft design (2026-11-08):
//! `lyre_harp --solidcraft=DIR` writes it, and SolidCraft builds the STEP
//! file from it.
//!
//! Every point the 3D view is built from is a read-only parameter, worked out
//! from the one before it as the generator works it out: the two outlines
//! from the length, the width, the bore and the hole's proportions; the air's
//! faces; each wall's centreline, offset half a ply from its face; the
//! cheeks' rims, the narrow band past the centreline that holds the
//! mortices; every panel along its wall, mitre-trimmed at each end; the hitch
//! block along the outer face and the tuning block between the faces; the
//! knot, at its share of the resonator. So the frame follows its size, its
//! bore, its ply and its fit. Which facet a block starts on, and how many
//! facets each arc has, are the default build's: they stay put while the
//! numbers move.
//!
//! As in the 3D view the panels are plain boxes, without teeth or slots, the
//! cheeks have no mortices, and the pins are left out. The frame lies in XY,
//! as the page has it, the cheeks at Z = +-bore/2.
use super::*;
use lasermade_tools::solidcraft::{n as nn, Script};

/// A point: its two coordinates as expressions (or parameters), and their
/// value at the defaults, which every decision below is taken on.
#[derive(Clone)]
struct Sp {
    x: String,
    y: String,
    v: P,
}

struct Ctx {
    s: Script,
    k: usize,
}

impl Ctx {
    /// A named point: its coordinates become read-only parameters.
    fn pt(&mut self, name: &str, x: String, y: String, v: P) -> Sp {
        self.k += 1;
        let (nx, ny) = (format!("{name}_x"), format!("{name}_y"));
        self.s.len(&nx, x, "read only: a point of the frame");
        self.s.len(&ny, y, "read only");
        Sp { x: nx, y: ny, v }
    }
    fn num(&mut self, name: &str, e: String) -> String {
        self.s.num(name, e, "read only");
        name.to_string()
    }
    fn len(&mut self, name: &str, e: String) -> String {
        self.s.len(name, e, "read only");
        name.to_string()
    }
}

/// A closed polygon's edge normals (-uy, ux)/l and its vertices' mitres
/// (n1 + n2)/(1 + n1.n2), vertex k between edges k - 1 and k; the polygon's
/// last point is its first.
struct Poly {
    p: Vec<Sp>,
    n: Vec<(String, String, P)>,
    m: Vec<(String, String, P)>,
}

fn poly(c: &mut Ctx, name: &str, p: Vec<Sp>) -> Poly {
    let e = p.len() - 1;
    let mut n = vec![];
    for k in 0..e {
        let (a, b) = (&p[k], &p[k + 1]);
        let l = format!("sqrt(({} - {})^2 + ({} - {})^2)", b.x, a.x, b.y, a.y);
        let (ux, uy) = (b.v.0 - a.v.0, b.v.1 - a.v.1);
        let lv = ux.hypot(uy);
        let nx = c.num(&format!("{name}_n{k}_x"), format!("-({} - {})/({l})", b.y, a.y));
        let ny = c.num(&format!("{name}_n{k}_y"), format!("({} - {})/({l})", b.x, a.x));
        n.push((nx, ny, (-uy / lv, ux / lv)));
    }
    let mut m = vec![];
    for k in 0..e {
        let (n1, n2) = (&n[(k + e - 1) % e], &n[k]);
        let den = format!("(1 + {}*{} + {}*{})", n1.0, n2.0, n1.1, n2.1);
        let dv = 1.0 + n1.2 .0 * n2.2 .0 + n1.2 .1 * n2.2 .1;
        let mx = c.num(&format!("{name}_m{k}_x"), format!("({} + {})/{den}", n1.0, n2.0));
        let my = c.num(&format!("{name}_m{k}_y"), format!("({} + {})/{den}", n1.1, n2.1));
        m.push((mx, my, ((n1.2 .0 + n2.2 .0) / dv, (n1.2 .1 + n2.2 .1) / dv)));
    }
    Poly { p, n, m }
}

/// offset(poly, d) of a closed polygon: each vertex along its mitre, its
/// normals and mitres the same.
fn shift(c: &mut Ctx, name: &str, q: &Poly, d: &str, dv: f64) -> Poly {
    let e = q.p.len() - 1;
    let mut p: Vec<Sp> = (0..e)
        .map(|k| {
            let (a, m) = (&q.p[k], &q.m[k]);
            c.pt(&format!("{name}{k}"), format!("{} + ({d})*{}", a.x, m.0), format!("{} + ({d})*{}", a.y, m.1), (a.v.0 + dv * m.2 .0, a.v.1 + dv * m.2 .1))
        })
        .collect();
    p.push(p[0].clone());
    Poly { p, n: q.n.clone(), m: q.m.clone() }
}

fn length(p: &[Sp]) -> f64 {
    p.windows(2).map(|w| (w[1].v.0 - w[0].v.0).hypot(w[1].v.1 - w[0].v.1)).sum()
}

/// away(): the offset that makes it bigger, or smaller.
fn away(c: &mut Ctx, name: &str, q: &Poly, d: &str, dv: f64, bigger: bool) -> Poly {
    let lens = |s: f64| -> f64 {
        let pts: Vec<Sp> = (0..q.p.len())
            .map(|k| {
                let k = k % (q.p.len() - 1);
                Sp { x: String::new(), y: String::new(), v: (q.p[k].v.0 + s * dv * q.m[k].2 .0, q.p[k].v.1 + s * dv * q.m[k].2 .1) }
            })
            .collect();
        length(&pts)
    };
    let sg = if (lens(1.0) > lens(-1.0)) == bigger { 1.0 } else { -1.0 };
    shift(c, name, q, &format!("{}*({d})", nn(sg)), sg * dv)
}

/// The page's up(): (x, y) to (-y, -x).
fn up(p: &Sp) -> [String; 2] {
    [format!("-({})", p.y), format!("-({})", p.x)]
}

/// How far `got` is from the generator's `want`, worst point.
fn worst(got: &[[f64; 2]], want: &[P]) -> f64 {
    if got.len() != want.len() {
        return f64::INFINITY;
    }
    got.iter().zip(want).map(|(g, w)| (g[0] - w.0).hypot(g[1] - w.1)).fold(0.0, f64::max)
}

pub fn script(l: &Lyre) -> R<String> {
    let b = &l.b;
    let t = b.thick;
    let gm = l.geo();
    let mut c = Ctx { s: Script::new(), k: 0 };
    let s = &mut c.s;
    s.len("length", format!("{} mm", nn(LENGTH)), "the outer outline, end to end");
    s.num("width_k", nn(WIDTH_OF_LENGTH), "its width per mm of length, from the sketch");
    s.len("bore", format!("{} mm", nn(BORE)), "the duct's depth everywhere, and its width over the arch and down the sides");
    s.len("ply", format!("{} mm", nn(t)), "the ply: every wall, cheek and lamination");
    s.num("hole_straight", nn(HOLE_STRAIGHT), "the hole's straight sides, per mm of its half-width");
    s.num("hole_corner", nn(HOLE_CORNER), "its bottom corners' radius, the same");
    s.num("hole_bump", nn(HOLE_BUMP), "the bump between them, how high it rises, the same");
    s.len("kerf", format!("{} mm", nn(KERF)), "the beam, measured");
    s.len("fit", format!("{} mm", nn(l.fit)), "the kerf the joints are drawn as if cut at: the generator's --fit=");
    s.len("cheek_past", "(ply - (fit - kerf))/2 - kerf/2", "read only: the cheeks' rims past each wall's centreline, its mortice's half-width");
    s.num("knot_up", nn(KNOT_UP), "the knot's centre, its share of the way from the outer wall's far end to the bump");
    s.len("hitch_w", format!("{} mm", nn(HITCH_WIDTH)), "the hitch block, in from the outer wall's face");
    s.num("pins", nn(PINS as f64), "pins across, the hitch's and the tuning's");
    s.len("pin_spacing", format!("{} mm", nn(PIN_SPACING)), "their spacing");
    s.len("tune_extra", format!("{} mm", nn(TUNE_EXTRA)), "the tuning block past the outermost pin, round the arch");
    s.angle("facet", format!("{} deg", nn(FACET)), "the most an arc turns per panel; the tuning block ends at a facet's middle");
    s.len("g_A", "length*width_k/2", "read only: the outer outline's half-width");
    s.len("g_band", "bore + 2*ply", "read only: the band over the arch, wall to wall");
    s.len("g_a", "g_A - g_band", "read only: the hole's half-width");
    s.len("g_Hs", "length - 2*g_A", "read only: the outer outline's straight sides");
    s.len("g_hs", "hole_straight*g_a", "read only: the hole's");
    s.len("g_rc", "hole_corner*g_a", "read only: its bottom corners' radius");
    s.len("g_sag", "hole_bump*g_a", "read only: the bump's rise");
    s.len("g_cx", "g_a - g_rc", "read only: the corners' centres, either side");
    s.len("g_apex", "-g_hs - g_rc + g_sag", "read only: the bump's top");
    s.len("g_d", "g_apex + g_hs", "read only");
    s.len("g_Rb", "(g_cx^2 + g_d^2 - g_rc^2)/(2*(g_d + g_rc))", "read only: the bump's radius, tangent to both corners");
    s.len("g_by", "g_apex - g_Rb", "read only: the bump's centre");
    // the outlines moved ply into the duct: the faces
    let off = t;
    s.len("o_r", "g_A - ply", "read only: the outer face's ends' radius");
    s.len("h_ri", "g_a + ply", "read only: the hole face's arch");
    s.len("h_rc", "g_rc + ply", "read only: its corners");
    s.len("h_rb", "g_Rb - ply", "read only: its bump");
    s.angle("th_l", "atan2(g_by + g_hs, g_cx)", "read only: the left corner to the bump's centre");
    s.angle("th_r", "atan2(g_by + g_hs, -g_cx)", "read only");
    s.angle("ph_l", "atan2(-g_hs - g_by, -g_cx)", "read only: the bump to the left corner's centre");
    s.angle("ph_r", "atan2(-g_hs - g_by, g_cx)", "read only");
    let deg = |r: f64| r.to_degrees();
    // a run of points (x, y) of the outline, before lay()
    let mut raw_o: Vec<(String, String, P)> = vec![];
    let ro = gm.A - off;
    for (cy, cyv, a0) in [("0 mm", 0.0, 0.0), ("-g_Hs", -gm.Hs, PI)] {
        let n = facets(PI, ro);
        let step = PI / n as f64;
        for i in 0..n {
            let a = a0 + step * (i as f64 + 0.5);
            let (kx, ky) = (a.cos() / (step / 2.0).cos(), a.sin() / (step / 2.0).cos());
            raw_o.push((format!("{}*o_r", nn(kx)), format!("{cy} + {}*o_r", nn(ky)), (kx * ro, cyv + ky * ro)));
        }
    }
    raw_o.push(raw_o[0].clone());
    // the hole: arc() runs joined, each run's first point dropped after the first run
    let (ri, rcc, rb) = (gm.a + off, gm.rc + off, gm.Rb - off);
    let (bxv, byv) = gm.bc;
    let (hs, cxv) = (gm.hs, gm.cx);
    let tl = (byv + hs).atan2(cxv);
    let tr = (byv + hs).atan2(-cxv);
    let pl = (-hs - byv).atan2(-cxv);
    let pr = (-hs - byv).atan2(cxv);
    // (centre x, centre y, radius, a0, a1) as expressions and values
    type Arc<'a> = (&'a str, &'a str, &'a str, String, String, f64, f64, f64, f64, f64);
    let arcs: Vec<Arc> = vec![
        ("0 mm", "0 mm", "h_ri", "0 deg".into(), "180 deg".into(), 0.0, 0.0, ri, 0.0, PI),
        ("-g_cx", "-g_hs", "h_rc", "180 deg".into(), "360 deg + th_l".into(), -cxv, -hs, rcc, PI, 2.0 * PI + tl),
        ("0 mm", "g_by", "h_rb", "ph_l".into(), "ph_r".into(), bxv, byv, rb, pl, pr),
        ("g_cx", "-g_hs", "h_rc", "th_r".into(), "0 deg".into(), cxv, -hs, rcc, tr, 0.0),
    ];
    let mut runs: Vec<Vec<(String, String, P)>> = vec![vec![("h_ri".into(), "-g_hs".into(), (ri, -hs)), ("h_ri".into(), "0 mm".into(), (ri, 0.0))]];
    for (k, (cx, cy, r, a0, a1, cxn, cyn, rn, a0n, a1n)) in arcs.into_iter().enumerate() {
        let n = facets(a1n - a0n, rn);
        let mut run = vec![];
        for i in 0..=n {
            let f = i as f64 / n as f64;
            let tv = a0n + (a1n - a0n) * f;
            let te = format!("(({a0}) + (({a1}) - ({a0}))*{})", nn(f));
            run.push((format!("{cx} + {r}*cos({te})"), format!("{cy} + {r}*sin({te})"), (cxn + rn * tv.cos(), cyn + rn * tv.sin())));
        }
        runs.push(run);
        if k == 0 {
            runs.push(vec![("-h_ri".into(), "0 mm".into(), (-ri, 0.0)), ("-h_ri".into(), "-g_hs".into(), (-ri, -hs))]);
        }
    }
    runs.push(vec![("h_ri".into(), "-g_hs".into(), (ri, -hs))]);
    let mut raw_h: Vec<(String, String, P)> = vec![];
    for run in runs {
        let skip = if raw_h.is_empty() { 0 } else { 1 };
        raw_h.extend(run.into_iter().skip(skip));
    }
    let nh = raw_h.len();
    raw_h[nh - 1] = raw_h[0].clone();
    let _ = deg;
    // lay(): (x, y) to (-y, x)
    let lay_pts = |c: &mut Ctx, name: &str, raw: &[(String, String, P)]| -> Poly {
        let e = raw.len() - 1;
        let mut p: Vec<Sp> = raw[..e].iter().enumerate().map(|(k, (x, y, v))| c.pt(&format!("{name}{k}"), format!("-({y})"), x.clone(), (-v.1, v.0))).collect();
        p.push(p[0].clone());
        poly(c, name, p)
    };
    let face_o = lay_pts(&mut c, "fo", &raw_o);
    let face_i = lay_pts(&mut c, "fi", &raw_h);
    // check the faces against the generator's
    let (o, i) = l.outlines(t)?;
    let lv = |v: &[Sp]| v.iter().map(|q| [q.v.0, q.v.1]).collect::<Vec<_>>();
    let (wo, wi) = (worst(&lv(&face_o.p), &lay(&o)), worst(&lv(&face_i.p), &lay(&i)));
    if wo > 1e-9 || wi > 1e-9 {
        return Err(format!("the design's faces are {wo:e} and {wi:e} off the generator's"));
    }
    // the walls' centrelines, and the cheeks' rims past them
    let outer = away(&mut c, "co", &face_o, "ply/2", t / 2.0, true);
    let hole = away(&mut c, "ci", &face_i, "ply/2", t / 2.0, false);
    let cp = b.cheek_off() - b.wall_off();
    let rim_o = away(&mut c, "ro", &outer, "cheek_past", cp, true);
    let rim_i = away(&mut c, "ri", &hole, "cheek_past", cp, false);
    let (gouter, ghole, parts) = l.build()?;
    if worst(&lv(&outer.p), &gouter) > 1e-9 || worst(&lv(&hole.p), &ghole) > 1e-9 {
        return Err("the design's wall centrelines are off the generator's".into());
    }
    // the panels: inner wall first, then outer, each edge less its mitres' trims
    struct Pn {
        name: String,
        mid: (String, String),
        len: String,
        u: (String, String),
        nrm: (String, String),
        v: (P, f64),
    }
    let mut pans: Vec<Pn> = vec![];
    for (wall, q, tag) in [("inner", &hole, "pi"), ("outer", &outer, "po")] {
        let e = q.p.len() - 1;
        let trims: Vec<(String, f64)> = (0..e)
            .map(|k| {
                let (n1, n2) = (&q.n[(k + e - 1) % e], &q.n[k]);
                let cr = format!("abs({}*{} - {}*{})", n1.0, n2.1, n1.1, n2.0);
                let dt = format!("(1 + {}*{} + {}*{})", n1.0, n2.0, n1.1, n2.1);
                let v = t / 2.0 * (n1.2 .0 * n2.2 .1 - n1.2 .1 * n2.2 .0).abs() / (1.0 + n1.2 .0 * n2.2 .0 + n1.2 .1 * n2.2 .1);
                (c.len(&format!("{tag}_e{k}"), format!("ply/2*{cr}/{dt}")), v)
            })
            .collect();
        for k in 0..e {
            let (a, bb) = (&q.p[k], &q.p[k + 1]);
            let (e0, e1) = (&trims[k], &trims[(k + 1) % e]);
            let nk = &q.n[k];
            let (ux, uy) = (nk.1.clone(), format!("-{}", nk.0));
            let lenv = (bb.v.0 - a.v.0).hypot(bb.v.1 - a.v.1) - e0.1 - e1.1;
            let len = c.len(&format!("{tag}{k}_len"), format!("sqrt(({} - {})^2 + ({} - {})^2) - ({}) - ({})", bb.x, a.x, bb.y, a.y, e0.0, e1.0));
            let sh = format!("(({}) - ({}))/2", e0.0, e1.0);
            let (uxv, uyv) = (nk.2 .1, -nk.2 .0);
            let mv = ((a.v.0 + bb.v.0) / 2.0 + (e0.1 - e1.1) / 2.0 * uxv, (a.v.1 + bb.v.1) / 2.0 + (e0.1 - e1.1) / 2.0 * uyv);
            let mid = c.pt(&format!("{tag}{k}_mid"), format!("({} + {})/2 + {sh}*({ux})", a.x, bb.x), format!("({} + {})/2 + {sh}*({uy})", a.y, bb.y), mv);
            pans.push(Pn { name: format!("{wall} panel {k}"), mid: (mid.x, mid.y), len, u: (ux, uy), nrm: (nk.0.clone(), nk.1.clone()), v: (mv, lenv) });
        }
    }
    if pans.len() != parts.len() || pans.iter().zip(&parts).any(|(p, q)| (p.v.0 .0 - q.mid.0).hypot(p.v.0 .1 - q.mid.1) > 1e-9 || (p.v.1 - q.len).abs() > 1e-9) {
        return Err("the design's panels are off the generator's".into());
    }
    for (p, q) in pans.iter_mut().zip(&parts) {
        p.name = format!("{} panel {}", q.wall, q.tag);
    }
    let rims = contours(&l.cheek(&gouter, &ghole));
    if worst(&lv(&rim_o.p), &rims[0]) > 1e-9 || worst(&lv(&rim_i.p), &rims[1]) > 1e-9 {
        return Err("the design's rims are off the generator's".into());
    }
    // the knot, at its share of the resonator
    c.s.len("k_bottom", "-(g_Hs + g_A - ply)", "read only: the outer wall's face, at the far end");
    c.s.len("k_top", "-g_hs - g_rc + g_sag - ply", "read only: the hole's wall face, at the bump");
    c.s.len("k_y", "k_bottom + knot_up*(k_top - k_bottom)", "read only: the knot's centre on the long axis");
    let (kc, _) = knot_paths()?;
    // hitch block: the default build's facets round the axis at the resonator end
    let (gedge, ghitch) = l.hitch_block(&lay(&o));
    let fo: Vec<P> = face_o.p.iter().map(|q| q.v).collect();
    let nf = fo.len() - 1;
    let mut best: Option<(f64, usize)> = None;
    for i in 0..nf {
        if (fo[i].1 > 0.0) != (fo[i + 1].1 > 0.0) {
            let v = fo[i].0.max(fo[i + 1].0);
            if best.map(|(bv, _)| v >= bv).unwrap_or(true) {
                best = Some((v, i));
            }
        }
    }
    let cidx = best.unwrap().1 as i64;
    let reach = HITCH_FACETS as f64 / 2.0;
    let whole = (reach - 0.5).floor() as i64;
    let frac = reach - 0.5 - whole as f64;
    let wrap = |i: i64| i.rem_euclid(nf as i64) as usize;
    let at = |c: &mut Ctx, name: &str, i: i64, f: f64| -> Sp {
        let (a, bq) = (&face_o.p[wrap(i)], &face_o.p[wrap(i + 1)]);
        c.pt(name, format!("{} + ({} - {})*{}", a.x, bq.x, a.x, nn(f)), format!("{} + ({} - {})*{}", a.y, bq.y, a.y, nn(f)), (a.v.0 + (bq.v.0 - a.v.0) * f, a.v.1 + (bq.v.1 - a.v.1) * f))
    };
    let (lo, hi) = (cidx - whole - 1, cidx + whole + 1);
    // the edge, each point with the face edge before and after it
    let mut edge: Vec<(Sp, Option<usize>)> = vec![(at(&mut c, "hb_start", lo, 1.0 - frac), None)];
    for i in lo..hi {
        edge.push((face_o.p[wrap(i + 1)].clone(), Some(wrap(i + 1))));
    }
    edge.push((at(&mut c, "hb_end", hi, frac), None));
    // its offset by hitch_w, into the duct
    let ne = edge.len();
    let side = |sg: f64, c: &mut Ctx, name: &str| -> Vec<Sp> {
        (0..ne)
            .map(|j| {
                let (p, v) = &edge[j];
                let (mx, my, mv) = match v {
                    Some(fv) => face_o.m[*fv].clone(),
                    None => face_o.n[wrap(if j == 0 { lo } else { hi })].clone(),
                };
                c.pt(&format!("{name}{j}"), format!("{} + {}*hitch_w*{mx}", p.x, nn(sg)), format!("{} + {}*hitch_w*{my}", p.y, nn(sg)), (p.v.0 + sg * HITCH_WIDTH * mv.0, p.v.1 + sg * HITCH_WIDTH * mv.1))
            })
            .collect()
    };
    let probe = {
        let j = ne / 2;
        let (p, v) = &edge[j];
        let mv = match v {
            Some(fv) => face_o.m[*fv].2,
            None => face_o.n[wrap(if j == 0 { lo } else { hi })].2,
        };
        (p.v.0 + HITCH_WIDTH * mv.0, p.v.1 + HITCH_WIDTH * mv.1)
    };
    let sg = if in_poly(&fo, probe.0, probe.1) { 1.0 } else { -1.0 };
    let inner = side(sg, &mut c, "hb_in");
    let mut hitch: Vec<Sp> = edge.iter().map(|e| e.0.clone()).collect();
    hitch.extend(inner.iter().rev().cloned());
    let gh: Vec<P> = ghitch[..ghitch.len() - 1].to_vec();
    if worst(&lv(&hitch), &gh) > 1e-9 || gedge.len() != ne {
        return Err("the design's hitch block is off the generator's".into());
    }
    // the tuning block: both faces between the rays at -phi and +phi round the arch
    c.s.len("r_mid", "g_A - ply - bore/2", "read only: the duct's middle round the arch");
    c.s.angle("tune_need", "asin((pins - 1)/2*pin_spacing/r_mid) + tune_extra/r_mid*180 deg/pi", "read only: the block's half-angle, past the outermost pin");
    c.s.angle("tune_phi", "ceil(tune_need/facet - 0.000000001)*facet", "read only: out to the next facet's middle");
    let phi = l.tune_span();
    let fi: Vec<P> = face_i.p.iter().map(|q| q.v).collect();
    let run_of = |c: &mut Ctx, f: &Poly, fv: &[P], name: &str| -> R<Vec<Sp>> {
        let ray = |c: &mut Ctx, sgn: f64, nm: &str| -> R<Sp> {
            let (dx, dy) = (-(sgn * phi).cos(), (sgn * phi).sin());
            let (dxe, dye) = ("(-cos(tune_phi))".to_string(), format!("({}*sin(tune_phi))", nn(sgn)));
            for (k, w) in fv.windows(2).enumerate() {
                let (a, bq) = (w[0], w[1]);
                let (ux, uy) = (bq.0 - a.0, bq.1 - a.1);
                let den = dx * uy - dy * ux;
                if den.abs() < 1e-12 {
                    continue;
                }
                let tt = (dy * a.0 - dx * a.1) / den;
                if (-1e-12..=1.0 + 1e-12).contains(&tt) {
                    let q = (a.0 + ux * tt, a.1 + uy * tt);
                    if q.0 * dx + q.1 * dy > 0.0 {
                        let (pa, pb) = (&f.p[k], &f.p[k + 1]);
                        let te = format!("(({dye})*{} - ({dxe})*{})/(({dxe})*({} - {}) - ({dye})*({} - {}))", pa.x, pa.y, pb.y, pa.y, pb.x, pa.x);
                        return Ok(c.pt(nm, format!("{} + ({} - {})*{te}", pa.x, pb.x, pa.x), format!("{} + ({} - {})*{te}", pa.y, pb.y, pa.y), q));
                    }
                }
            }
            Err("no face for the tuning block's end".into())
        };
        let aa = |q: P| q.1.atan2(-q.0);
        let mut inside: Vec<Sp> = f.p[..f.p.len() - 1].iter().filter(|q| aa(q.v).abs() < phi && q.v.0 < 0.0).cloned().collect();
        inside.sort_by(|p, q| aa(p.v).partial_cmp(&aa(q.v)).unwrap());
        let mut e = vec![ray(c, -1.0, &format!("{name}_lo"))?];
        e.extend(inside);
        e.push(ray(c, 1.0, &format!("{name}_hi"))?);
        let mut out: Vec<Sp> = vec![];
        for (k, q) in e.iter().enumerate() {
            if k == 0 || (q.v.0 - e[k - 1].v.0).hypot(q.v.1 - e[k - 1].v.1) > 1e-9 {
                out.push(q.clone());
            }
        }
        Ok(out)
    };
    let te_o = run_of(&mut c, &face_o, &fo, "tb_o")?;
    let te_i = run_of(&mut c, &face_i, &fi, "tb_i")?;
    let mut tune: Vec<Sp> = te_o.clone();
    tune.extend(te_i.iter().rev().cloned());
    let (_, gt) = l.tuning_block(&lay(&o), &lay(&i))?;
    if worst(&lv(&tune), &gt[..gt.len() - 1]) > 1e-9 {
        return Err("the design's tuning block is off the generator's".into());
    }
    // the parts: the cheeks, the blocks' laminations, the panels
    let s = &mut c.s;
    let ring = |pts: &[Sp]| -> Vec<[String; 2]> { pts.iter().map(up).collect() };
    let open = |p: &Poly| p.p[..p.p.len() - 1].to_vec();
    for (name, z, knot) in [("front cheek", "bore/2", true), ("back cheek", "-bore/2 - ply", false)] {
        let sk = s.sketch("XY", Some(z), &format!("{name} sketch"));
        s.poly(&ring(&open(&rim_o)));
        s.poly(&ring(&open(&rim_i)));
        if knot {
            for lp in &kc {
                let mut lp = lp.clone();
                if lp.len() > 1 && (lp[0].0 - lp[lp.len() - 1].0).hypot(lp[0].1 - lp[lp.len() - 1].1) < 1e-9 {
                    lp.pop();
                }
                // up((x + kx, y)) with kx = -k_y
                let pts: Vec<[String; 2]> = lp.iter().map(|&(x, y)| [format!("{} mm", nn(-y)), format!("{} mm + k_y", nn(-x))]).collect();
                s.poly(&pts);
            }
        }
        s.finish();
        s.extrude(&sk, "ply", false, "new", &[], name);
    }
    for (what, pts, layers) in [("hitch block", &hitch, HITCH_LAYERS), ("tuning block", &tune, TUNE_LAYERS)] {
        for k in 0..layers {
            let name = format!("{what} layer {}", k + 1);
            let sk = s.sketch("XY", Some(&format!("bore/2 - {}*ply", k + 1)), &format!("{name} sketch"));
            s.poly(&ring(pts));
            s.finish();
            s.extrude(&sk, "ply", false, "new", &[], &name);
        }
    }
    for p in &pans {
        let sk = s.sketch("XY", Some("-bore/2"), &format!("{} sketch", p.name));
        let corner = |a: f64, bq: f64| -> [String; 2] {
            let x = format!("{} + {}*({})/2*({}) + {}*ply/2*({})", p.mid.0, nn(a), p.len, p.u.0, nn(bq), p.nrm.0);
            let y = format!("{} + {}*({})/2*({}) + {}*ply/2*({})", p.mid.1, nn(a), p.len, p.u.1, nn(bq), p.nrm.1);
            [format!("-({y})"), format!("-({x})")]
        };
        s.poly(&[corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0), corner(-1.0, 1.0)]);
        s.finish();
        s.extrude(&sk, "bore", false, "new", &[], &p.name);
    }
    s.measure();
    Ok(c.s.text())
}
