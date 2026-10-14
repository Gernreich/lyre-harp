#!/usr/bin/env python3
"""A lyre-harp frame as one closed duct whose width changes round the loop.

    python3 lyre_harp.py --no-write
    python3 lyre_harp.py --out=DIR/lyre-harp-....svg
    python3 lyre_harp.py --drawing=DIR/lyre-harp-drawing.svg
    python3 lyre_harp.py --render=DIR/lyre-harp-....html

ribbon_bore.py builds every duct as two walls offset a fixed bore either side of
ONE centreline, so its section is the same all the way round. This one is not.
The walls are two independent closed outlines, drawn from the author's sketch
(a rough lyre-harp.svg, since deleted) as redrawn and approved on 2026-09-16:

  * the OUTER outline, a stadium 400mm long -- a half-circle at each end and
    two parallel sides;
  * the HOLE, where the strings go: a half-circle concentric with the outer
    one, parallel sides, and a bottom of two tangent corners with a small bump
    rising into the hole between them.

Between them runs the air. Over the arch and down the parallel sides the two
walls stand a bore apart, so the duct there is BORE x BORE, 30 x 30mm. Below
the hole the walls part: the duct is still BORE deep, cheek to cheek, but as
wide as the gap between the hole's bottom and the outer bottom -- the
resonator, about 200mm across.

Everything that is a part -- the panel with its teeth, the mortice, the
engraved number, the packing, the sheet and its colours, the ring cheek cut as
two contours -- is ribbon_bore's own, imported and not copied. What is new is
only what a single centreline could not describe: the two outlines, which way
is out from each, and the checks that assumed a constant bore.
"""
import math
import os
import re
import sys

# ribbon_bore.py is the swept-curve trumpet generator, in the swept-curve
# repository cloned beside this one. This file lived beside it until 2026-10-14.
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                '..', 'swept-curve'))
import ribbon_bore as B                                        # noqa: E402

LENGTH = 400.0       # outer outline, end to end
# KERF, the full width the laser takes out, for THIS instrument only. 0.13 is
# the MEASURED beam, ribbon_bore.py's own figure from 2026-09-09. It is not a
# fit knob and nothing here may move it to chase a joint.
#
# FIT is the fit knob, and it is the whole history of this instrument in one
# number. The test came off the bed with LOOSE FINGER JOINTS, so the kerf was
# raised to 0.17, then to 0.20 (2026-09-20, both on the author's instruction) --
# which tightened the joints because an overstated kerf draws tabs fat and
# mortices thin, but it also drew every OTHER line in the design 0.07 out,
# outlines oversize and holes undersize, the band, the duct and the knot cuts
# with them. So the kerf goes back to the truth and the interference moves to
# the two constants that exist for it, leaving the joints exactly where 0.20 put
# them. FIT is the kerf the JOINTS are cut as if, and nothing else reads it.
#
# 0.20 -> 0.25 the same day, on the author's instruction: they are after a
# FRICTION FIT, one that needs a rubber mallet to close, and are walking the
# test piece up to it. That is +0.19mm of interference along the tooth and
# +0.12 across the ply, against +0.09 and +0.07 at 0.20. Use --fit= to try a
# value; move this default when one is to be cut, so the sheets on disk and
# the number in the source never disagree about what is on the bench.
KERF = 0.13
FIT = 0.25
# Why those two lines below, and not one. Write D = FIT - KERF. Along the tooth
# BOTH sides of the joint are drawn, so overstating the kerf fattens the tab by
# D and thins the notch by D: 2D of interference. Across the ply only the notch
# is drawn -- the other half of that joint is the plywood, which no kerf can
# grow -- so the same D buys D. One lever cannot pay 2:1 and 1:1 at once.
# SLOT_TIGHTEN comes off the notch in both directions, so it settles the depth;
# the width is then short by D, and half of that comes off the clearance a side.
# It takes PLAY NEGATIVE, at -0.01, and that is the honest reading and not a
# trick: there is no clearance left in this joint, there is interference, and a
# negative clearance per side is what that sentence means in a number.
# Verified against the 0.20 sheets before the change: width +0.090mm and depth
# +0.070mm of interference both ways, and the mortice even lands on the same
# 2.80mm drawn depth.
# Functions, not constants, because --fit= moves FIT at run time and a value
# worked out at import would go on answering for the old one. _PLAY0 is caught
# at import instead, before build() overwrites B.PLAY_UNMEASURED with the
# figure below -- read from ribbon_bore rather than written out again, since the
# clearance this reduces is ITS number and a copied 0.025 would go on saying
# 0.025 after the original moved.
_PLAY0 = B.PLAY_UNMEASURED


def tighten():
    """Off the notch, both directions. 0.07 at FIT 0.20."""
    return FIT - KERF


def fit_play():
    """Clearance a side, negative once the joint is an interference fit."""
    return _PLAY0 - tighten() / 2
BORE = 30.0          # duct depth everywhere; duct width over the arch and sides
# The approved drawing, as proportions. Width against length is the drawing's
# 391 : 831. The hole's bottom is scaled on the hole's own half-width, not on
# the length: the band is a fixed 36mm, so the hole does not shrink in step with
# the outline, and scaling its corners on the length left them 10mm apart with
# no room for the bump between them.
WIDTH_OF_LENGTH = 391.0 / 830.9
HOLE_STRAIGHT, HOLE_CORNER, HOLE_BUMP = 175.3 / 159.5, 98.9 / 159.5, 10.3 / 159.5
FACET = 15.0         # the most an arc turns per panel
MIN_PANEL = 14.0     # but no finer than this along an arc: a tooth needs 10
NARROW = True
OUT = None
# The sound hole: a knotwork rosette from the knotwork-soundholes repository,
# used as its generator wrote it. Its centre sits on the long axis, KNOT_UP of
# the way from the bottom of the resonator's air to the top of it -- the outer
# wall's face at the far end, and the hole wall's face at the bump. It goes in
# ONE cheek, the front, so the two cheeks stop being one part cut twice.
KNOT = os.path.expanduser('~/LaserMadeMusic/GIT/knotwork-soundholes/'
                          '2-lead_7-bight_knot_radius30mm.svg')
KNOT_UP = 2.0 / 3.0
# The hitch-pin block: the strings' loop ends go over pins driven through the
# bottom wall, and 3mm of ply will not hold a pin under string tension, so a
# block is laminated inside the bottom end for the pins to bite into. Each
# layer is one flat part; HITCH_LAYERS of them stack HITCH_LAYERS x THICK deep.
# In plan the block sits against the outer wall's face round the bottom end,
# HITCH_WIDTH wide from that face towards the middle, and runs HITCH_FACETS
# facets' worth along it, centred on the long axis.
HITCH_FACETS, HITCH_WIDTH, HITCH_LAYERS = 8, 30.0, 6
# The hitch pins, one per string, driven through the front cheek into the
# block. The strings run parallel to the long axis, so the pins stand
# PIN_SPACING apart across it, centred on it. Each sits half the block's
# width in from the wall, so the row follows the curve of the bottom end.
# They are ENGRAVED on the front cheek -- a PIN_DIA circle and a cross through
# it -- as drilling marks, not cut: a hole cut in the cheek alone would not
# guide a drill on into the block.
PINS, PIN_SPACING, PIN_DIA = 7, 16.0, 2.0
# The tuning-pin block, the hitch block's opposite number at the arch. It fills
# the duct there -- BORE wide, touching the outer wall's face and the string
# hole's -- and carries one tuning pin per string, each in line across the
# instrument with its hitch pin below. It runs TUNE_EXTRA past the outermost
# pin at least, for wood to hold a pin that is turned; the span is then taken
# out to the next facet midpoint, as the hitch block's is.
TUNE_LAYERS, TUNE_EXTRA = 6, 15.0
# --test cuts a piece of the arch to try the joints on before committing a
# whole frame: the string-hole wall's panels TEST_FROM..TEST_TO, every outer
# panel beside them, and a sector of the cheek for each side carrying their
# mortices. TEST_MARGIN of cheek is left beyond the outermost mortice so the
# rim has something to hold on to.
TEST_FROM, TEST_TO, TEST_MARGIN = '4', 'A', 10.0
# --ladder: the same joint cut at each of these, to find the fit by hand
# instead of by arithmetic. Spanning the crossover at 0.18, where the joint has
# neither clearance nor interference, so the sheet holds rungs either side of
# the point where it stops going together by thumb.
LADDER = (0.15, 0.18, 0.21, 0.24, 0.27)
# --kerf-test: the machine's own number, not this instrument's. Two ways of
# asking on one sheet. KERF_CUTS slices give KERF_CUTS + 1 pieces and the
# pieces are short by KERF_CUTS kerfs all together, so the caliper's own error
# divides by that many -- which is the whole point of slicing ten times rather
# than measuring one cut. KERF_SQ is the cross-check: a square hole and the
# piece that fell out of it differ by exactly two kerfs, one off each edge.
KERF_LEN, KERF_WIDE, KERF_CUTS = 120.0, 15.0, 10
KERF_SQ = 30.0
LADDER_MARGIN = 10.0     # cheek left past the outermost mortice on a coupon
LADDER_COUPON_H = 16.0   # across the coupon; the mortices run down its middle


def geometry():
    """The design numbers, y up, the arch centre at the origin."""
    t = B.THICK
    band = BORE + 2 * t
    A = LENGTH * WIDTH_OF_LENGTH / 2            # outer half-width
    a = A - band                                # hole half-width
    Hs = LENGTH - 2 * A                         # outer straight sides
    hs, rc, sag = HOLE_STRAIGHT * a, HOLE_CORNER * a, HOLE_BUMP * a
    cx = a - rc                                 # corner centres at +-cx, -hs
    apex = -hs - rc + sag
    d = apex + hs
    # the bump: a circle below the hole, tangent to both corners from outside
    Rb = (cx * cx + d * d - rc * rc) / (2 * (d + rc))
    if Rb <= 0:
        raise ValueError('the hole corners leave no room for the bump between '
                         'them')
    return dict(A=A, a=a, Hs=Hs, hs=hs, rc=rc, cx=cx, sag=sag, Rb=Rb,
                bc=(0.0, apex - Rb), band=band)


def arc(cx, cy, r, a0, a1):
    """Vertices on a circle from angle a0 to a1, both included.

    ON the circle rather than inscribed with tangent chords: nothing here pairs
    a panel on one wall with a panel on the other, so there is no bore to keep
    constant across a facet and the simplest faceting is the right one. As fine
    as FACET, and no finer than MIN_PANEL along the arc.
    """
    sweep = a1 - a0
    n = max(1, math.ceil(abs(math.degrees(sweep)) / FACET))
    n = max(1, min(n, int(abs(sweep) * r // MIN_PANEL)))
    return [(cx + r * math.cos(a0 + sweep * i / n),
             cy + r * math.sin(a0 + sweep * i / n)) for i in range(n + 1)]


def tangent_arc(cx, cy, r, a0, a1):
    """Vertices whose chords TOUCH the circle, for the outer wall's ends.

    arc() puts its vertices on the circle, so every chord cuts inside it by
    r(1 - cos(step/2)). On the hole that moves the wall away from the air and
    costs nothing. On the outer wall it moves the wall INTO the air, and the
    duct over the arch came out 29.72mm against 30. Here each chord is tangent
    at its middle and the vertices stand outside the circle, so the face never
    comes inside the radius the section was drawn at.

    The first and last vertex fall on the tangent lines at a0 and a1, which on
    a stadium are the straight sides themselves, so the sides simply run on
    to meet them and no end vertex is returned.
    """
    sweep = a1 - a0
    n = max(1, math.ceil(abs(math.degrees(sweep)) / FACET))
    n = max(1, min(n, int(abs(sweep) * r // MIN_PANEL)))
    step = sweep / n
    R = r / math.cos(step / 2)
    return [(cx + R * math.cos(a0 + step * (i + 0.5)),
             cy + R * math.sin(a0 + step * (i + 0.5))) for i in range(n)]


def join(*runs):
    out = []
    for run in runs:
        out += run if not out else run[1:]
    return out


def outlines(off):
    """(outer, hole) closed polylines, each moved `off` into the duct.

    off = THICK gives the two faces the air touches, which build() draws first
    and hangs the walls from; off = 0 is the outline of the drawing. The outer
    arcs are tangent_arc(), the hole's arcs arc(). Both close exactly.
    """
    g = geometry()
    A, a, Hs, hs, rc, cx, Rb = (g[k] for k in
                                ('A', 'a', 'Hs', 'hs', 'rc', 'cx', 'Rb'))
    ro = A - off
    top = tangent_arc(0.0, 0.0, ro, 0.0, math.pi)
    outer = top + tangent_arc(0.0, -Hs, ro, math.pi, 2 * math.pi) + [top[0]]

    # The hole grows by `off`: every radius about its own centre, and the
    # bump's the other way, because its centre lies outside the hole. The
    # centres do not move, so the corners stay tangent to the bump: the gap
    # between centres is rc + Rb before and (rc + off) + (Rb - off) after.
    ri, rcc, rb = a + off, rc + off, Rb - off
    bx, by = g['bc']
    theta_l = math.atan2(by + hs, cx)            # left corner -> bump centre
    theta_r = math.atan2(by + hs, -cx)           # right corner -> bump centre
    phi_l = math.atan2(-hs - by, -cx)            # bump -> left corner centre
    phi_r = math.atan2(-hs - by, cx)             # bump -> right corner centre
    hole = join([(ri, -hs), (ri, 0.0)],
                arc(0.0, 0.0, ri, 0.0, math.pi),
                [(-ri, 0.0), (-ri, -hs)],
                arc(-cx, -hs, rcc, math.pi, 2 * math.pi + theta_l),
                arc(bx, by, rb, phi_l, phi_r),
                arc(cx, -hs, rcc, theta_r, 0.0),
                [(ri, -hs)])
    hole[-1] = hole[0]
    return outer, hole


def lay(pts):
    """Long axis along the bed. pack() does not turn a part, and 400 x 188mm
    fits the 580 x 288 usable area only lying down."""
    return [(-y, x) for x, y in pts]


def turn(poly, k):
    """How far a closed wall turns at vertex k; the seam is a vertex too."""
    n = len(poly) - 1
    p0, p1, p2 = poly[(k - 1) % n], poly[k % n], poly[(k + 1) % n]
    h1 = math.atan2(p1[1] - p0[1], p1[0] - p0[0])
    h2 = math.atan2(p2[1] - p1[1], p2[0] - p1[0])
    return abs((h2 - h1 + math.pi) % (2 * math.pi) - math.pi)


def build():
    """The two wall centrelines and every panel on them."""
    B.BORE, B.NARROW, B.PORT, B.PORT_AT = BORE, NARROW, False, None
    B.BURN = KERF
    # The fit, off the notch only. PLAY_BY_BORE is emptied rather than given an
    # entry for 30: it is ribbon_bore's table of what has been cut on the
    # TRUMPET bores, this instrument has measured its own, and leaving the table
    # in place would mean play() answered from whichever of B.BORE happened to
    # be set. Assigning absolutes, not adjusting, so a second build() in one
    # process cannot apply the tightening twice.
    B.SLOT_TIGHTEN = tighten()
    B.PLAY_BY_BORE, B.PLAY_UNMEASURED = {}, fit_play()
    # The FACES are drawn, and the walls follow from them. Drawn the other way
    # round -- wall centrelines on the circles, faces offset from them -- each
    # mitred vertex of the hole's face stood 1.5/cos(step/2) off its wall
    # rather than 1.5, and the duct over the arch was 29.987mm. The face is the
    # section; it is what gets put on the number.
    face_o, face_i = (lay(p) for p in outlines(B.THICK))
    outer = away(face_o, B.THICK / 2, True)
    hole = away(face_i, B.THICK / 2, False)
    parts, seq = [], 0
    for name, poly in (('inner', hole), ('outer', outer)):
        for i, (a, b) in enumerate(zip(poly, poly[1:]), 1):
            # ribbon_bore's trim, for its reason: a panel end is a square cut,
            # and two neighbours meeting at a mitre jam on the concave side
            # unless each is shortened to where their corners just touch
            e0 = B.THICK / 2 * math.tan(turn(poly, i - 1) / 2)
            e1 = B.THICK / 2 * math.tan(turn(poly, i) / 2)
            L = B.seglen(a, b) - e0 - e1
            if L < B.TOOTH + 2 * B.SHOULDER:
                raise ValueError(
                    f'{name} panel {i} is {L:.2f}mm and a tooth needs '
                    f'{B.TOOTH + 2 * B.SHOULDER:g}mm. Raise MIN_PANEL.')
            ang = math.atan2(b[1] - a[1], b[0] - a[0])
            mid = ((a[0] + b[0]) / 2 + (e0 - e1) / 2 * math.cos(ang),
                   (a[1] + b[1]) / 2 + (e0 - e1) / 2 * math.sin(ang))
            # OUT means away from the air. For the outer wall that is out of
            # the outer outline, for the hole's wall it is into the hole. Asked
            # of the polygon, not of which way round it happens to be drawn.
            nx, ny = -math.sin(ang), math.cos(ang)
            probe = B.in_poly(poly, mid[0] + nx, mid[1] + ny)
            if probe != (name == 'inner'):
                nx, ny = -nx, -ny
            seq += 1
            parts.append({'kind': 'panel', 'wall': name, 'n': i, 'len': L,
                          'mid': mid, 'ang': ang, 'out': (nx, ny),
                          'tag': f'{seq:X}'})
    for q in parts:
        q['teeth'] = B.teeth(q['len'])
    return outer, hole, parts


def away(poly, d, bigger):
    """poly offset by d, towards whichever side makes it bigger or smaller."""
    p, q = B.offset(poly, d), B.offset(poly, -d)
    lp = sum(B.seglen(u, v) for u, v in zip(p, p[1:]))
    lq = sum(B.seglen(u, v) for u, v in zip(q, q[1:]))
    return p if (lp > lq) == bigger else q


def cheek(outer, hole):
    """Both rims, as one list the way ribbon_bore.cheek() writes a ring.

    ribbon_bore.contours() splits it back into two loops for cutting, and its
    inside() reads the joined list correctly, so every tool downstream takes
    it unchanged. Each rim stands off its own wall exactly as a ribbon cheek
    does, flush on the mortices under NARROW.
    """
    e = B.cheek_off() - B.wall_off()
    rim_o = away(outer, e, True)
    rim_i = away(hole, e, False)
    rim_o[-1], rim_i[-1] = rim_o[0], rim_i[0]
    return rim_o + rim_i


def faces(outer, hole):
    """The air's own boundary: each wall's face on the duct side."""
    return (away(outer, B.THICK / 2, False), away(hole, B.THICK / 2, True))


def knot_paths():
    """(cut loops, engraved polylines) from the knot file, in its own mm.

    The file is 1 unit = 1mm about the hole's centre. Its black group is the
    waste that drops out, every path closed; its blue group is the crossing
    marks, several runs to a path.
    """
    body = open(KNOT).read()

    def group(name):
        m = re.search(r'<g id="%s"[^>]*>(.*?)</g>' % name, body, re.S)
        if not m:
            raise ValueError(f'{KNOT} has no <g id="{name}">')
        runs = []
        for d in re.findall(r'\bd="([^"]*)"', m.group(1)):
            for sub in re.split(r'(?=M)', d):
                pts = [(float(x), float(y)) for x, y in
                       re.findall(r'(-?[\d.]+)[ ,](-?[\d.]+)', sub)]
                if len(pts) > 1:
                    runs.append((pts, 'Z' in sub))
        return runs
    cut = [p for p, closed in group('cut')]
    if not all(closed for p, closed in group('cut')):
        raise ValueError(f'{KNOT}: a cut path is not closed')
    return cut, [p for p, _ in group('engrave')]


def knot_centre():
    """Where the knot's centre goes, laid down, and the span it was placed in."""
    g = geometry()
    t = B.THICK
    bottom = -(g['Hs'] + g['A'] - t)                 # outer wall face, far end
    top = -g['hs'] - g['rc'] + g['sag'] - t          # hole wall face at the bump
    y = bottom + KNOT_UP * (top - bottom)
    return lay([(0.0, y)])[0], (bottom, top)


def hitch_block(face_o):
    """(edge, outline) of one lamination, laid down, true size.

    `edge` is the run of the outer wall's face the block lies against. The
    bottom end has an ODD number of facets with the middle one centred on the
    axis, so HITCH_FACETS whole facets cannot sit symmetrically under the
    strings when HITCH_FACETS is even. The span is centred on the axis
    instead and ends half way along a facet at each end: HITCH_FACETS - 1
    whole facets and a half at either end. That also keeps both ends of the
    block off the joints between wall panels.
    """
    n = len(face_o) - 1
    # the facet crossing the long axis (y = 0) at the resonator end
    _, c = max((max(face_o[i][0], face_o[i + 1][0]), i) for i in range(n)
               if (face_o[i][1] > 0) != (face_o[i + 1][1] > 0))
    reach = HITCH_FACETS / 2.0                # facets either side of the middle
    whole = int(math.floor(reach - 0.5))      # full facets beyond the middle one
    frac = reach - 0.5 - whole                # what is left, as a fraction

    def at(i, t):
        a, b = face_o[i % n], face_o[(i + 1) % n]
        return (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)
    lo, hi = c - whole - 1, c + whole + 1
    edge = ([at(lo, 1.0 - frac)] + [face_o[(i + 1) % n] for i in range(lo, hi)]
            + [at(hi, frac)])
    # an odd count ends the span ON a vertex, which the list above then holds
    # twice, and a zero-length run has no direction to offset along
    edge = [q for k, q in enumerate(edge)
            if k == 0 or B.seglen(edge[k - 1], q) > 1e-9]
    # inwards is whichever offset lands in the air
    inner = B.offset(edge, HITCH_WIDTH)
    probe = inner[len(inner) // 2]
    if not B.in_poly(face_o, *probe):
        inner = B.offset(edge, -HITCH_WIDTH)
    outline = edge + inner[::-1] + [edge[0]]
    return edge, outline


def pin_spots(face_o):
    """Where each hitch pin goes, laid down: across the axis, on the block's
    mid-line, half HITCH_WIDTH in from the wall face."""
    edge, _ = hitch_block(face_o)
    mid = B.offset(edge, HITCH_WIDTH / 2)
    if not B.in_poly(face_o, *mid[len(mid) // 2]):
        mid = B.offset(edge, -HITCH_WIDTH / 2)
    spots = []
    for k in range(PINS):
        u = (k - (PINS - 1) / 2) * PIN_SPACING      # across the axis
        for a, b in zip(mid, mid[1:]):
            if (a[1] - u) * (b[1] - u) <= 0 and a[1] != b[1]:
                t = (u - a[1]) / (b[1] - a[1])
                spots.append((a[0] + (b[0] - a[0]) * t, u))
                break
        else:
            raise ValueError(
                f'hitch pin {k + 1}, {u:g}mm off the axis, falls beyond the '
                f'block, which does not reach that far across. Fewer pins, '
                f'closer spacing, or more HITCH_FACETS.')
    return spots


def pin_marks(face_o, dx=0.0, dy=0.0):
    """The engraved drilling marks: a PIN_DIA circle and a cross through it."""
    r, arm = PIN_DIA / 2, PIN_DIA
    out = []
    for x, y in pin_spots(face_o):
        x, y = x + dx, y + dy
        ring = [(x + r * math.cos(2 * math.pi * i / 24),
                 y + r * math.sin(2 * math.pi * i / 24)) for i in range(25)]
        out.append(B.path(ring, close=False))
        out.append(B.path([(x - arm, y), (x + arm, y)], close=False))
        out.append(B.path([(x, y - arm), (x, y + arm)], close=False))
    return out


def arch_angle(p):
    """Where a point stands round the arch: 0 on the long axis, + towards +y.

    Laid down, the arch's centre is the origin and the arch itself is at
    negative x, so the axis points that way.
    """
    return math.atan2(p[1], -p[0])


def ray_hit(poly, phi):
    """Where the ray from the arch's centre at angle phi crosses a face."""
    dx, dy = -math.cos(phi), math.sin(phi)
    for a, b in zip(poly, poly[1:]):
        ux, uy = b[0] - a[0], b[1] - a[1]
        den = dx * uy - dy * ux
        if abs(den) < 1e-12:
            continue
        t = (dy * a[0] - dx * a[1]) / den
        if -1e-12 <= t <= 1 + 1e-12:
            q = (a[0] + ux * t, a[1] + uy * t)
            if q[0] * dx + q[1] * dy > 0:          # ahead of the centre
                return q
    raise ValueError(f'no face at {math.degrees(phi):.1f} degrees round the arch')


def tune_span():
    """Half the block's angle: past the outermost pin, out to a facet mid."""
    g = geometry()
    r_mid = g['A'] - B.THICK - BORE / 2
    u = (PINS - 1) / 2 * PIN_SPACING
    need = math.asin(u / r_mid) + TUNE_EXTRA / r_mid
    step = math.radians(FACET)
    return math.ceil(need / step - 1e-9) * step


def tuning_block(face_o, face_i):
    """One lamination of the tuning-pin block, laid down, true size.

    Both edges are the walls' own faces over the span, so the block touches
    the outer wall and the string hole's wall and is the duct's width between
    them. The ends are radial, cut where the span's rays cross each face.
    """
    phi = tune_span()

    def run(face):
        inside = sorted((q for q in face[:-1] if abs(arch_angle(q)) < phi
                         and q[0] < 0), key=arch_angle)
        edge = [ray_hit(face, -phi)] + inside + [ray_hit(face, phi)]
        # The string hole's wall has a vertex every FACET from the axis, so a
        # span that is a whole number of facets ends ON one and the list holds
        # it twice. A zero-length run has no direction to offset along, and the
        # kerf offset came back as a spike 300mm long.
        return [q for k, q in enumerate(edge)
                if k == 0 or B.seglen(edge[k - 1], q) > 1e-9]
    outer_edge, inner_edge = run(face_o), run(face_i)
    return outer_edge, outer_edge + inner_edge[::-1] + [outer_edge[0]]


def tune_spots():
    """Each tuning pin: across the axis like its hitch pin, mid-band."""
    g = geometry()
    r_mid = g['A'] - B.THICK - BORE / 2
    out = []
    for k in range(PINS):
        u = (k - (PINS - 1) / 2) * PIN_SPACING
        out.append((-math.sqrt(r_mid ** 2 - u ** 2), u))
    return out


def tune_marks(dx=0.0, dy=0.0):
    """Drilling marks for the tuning pins: a cross, no circle -- the pin's
    diameter is the tuner's to choose, and a ring drawn at a guess would be
    read as the hole to drill."""
    out = []
    for x, y in tune_spots():
        x, y = x + dx, y + dy
        out.append(B.path([(x - 3.0, y), (x + 3.0, y)], close=False))
        out.append(B.path([(x, y - 3.0), (x, y + 3.0)], close=False))
    return out


def checks(outer, hole, parts, cheekpoly, written, ink, cut_slots):
    res = []

    def note(ok, what, detail):
        res.append((ok, what, detail))

    face_o, face_i = faces(outer, hole)
    rim = [(r[i], r[i + 1]) for r in B.contours(cheekpoly)
           for i in range(len(r) - 1)]
    mortices = [sl for q in parts for sl in B.slots_for(q)]

    # --- the section. Depth is the panel's shoulder-to-shoulder height,
    # BORE by construction in ribbon_bore.panel(); width is what varies, so
    # measure it: the narrowest gap between the two duct faces, anywhere.
    def seg_pts(poly, per=6):
        return [(a[0] + (b[0] - a[0]) * t / per, a[1] + (b[1] - a[1]) * t / per)
                for a, b in zip(poly, poly[1:]) for t in range(per)]
    narrow = min(min(B.pt_seg(p, u, v) for u, v in zip(face_o, face_o[1:]))
                 for p in seg_pts(face_i))
    note(narrow >= BORE - 1e-6, 'the duct is never narrower than the bore',
         f'narrowest {narrow:.4f}mm wall face to wall face, against {BORE:g}')

    # along the long axis (y = 0 once laid down): from the hole wall's face to
    # the outer wall's face at the far end, which is the resonator's length
    def on_axis(poly):
        xs = []
        for u, v in zip(poly, poly[1:]):
            if (u[1] > 0) != (v[1] > 0):
                xs.append(u[0] + (v[0] - u[0]) * (0 - u[1]) / (v[1] - u[1]))
        return max(xs)
    reso = on_axis(face_o) - on_axis(face_i)
    note(reso > 2 * BORE, 'the resonator opens out below the hole',
         f'{reso:.1f}mm along the axis from the hole wall to the outer wall, '
         f'{reso / BORE:.1f} bores')

    off = sum(1 for sl in mortices for pt in sl
              if not (B.inside(cheekpoly, *pt)
                      or min(B.pt_seg(pt, u, v) for u, v in rim) <= B.BURN / 2))
    note(off == 0 and mortices, 'every slot corner is inside its cheek',
         f'{4 * len(mortices)} corners on {len(mortices)} slots, {off} outside')

    def gap(A, Bq):
        return min(B.seg_gap(A[i], A[(i + 1) % 4], Bq[j], Bq[(j + 1) % 4])
                   for i in range(4) for j in range(4))
    pairs = [(i, j) for i in range(len(mortices))
             for j in range(i + 1, len(mortices))]
    worst = min(gap(mortices[i], mortices[j]) for i, j in pairs)
    note(worst - B.BURN >= B.MIN_FEATURE - 1e-9,
         'the ply between two holes survives the kerf',
         f'{len(pairs)} pairs, narrowest {worst:.3f}mm drawn, '
         f'{worst - B.BURN:.3f}mm after the kerf, against {B.MIN_FEATURE:g}')

    def plan_rect(q):
        hl, ht = q['len'] / 2, B.THICK / 2
        ca, sa = math.cos(q['ang']), math.sin(q['ang'])
        mx, my = q['mid']
        return [(mx + u * hl * ca - v * ht * sa, my + u * hl * sa + v * ht * ca)
                for u, v in ((-1, -1), (1, -1), (1, 1), (-1, 1))]

    def overlap(P, Q):
        for R in (P, Q):
            for k in range(4):
                ex, ey = R[(k + 1) % 4][0] - R[k][0], R[(k + 1) % 4][1] - R[k][1]
                a = [-ey * x + ex * y for x, y in P]
                b = [-ey * x + ex * y for x, y in Q]
                if min(a) >= max(b) - 1e-7 or min(b) >= max(a) - 1e-7:
                    return False
        return True
    rects = [plan_rect(q) for q in parts]
    np_ = jam = 0
    for i in range(len(rects)):
        for j in range(i + 1, len(rects)):
            np_ += 1
            jam += overlap(rects[i], rects[j])
    note(np_ and jam == 0, 'no two wall panels share plan area',
         f'{np_} pairs across both walls, {jam} jamming')

    short = min(q['len'] for q in parts)
    note(short >= B.TOOTH + 2 * B.SHOULDER, 'the shortest panel still holds a tooth',
         f'{short:.2f}mm against {B.TOOTH + 2 * B.SHOULDER:g}mm needed')

    bad = sum(1 for x, y, owner, _ in ink if not B.inside(owner, x, y))
    note(ink and bad == 0, 'every engraved point is on its own part',
         f'{len(ink)} points, {bad} off the material')
    # Inside a hole means deeper than half a kerf. The knot file ends its
    # crossing marks ON the edges of its waste, to three decimals, and two of
    # them land 0.0007 and 0.0018mm over the line: points the cut itself burns
    # away, not engraving in a hole. The same half kerf the rim check allows a
    # mortice. A mark genuinely in a hole is still counted.
    def in_hole(x, y, sl):
        return (B.inside(sl, x, y) and
                min(B.pt_seg((x, y), sl[i - 1], sl[i])
                    for i in range(len(sl))) > B.BURN / 2)
    over = sum(1 for x, y, _, f in ink
               if any(in_hole(x, y, sl) for sl, f2 in cut_slots if f2 == f))
    # two cheek sheets now, both carrying every mortice, and the front the
    # knot's waste as well
    knot_cut, _ = knot_paths()
    want = 2 * len(mortices) + len(knot_cut)
    note(over == 0 and len(cut_slots) == want,
         'no engraving lands in a slot',
         f'{len(ink)} points against {len(cut_slots)} holes ({len(mortices)} '
         f'mortices on each of 2 cheeks plus {len(knot_cut)} knot cuts, so '
         f'{want} expected), {over} inside one')

    # --- the sound hole. Wholly over the resonator's air, with ply to spare
    # before either wall's face: a cut that reaches a face opens into the wall
    # standing on it rather than into the duct.
    (kx, ky), (lo, hi) = knot_centre()
    placed = [[(x + kx, y + ky) for x, y in loop] for loop in knot_cut]
    pts = [q for loop in placed for q in loop]
    edges = list(zip(face_o, face_o[1:])) + list(zip(face_i, face_i[1:]))
    clear = min(min(B.pt_seg(q, u, v) for u, v in edges) for q in pts)
    inside_air = all(B.in_poly(face_o, *q) and not B.in_poly(face_i, *q)
                     for q in pts)
    note(inside_air and clear >= B.MIN_FEATURE + B.BURN,
         'the sound hole lies over the resonator',
         f'{len(placed)} cuts, nearest {clear:.1f}mm from a wall face, '
         f'against {B.MIN_FEATURE + B.BURN:g}mm')
    up = (-kx - lo) / (hi - lo)
    # against the request, 2/3, written here rather than read from KNOT_UP: a
    # check that reads the number it is checking cannot fail
    note(abs(up - 2.0 / 3.0) < 1e-9, 'the sound hole sits where it was asked',
         f'centre {up * 100:.1f}% of the way up the resonator\'s '
         f'{hi - lo:.1f}mm, {-kx - lo:.1f}mm from its bottom')

    def crosses(a, b, c, d):
        d1, d2 = (b[0] - a[0], b[1] - a[1]), (d[0] - c[0], d[1] - c[1])
        den = d1[0] * d2[1] - d1[1] * d2[0]
        if abs(den) < 1e-12:
            return False
        u = ((c[0] - a[0]) * d2[1] - (c[1] - a[1]) * d2[0]) / den
        v = ((c[0] - a[0]) * d1[1] - (c[1] - a[1]) * d1[0]) / den
        return 1e-9 < u < 1 - 1e-9 and 1e-9 < v < 1 - 1e-9
    xing = sum(1 for i in range(len(rim)) for j in range(i + 1, len(rim))
               if crosses(*rim[i], *rim[j]))
    note(xing == 0, 'the cheek outline does not cross itself',
         f'{len(rim)} edges on {len(B.contours(cheekpoly))} contours, '
         f'{xing} crossing')

    flush = max(min(B.pt_seg(p, u, v) for p in sl for u, v in rim)
                for sl in mortices)
    note(flush <= B.BURN / 2, 'the rim is flush with every mortice',
         f'furthest of {len(mortices)} mortices {flush:.4f}mm from the rim')

    # no black line through the air: every cut edge outside the duct's faces
    def in_air(p):
        return (B.in_poly(face_o, *p) and not B.in_poly(face_i, *p)
                and min(B.pt_seg(p, u, v) for u, v in
                        list(zip(face_o, face_o[1:]))
                        + list(zip(face_i, face_i[1:]))) > 1e-6)
    across = sum(1 for u, v in rim for t in (0.25, 0.5, 0.75)
                 if in_air((u[0] + (v[0] - u[0]) * t, u[1] + (v[1] - u[1]) * t)))
    note(across == 0, 'no cut line crosses the airway',
         f'{len(B.contours(cheekpoly))} cheek contour(s), {across} point(s) '
         f'inside the duct')

    # --- the hitch-pin block
    edge, block = hitch_block(face_o)
    face_edges = list(zip(face_o, face_o[1:]))
    on_wall = max(min(B.pt_seg(q, u, v) for u, v in face_edges)
                  for q in seg_pts(edge))
    n = len(face_o) - 1
    facet = min(B.seglen(face_o[i], face_o[i + 1]) for i in range(n))
    span = sum(B.seglen(u, v) for u, v in zip(edge, edge[1:]))
    # against the request -- 8 facets, 30mm -- and not against the constants
    # that drew it, which a check reading its own input could never fail
    note(on_wall < 1e-6 and abs(span / facet - 8) < 1e-6,
         'the hitch block lies against the bottom wall',
         f'{span:.1f}mm of the wall face, {span / facet:g} facets of '
         f'{facet:.2f}mm, furthest {on_wall:.1e}mm off it')
    inner_run = block[len(edge):-1]
    widths = [min(B.pt_seg(q, u, v) for u, v in face_edges)
              for q in seg_pts(inner_run + [inner_run[-1]])]
    note(abs(min(widths) - 30.0) < 1e-6,
         'the hitch block is as wide as asked',
         f'{min(widths):.4f}mm from the wall face at its narrowest, against '
         f'{HITCH_WIDTH:g}')
    kcut, _ = knot_paths()
    (kx, ky), _ = knot_centre()
    kpts = [(x + kx, y + ky) for loop in kcut for x, y in loop]
    bedges = list(zip(block, block[1:]))
    to_knot = min(min(B.pt_seg(q, u, v) for u, v in bedges) for q in kpts)
    in_air = all(B.in_poly(face_o, *q) or
                 min(B.pt_seg(q, u, v) for u, v in face_edges) < 1e-6
                 for q in block)
    note(in_air and to_knot > 1.5 and not any(B.in_poly(block, *q)
                                              for q in kpts),
         'the hitch block stays clear of the sound hole',
         f'inside the resonator, {to_knot:.1f}mm from the nearest knot cut')
    # --- the hitch pins, against the request: 7 of them, 16mm apart, 2mm
    spots = pin_spots(face_o)
    gaps = [B.seglen((0, a[1]), (0, b[1])) for a, b in zip(spots, spots[1:])]
    note(len(spots) == 7 and all(abs(g - 16.0) < 1e-9 for g in gaps)
         and abs(sum(q[1] for q in spots)) < 1e-9,
         'the hitch pins are where they were asked',
         f'{len(spots)} pins, {min(gaps):g} to {max(gaps):g}mm apart across '
         f'the axis, centred on it')
    reach = PIN_DIA  # the cross arms reach this far from the centre
    in_block = min(min(B.pt_seg(q, u, v) for u, v in bedges) for q in spots)
    note(all(B.in_poly(block, *q) for q in spots) and in_block >= reach + 5,
         'every hitch pin lands in the block',
         f'nearest pin centre {in_block:.1f}mm from the block\'s edge, so a '
         f'{PIN_DIA:g}mm pin has {in_block - PIN_DIA / 2:.1f}mm of wood round it')
    # --- the tuning-pin block
    t_edge, t_block = tuning_block(face_o, face_i)
    inner_edge = t_block[len(t_edge):-1]
    o_edges = list(zip(face_o, face_o[1:]))
    i_edges = list(zip(face_i, face_i[1:]))
    on_o = max(min(B.pt_seg(q, u, v) for u, v in o_edges)
               for q in seg_pts(t_edge))
    on_i = max(min(B.pt_seg(q, u, v) for u, v in i_edges)
               for q in seg_pts(inner_edge))
    note(max(on_o, on_i) < 1e-6, 'the tuning block touches both walls',
         f'{on_o:.1e}mm off the outer wall face, {on_i:.1e}mm off the string '
         f'hole\'s')
    across = min(min(B.pt_seg(q, u, v) for u, v in zip(inner_edge, inner_edge[1:]))
                 for q in seg_pts(t_edge))
    note(abs(across - BORE) < 1e-6, 'the tuning block is the duct\'s width',
         f'{across:.4f}mm wall face to wall face, against {BORE:g}')
    t_spots = tune_spots()
    aligned = all(abs(a[1] - b[1]) < 1e-9 for a, b in zip(t_spots, spots))
    note(len(t_spots) == PINS and aligned,
         'every tuning pin lines up with its hitch pin',
         f'{len(t_spots)} pins, each within 1e-9mm across the axis of the pin '
         f'below it')
    t_edges = list(zip(t_block, t_block[1:]))
    hold = min(min(B.pt_seg(q, u, v) for u, v in t_edges) for q in t_spots)
    ends = [t_block[0], t_edge[-1]]
    past = min(B.seglen(q, e) for q in (t_spots[0], t_spots[-1]) for e in ends)
    note(all(B.in_poly(t_block, *q) for q in t_spots) and past >= TUNE_EXTRA,
         'the tuning block runs past the outermost pins',
         f'{past:.1f}mm from the end pin to the nearest end, against '
         f'{TUNE_EXTRA:g}; {hold:.1f}mm of wood round the tightest pin')
    t_deep = TUNE_LAYERS * B.THICK
    note(t_deep <= BORE, 'the tuning block fits between the cheeks',
         f'{TUNE_LAYERS} layers x {B.THICK:g}mm = {t_deep:g}mm in a '
         f'{BORE:g}mm duct')

    deep = HITCH_LAYERS * B.THICK
    note(deep <= BORE, 'the laminated block fits between the cheeks',
         f'{HITCH_LAYERS} layers x {B.THICK:g}mm = {deep:g}mm in a '
         f'{BORE:g}mm duct')

    big = [n for n, w, h, _, _ in written if w > B.BED_W or h > B.BED_H]
    note(written and not big, 'every sheet fits the P2S bed',
         f'{len(written)} sheet(s), largest '
         f'{max(w for _, w, _, _, _ in written):.0f} x '
         f'{max(h for _, _, h, _, _ in written):.0f}mm against '
         f'{B.BED_W:g} x {B.BED_H:g}')
    return res


def lamination_sheet(tag, block, layers, spot, note, what, out_path, write,
                     ink):
    """One block's laminations on a sheet of their own, numbered 1 to layers.

    Drawn BURN/2 outside the true outline, as ribbon_bore draws every part, so
    the layers come off the bed at size and sit snug against the walls. Both
    blocks are written by this, because they differ in nothing but their shape
    and what the sheet says about them.
    """
    cut = away(block, B.BURN / 2, True)
    cut[-1] = cut[0]
    items = []
    for k in range(1, layers + 1):
        def marks(dx, dy, _k=k):
            return B.label(f'{_k:X}', spot[0] + dx, spot[1] + dy, 5.0,
                           math.pi / 2)
        items.append({'outline': cut, 'slots': [], 'marks': marks})
    stem_, ext = os.path.splitext(out_path)
    name = f'{stem_}-{tag}-cut-files{ext}'
    written = []
    for n, placed in enumerate(B.pack(items), 1):
        path_here = name if n == 1 else name.replace('-cut-files',
                                                     f'-sheet{n}-cut-files')
        marks, cuts = [], []
        for it, dx, dy in placed:
            here = [(q[0] + dx, q[1] + dy) for q in it['outline']]
            cuts.append(B.path(here))
            for d in it['marks'](dx, dy):
                marks.append(d)
                for tok in d.replace('M ', '').split(' L '):
                    a, b = tok.strip().split(',')
                    ink.append((float(a), float(b), here, path_here))
        W = max(B.bbox(it['outline'])[2] + dx for it, dx, dy in placed) + B.MARGIN_S
        H = max(B.bbox(it['outline'])[3] + dy for it, dx, dy in placed) + B.MARGIN_S

        def grp(ds, col, gid):
            return (f'  <g id="{gid}" fill="none" stroke="{col}" '
                    f'stroke-width="0.2">\n'
                    + '\n'.join(f'    <path d="{d}"/>' for d in ds)
                    + '\n  </g>\n')
        body = (f'<?xml version="1.0" encoding="utf-8"?>\n'
                f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.2f}mm" '
                f'height="{H:.2f}mm" viewBox="0 0 {W:.2f} {H:.2f}">\n'
                f'<title>Lyre-harp frame, {note}</title>\n'
                f'<desc>1 user unit = 1mm. {note}. {layers} identical '
                f'laminations of {B.THICK:g}mm ply, {layers * B.THICK:g}mm '
                f'glued up. {what} Drawn {B.BURN / 2:g}mm oversize for a '
                f'{B.BURN:g}mm kerf, joints cut to the fit of {FIT:g}. '
                f'Blue #0000ff engraves, black #000000 '
                f'cuts.</desc>\n'
                + grp(marks, B.MARK, 'numbers') + grp(cuts, B.CUT, 'outlines')
                + '</svg>\n')
        if write:
            open(path_here, 'w').write(body)
        written.append((os.path.basename(path_here), W, H, len(placed), note))
    return written


def blocks_sheets(face_o, face_i, out_path, write, ink):
    """Both laminated blocks, a sheet each."""
    _, hitch = hitch_block(face_o)
    spot = (max(q[0] for q in hitch) - HITCH_WIDTH / 2, 0.0)
    out = lamination_sheet(
        'hitch-block', hitch, HITCH_LAYERS, spot,
        f'the hitch-pin block - {HITCH_LAYERS} laminations, glue them in '
        f'number order',
        f'The long curved edge lies against the inside of the bottom wall over '
        f'{HITCH_FACETS} facets; the block is {HITCH_WIDTH:g}mm wide from it. '
        f'The {PINS} hitch pins go through the front cheek into it, '
        f'{PIN_SPACING:g}mm apart.',
        out_path, write, ink)
    t_edge, tune = tuning_block(face_o, face_i)
    g = geometry()
    spot = (-(g['A'] - B.THICK - BORE / 2), 0.0)
    out += lamination_sheet(
        'tuning-block', tune, TUNE_LAYERS, spot,
        f'the tuning-pin block - {TUNE_LAYERS} laminations, glue them in '
        f'number order',
        f'It fills the duct at the arch, touching the outer wall and the '
        f'string hole\'s wall, {BORE:g}mm between them, and runs '
        f'{2 * math.degrees(tune_span()):.0f} degrees round the arch. The '
        f'{PINS} tuning pins go through the front cheek into it, '
        f'{PIN_SPACING:g}mm apart, each in line across the instrument with '
        f'its hitch pin.',
        out_path, write, ink)
    return out


def test_parts(parts):
    """The panels the test piece is made of, and the arc they cover.

    The inner run is named by tag, TEST_FROM to TEST_TO. The outer run is
    whichever panels stand beside them: chosen by where they are, not by tag,
    so the two walls cannot be named out of step with each other.
    """
    lo, hi = int(TEST_FROM, 16), int(TEST_TO, 16)
    inner = [q for q in parts
             if q['wall'] == 'inner' and lo <= int(q['tag'], 16) <= hi]
    if not inner:
        raise ValueError(f'no inner panel is tagged {TEST_FROM}..{TEST_TO}')
    ends = [arch_angle(e) for q in inner
            for e in (q['mid'],
                      (q['mid'][0] + math.cos(q['ang']) * q['len'] / 2,
                       q['mid'][1] + math.sin(q['ang']) * q['len'] / 2),
                      (q['mid'][0] - math.cos(q['ang']) * q['len'] / 2,
                       q['mid'][1] - math.sin(q['ang']) * q['len'] / 2))]
    a0, a1 = min(ends), max(ends)
    def spans(q):
        """Where a panel starts and ends round the arch."""
        return sorted(arch_angle((q['mid'][0] + sg * math.cos(q['ang'])
                                  * q['len'] / 2,
                                  q['mid'][1] + sg * math.sin(q['ang'])
                                  * q['len'] / 2)) for sg in (-1, 1))
    # every outer panel that OVERLAPS the arc, not only those whose middle
    # falls in it: the two at the ends share their joint with the inner run
    # ON THE ARCH, and overlapping the arc. Angles wrap at the resonator end,
    # so a panel down there reads as an arch angle too and one joined the list.
    outer = [q for q in parts if q['wall'] == 'outer' and q['mid'][0] < 0
             and abs(arch_angle(q['mid'])) < math.pi / 2
             and spans(q)[0] < a1 - 1e-9 and spans(q)[1] > a0 + 1e-9]
    return inner + outer, (a0, a1)


def test_cheek(rims, span, parts_here):
    """A sector of the cheek over the arc, with room past the mortices."""
    a0, a1 = span
    r_hole = min(rims, key=lambda r: sum(B.seglen(u, v)
                                         for u, v in zip(r, r[1:])))
    r_out = max(rims, key=lambda r: sum(B.seglen(u, v)
                                        for u, v in zip(r, r[1:])))
    grow = TEST_MARGIN / (geometry()['A'] - B.THICK)      # margin, as an angle
    lo, hi = a0 - grow, a1 + grow

    def run(face):
        inside = sorted((q for q in face[:-1] if lo < arch_angle(q) < hi
                         and q[0] < 0), key=arch_angle)
        edge = [ray_hit(face, lo)] + inside + [ray_hit(face, hi)]
        return [q for k, q in enumerate(edge)
                if k == 0 or B.seglen(edge[k - 1], q) > 1e-9]
    out_edge, hole_edge = run(r_out), run(r_hole)
    return out_edge + hole_edge[::-1] + [out_edge[0]]


def test_sheet(parts, rims, out_path, write, ink):
    """The whole test piece on one sheet: two cheek sectors and the panels."""
    here, span = test_parts(parts)
    sector = test_cheek(rims, span, here)
    slots = [sl for q in here for sl in B.slots_for(q)]
    items = []
    for k in ('A', 'B'):
        def marks(dx, dy, _k=k, _p=here):
            m = B.label(_k, min(q[0] for q in sector) + 24 + dx,
                        dy, 4.0, math.pi / 2)
            for q in _p:
                ox, oy = q['out']
                m += B.label(q['tag'], q['mid'][0] - ox * 5.5 + dx,
                             q['mid'][1] - oy * 5.5 + dy, 2.0, q['ang'])
            return m
        items.append({'outline': sector, 'slots': slots, 'marks': marks})
    for q in here:
        w = q['len'] + B.BURN
        h2 = (BORE + 2 * B.THICK + B.BURN) / 2
        poly = [(px + w / 2, py + h2)
                for px, py in B.panel(q['len'], q.get('teeth'))]

        def panel_marks(dx, dy, _t=q['tag'], _w=w, _h=h2):
            return B.label(_t, _w / 2 + dx, _h + dy, 3.2)
        items.append({'outline': poly, 'slots': [], 'marks': panel_marks})
    stem_, ext = os.path.splitext(out_path)
    name = f'{stem_}-test-arch-{TEST_FROM}to{TEST_TO}-cut-files{ext}'
    n_in = sum(1 for q in here if q['wall'] == 'inner')
    note = (f'ARCH TEST PIECE - 2 cheek sectors, {n_in} string-hole panels '
            f'{TEST_FROM}-{TEST_TO} and {len(here) - n_in} outer panels')
    written = []
    for n, placed in enumerate(B.pack(items), 1):
        path_here = name if n == 1 else name.replace('-cut-files',
                                                     f'-sheet{n}-cut-files')
        marks, holes, cuts = [], [], []
        for it, dx, dy in placed:
            there = [(q[0] + dx, q[1] + dy) for q in it['outline']]
            cuts.append(B.path(there))
            for sl in it['slots']:
                holes.append(B.path([(q[0] + dx, q[1] + dy) for q in sl]))
            for d in it['marks'](dx, dy):
                marks.append(d)
                for tok in d.replace('M ', '').split(' L '):
                    a, b = tok.strip().split(',')
                    ink.append((float(a), float(b), there, path_here))
        W = max(B.bbox(it['outline'])[2] + dx for it, dx, dy in placed) + B.MARGIN_S
        H = max(B.bbox(it['outline'])[3] + dy for it, dx, dy in placed) + B.MARGIN_S

        def grp(ds, col, gid):
            return (f'  <g id="{gid}" fill="none" stroke="{col}" '
                    f'stroke-width="0.2">\n'
                    + '\n'.join(f'    <path d="{d}"/>' for d in ds)
                    + '\n  </g>\n') if ds else ''
        body = (f'<?xml version="1.0" encoding="utf-8"?>\n'
                f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.2f}mm" '
                f'height="{H:.2f}mm" viewBox="0 0 {W:.2f} {H:.2f}">\n'
                f'<title>Lyre-harp frame - {note}</title>\n'
                f'<desc>1 user unit = 1mm. {note}. A slice of the arch, cut to '
                f'try the finger joints before committing a whole frame: the '
                f'panels stand between the two sectors exactly as they do in '
                f'the instrument, {BORE:g}mm apart. Sector A and sector B are '
                f'the same part. {B.THICK:g}mm ply, slots for a {B.SHEET:g}mm '
                f'sheet at {KERF:g}mm kerf, cut to the fit of {FIT:g}. Blue '
                f'#0000ff engraves, orange #ff8000 cuts the slots first, black '
                f'#000000 frees the parts.</desc>\n'
                + grp(marks, B.MARK, 'numbers') + grp(holes, B.INNER, 'slots')
                + grp(cuts, B.CUT, 'outlines') + '</svg>\n')
        if write:
            open(path_here, 'w').write(body)
        written.append((os.path.basename(path_here), W, H, len(placed), note))
    # every mortice has to sit in the sector, as on the whole cheek
    rim = [(sector[i], sector[i + 1]) for i in range(len(sector) - 1)]
    off = sum(1 for sl in slots for q in sl
              if not (B.inside(sector, *q)
                      or min(B.pt_seg(q, u, v) for u, v in rim) <= B.BURN / 2))
    if off:
        raise ValueError(f'{off} mortice corner(s) fall outside the test '
                         f'sector; raise TEST_MARGIN')
    return written


def stem():
    g = geometry()
    return (f'lyre-harp-bore{BORE:g}-{2 * g["A"]:.0f}x{LENGTH:g}mm'
            + ('-narrow' if NARROW else ''))


def main(write=True):
    outer, hole, parts = build()
    cheekpoly = cheek(outer, hole)
    g = geometry()
    out_path = OUT or os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                   stem() + '.svg')
    # ribbon_bore.sheet() places the cheek's '0' along the start of the
    # "centreline" it is handed. There is no centreline here, and handing it
    # the outer wall put the '0' on the wall's own mortices.
    # a short run down the middle of one parallel side, in the duct, clear of
    # both walls' mortices
    mid = geometry()['A'] - geometry()['band'] / 2
    lead = lay([(mid, -10.0), (mid, -60.0)])
    (kx, ky), _ = knot_centre()
    knot_cut, knot_ink = knot_paths()
    holes = [[(x + kx, y + ky) for x, y in loop] for loop in knot_cut]
    marks = [B.path([(x + kx, y + ky) for x, y in run], close=False)
             for run in knot_ink]
    plain_items = B.items_for

    def items_for(parts_, cheekpoly_, cline_):
        """ribbon_bore's items, with the one cheek made two.

        A, the front, carries the sound hole: its waste goes in with the
        mortices, cut while the sheet still holds the cheek, and its crossing
        marks are engraved. B, the back, is the plain cheek. Each is cut once.
        """
        cheeks, panels = plain_items(parts_, cheekpoly_, cline_)
        (ck,) = cheeks
        face_o = faces(outer, hole)[0]
        front = dict(ck, slots=ck['slots'] + holes,
                     marks=lambda dx, dy: ck['marks'](dx, dy) + [
                         B.path([(float(a) + dx, float(b) + dy) for a, b in
                                 (t.split(',') for t in
                                  m[2:].split(' L '))], close=False)
                         for m in marks] + pin_marks(face_o, dx, dy)
                     + tune_marks(dx, dy),
                     note='cheek A, the front - CUT THIS SHEET ONCE - '
                          'carries the sound hole and the marks for the hitch '
                          'and tuning pins')
        back = dict(ck, note='cheek B, the back - CUT THIS SHEET ONCE')
        return [front, back], panels
    B.items_for = items_for
    try:
        written, ink, cut_slots = B.sheet(parts, cheekpoly, lead, out_path,
                                          write)
    finally:
        B.items_for = plain_items
    title = (f'Lyre-harp frame, {BORE:g}mm bore, {2 * g["A"]:.0f} x '
             f'{LENGTH:g}mm')
    if write:
        # sheet() writes a ribbon duct's title and description -- constant
        # section, swept along a curve -- which is not this part. Say what is.
        for name, _, _, _, note in written:
            f = os.path.join(os.path.dirname(os.path.abspath(out_path)), name)
            body = open(f).read()
            body = re.sub(r'<title>.*?</title>',
                          f'<title>{title} - {note}</title>', body, count=1,
                          flags=re.S)
            body = re.sub(
                r'<desc>.*?</desc>',
                f'<desc>1 user unit = 1mm. {note}. A closed duct {BORE:g}mm '
                f'deep between two cheeks, {BORE:g}mm wide over the arch and '
                f'down the parallel sides and opening to the full gap between '
                f'the hole and the outer wall below the hole, which is the '
                f'resonator. {len(parts)} wall panels, numbered round each '
                f'wall; each cheek slot carries its panel\'s number. '
                f'{B.THICK:g}mm ply, slots for a {B.SHEET:g}mm sheet at '
                f'{B.BURN:g}mm kerf, joints cut to the fit of {FIT:g}. '
                f'Blue #0000ff engraves, orange #ff8000 '
                f'cuts the slots first, black #000000 frees the parts.</desc>',
                body, count=1, flags=re.S)
            open(f, 'w').write(body)

    written += blocks_sheets(*faces(outer, hole), out_path, write, ink)

    n_in = sum(1 for q in parts if q['wall'] == 'inner')
    print(f'lyre-harp frame   {BORE:g}mm bore, {2 * g["A"]:.1f} x {LENGTH:g}mm')
    print(f'  outer: half-circles R{g["A"]:.1f} and {g["Hs"]:.1f}mm parallel '
          f'sides')
    print(f'  hole: half-circle R{g["a"]:.1f}, {g["hs"]:.1f}mm parallel sides, '
          f'corners R{g["rc"]:.1f}, bump R{g["Rb"]:.1f} rising '
          f'{g["sag"]:.1f}mm')
    print(f'  band {g["band"]:g}mm over the arch and sides; resonator below '
          f'the hole')
    print(f'  {B.THICK:g}mm ply, {KERF:g}mm kerf measured, joints cut to '
          f'the fit of {FIT:g}: {B.SLOT_TIGHTEN:g}mm off the notch, '
          f'{B.play():g}mm play a side')
    print(f'  hitch-pin block: {HITCH_LAYERS} laminations x {B.THICK:g}mm = '
          f'{HITCH_LAYERS * B.THICK:g}mm, {HITCH_WIDTH:g}mm wide over '
          f'{HITCH_FACETS} facets of the bottom wall')
    print(f'  tuning-pin block: {TUNE_LAYERS} laminations x {B.THICK:g}mm = '
          f'{TUNE_LAYERS * B.THICK:g}mm, the duct\'s {BORE:g}mm wide over '
          f'{2 * math.degrees(tune_span()):.0f} degrees of the arch')
    print(f'  {PINS} strings: hitch and tuning pins {PIN_SPACING:g}mm apart, '
          f'marked on cheek A')
    print(f'  {n_in} inner + {len(parts) - n_in} outer panels + 2 cheeks + '
          f'{HITCH_LAYERS} + {TUNE_LAYERS} laminations = '
          f'{len(parts) + 2 + HITCH_LAYERS + TUNE_LAYERS} parts, '
          f'{len(written)} sheets')
    for name, w, h, k, note in written:
        print(f'    {name:<58}{k:>3} parts  {w:.0f} x {h:.0f}mm  {note}')
    bad = 0
    print()
    for ok, what, detail in checks(outer, hole, parts, cheekpoly, written,
                                   ink, cut_slots):
        print(f'  {"pass" if ok else "FAIL"}  {what:<44} {detail}')
        bad += not ok
    if write and bad:
        for name, _, _, _, _ in written:
            f = os.path.join(os.path.dirname(os.path.abspath(out_path)), name)
            if os.path.exists(f):
                os.remove(f)
        print(f'\n  {bad} check(s) failed. Nothing written.')
    elif write:
        print(f'\n  wrote {len(written)} file(s)')
    return 1 if bad else 0


def kerf_test(path):
    """Measure the machine's kerf, in the stock the instrument is cut from.

    Nothing here is lyre-harp geometry -- it is the laser's number and it lives
    beside the instrument only because this is the file being tuned. Cut it in
    THE SAME PLY, at the same power, speed, focus and air, ideally on the same
    sheet: kerf moves with material and settings, and a figure measured on
    other stock is a figure about other stock.

    THE SLICED STRIP IS ONE RECTANGLE PLUS OPEN LINES, NOT N RECTANGLES. Drawn
    as touching rectangles every interior edge is cut twice, two kerfs come out
    where the arithmetic assumes one, and the answer is half. The check below
    counts the interior lines and refuses anything but single ones.

    Two strips cut side by side so the settings cannot drift between them, and
    so the kerf taken off the OUTSIDE of each cancels when they are compared.
    """
    m, gap = B.MARGIN_S, 6.0
    cuts, lines, marks = [], [], []
    y = m
    # the control: no interior cuts at all
    ctrl = [(m, y), (m + KERF_LEN, y), (m + KERF_LEN, y + KERF_WIDE),
            (m, y + KERF_WIDE)]
    cuts.append(B.path(ctrl))
    marks += B.label('C', m + KERF_LEN / 2, y + KERF_WIDE / 2, 5.0)
    # the comb: the same rectangle, sliced
    y += KERF_WIDE + gap
    comb = [(m, y), (m + KERF_LEN, y), (m + KERF_LEN, y + KERF_WIDE),
            (m, y + KERF_WIDE)]
    cuts.append(B.path(comb))
    step = KERF_LEN / (KERF_CUTS + 1)
    for i in range(1, KERF_CUTS + 1):
        x = m + i * step
        lines.append(B.path([(x, y), (x, y + KERF_WIDE)], close=False))
    for i in range(KERF_CUTS + 1):
        marks += B.label(f'{i + 1:X}', m + (i + 0.5) * step,
                         y + KERF_WIDE / 2, 4.0)
    # the cross-check: a square hole and the piece that drops out of it
    y += KERF_WIDE + gap
    pad = 12.0
    plate = [(m, y), (m + KERF_SQ + 2 * pad, y),
             (m + KERF_SQ + 2 * pad, y + KERF_SQ + 2 * pad),
             (m, y + KERF_SQ + 2 * pad)]
    cuts.append(B.path(plate))
    sq = [(m + pad, y + pad), (m + pad + KERF_SQ, y + pad),
          (m + pad + KERF_SQ, y + pad + KERF_SQ), (m + pad, y + pad + KERF_SQ)]
    cuts.append(B.path(sq))
    # D for the dropout. C is the control and the slices take 1..B, so a
    # digit here would read as a twelfth slice on the bench.
    marks += B.label('D', m + pad + KERF_SQ / 2, y + pad + KERF_SQ / 2, 6.0)
    W = 2 * m + KERF_LEN
    H = y + KERF_SQ + 2 * pad + m
    body = (f'<?xml version="1.0" encoding="utf-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.2f}mm" '
            f'height="{H:.2f}mm" viewBox="0 0 {W:.2f} {H:.2f}">\n'
            f'<title>KERF TEST - measure this machine, in this ply</title>\n'
            f'<desc>1 user unit = 1mm. CUT IN THE STOCK THE JOB IS CUT FROM, '
            f'at the same power, speed, focus and air - kerf moves with the '
            f'material, so a figure from other stock is about other stock. '
            f'Strip C is the control and is not sliced. The strip below it is '
            f'the same rectangle with {KERF_CUTS} cuts across it, giving '
            f'{KERF_CUTS + 1} pieces numbered 1 to '
            f'{KERF_CUTS + 1:X}. Butt all {KERF_CUTS + 1} back together '
            f'against a straightedge, lightly clamped, and measure: kerf = '
            f'(C - the stack) / {KERF_CUTS}. The square is a cross-check - '
            f'measure the hole and the piece D that fell out of it, kerf = '
            f'(hole - piece) / 2. Blue #0000ff engraves, black #000000 cuts. '
            f'Do not let the laser cut any line twice.</desc>\n'
            f'  <g id="numbers" fill="none" stroke="{B.MARK}" '
            f'stroke-width="0.2">\n'
            + '\n'.join(f'    <path d="{d}"/>' for d in marks)
            + f'\n  </g>\n  <g id="outlines" fill="none" stroke="{B.CUT}" '
              f'stroke-width="0.2">\n'
            + '\n'.join(f'    <path d="{d}"/>' for d in cuts + lines)
            + '\n  </g>\n</svg>\n')
    ok = []
    ok.append((len(cuts) == 4 and len(lines) == KERF_CUTS,
               'a control, a comb, and the square',
               f'4 outlines and {len(lines)} slicing lines'))
    # Every slice line must appear ONCE. Twice and the strip loses two kerfs
    # where the arithmetic counts one, and the measurement comes out half.
    dup = len(lines) - len(set(lines))
    ok.append((dup == 0, 'no slicing line is cut twice',
               f'{len(lines)} lines, {dup} duplicated'))
    ok.append((all(d.count(' L ') == 1 for d in lines),
               'each slice is one open line, not a rectangle',
               f'{len(lines)} lines, 2 points each, none closed'))
    widths = sorted({round(max(p[0] for p in q) - min(p[0] for p in q), 6)
                     for q in (ctrl, comb)})
    ok.append((len(widths) == 1,
               'control and comb are the same rectangle',
               f'both {widths[0]:.3f} x {KERF_WIDE:g}mm, so the outside '
               f'kerf cancels'))
    ok.append((abs(step * (KERF_CUTS + 1) - KERF_LEN) < 1e-9,
               'the slices divide the strip exactly',
               f'{KERF_CUTS + 1} pieces of {step:.3f}mm'))
    ok.append((W <= B.BED_W and H <= B.BED_H, 'the sheet fits the P2S bed',
               f'{W:.0f} x {H:.0f}mm against {B.BED_W:.0f} x {B.BED_H:.0f}'))
    print('KERF TEST   the machine\'s own number, in the job\'s own ply')
    print(f'  strip {KERF_LEN:g} x {KERF_WIDE:g}mm, {KERF_CUTS} cuts -> '
          f'{KERF_CUTS + 1} pieces;  square {KERF_SQ:g}mm\n')
    bad = 0
    for good, what, detail in ok:
        print(f'  {"pass" if good else "FAIL"}  {what:<44} {detail}')
        bad += not good
    if bad:
        print(f'\n  {bad} check(s) failed. Nothing written.')
        return 1
    with open(path, 'w') as f:
        f.write(body)
    print(f'\n  wrote {os.path.basename(path)}  {W:.0f} x {H:.0f}mm')
    print(f'\n  kerf = (C - the {KERF_CUTS + 1} pieces butted up) / '
          f'{KERF_CUTS}')
    print(f'  kerf = (the square hole - the piece that fell out) / 2')
    print(f'  currently in lyre_harp.py: KERF = {KERF:g}')
    return 0


def ladder(path):
    """One sheet of the same joint at several FIT values, to find the fit.

    Measuring the kerf answers one term of the stack-up. The joint also carries
    the cut's taper, the char on the edge and whatever the ply actually
    calipers, and none of those show up in a strip-and-slice kerf test. This
    cuts the real joint at each rung instead and lets the hand decide.

    ONE PANEL, NOT ONE PER RUNG. panel() reads BURN and TOOTH and nothing else,
    so the tab is identical at every FIT -- the mortice is the whole variable.
    Cutting five identical panels and engraving five different numbers on them
    would have been a lie in wood. The spare is a spare, not a second rung.

    The joint tested is the one there are most of: the two-tooth outer panel,
    22 of the 49. Rungs are engraved in HUNDREDTHS -- 15 is FIT 0.15 -- because
    the glyph table is hex digits and has no decimal point.
    """
    global FIT
    keep = FIT
    rungs = []
    try:
        for f in LADDER:
            FIT = f
            _, _, parts = build()
            q = next(p for p in parts
                     if p['wall'] == 'outer' and len(p['teeth']) == 2)
            local = dict(q, mid=(0.0, 0.0), ang=0.0)
            rungs.append((f, q, B.panel(q['len'], q['teeth']),
                          B.slots_for(local)))
    finally:
        FIT = keep
    tag = rungs[0][1]['tag']
    L0 = rungs[0][1]['len']
    # the coupon: a strip of cheek with this panel's mortices in it
    cw, ch = L0 + 2 * LADDER_MARGIN, LADDER_COUPON_H
    pan = rungs[0][2]
    pw = max(x for x, y in pan) - min(x for x, y in pan)
    ph = max(y for x, y in pan) - min(y for x, y in pan)
    m, gap = B.MARGIN_S, 6.0
    rowh = max(ch, ph)
    W = 2 * m + cw + gap + pw
    H = 2 * m + len(LADDER) * rowh + (len(LADDER) - 1) * gap
    cuts, holes, marks, placed = [], [], [], []
    for i, (f, q, pn, sl) in enumerate(rungs):
        oy = m + i * (rowh + gap) + rowh / 2
        ox = m + cw / 2
        box = [(ox - cw / 2, oy - ch / 2), (ox + cw / 2, oy - ch / 2),
               (ox + cw / 2, oy + ch / 2), (ox - cw / 2, oy + ch / 2)]
        cuts.append(B.path(box))
        placed.append(('coupon', box))
        for s in sl:
            holes.append(B.path([(x + ox, y + oy) for x, y in s]))
        marks += B.label(f'{round(f * 100):02d}', ox, oy + ch / 4, 3.0)
    # the panel, and one spare, in the right-hand column
    for i in range(2):
        oy = m + i * (rowh + gap) + rowh / 2
        ox = m + cw + gap + pw / 2
        poly = [(x + ox, y + oy) for x, y in pan]
        cuts.append(B.path(poly))
        placed.append(('panel', poly))
        marks += B.label(tag, ox, oy, 2.0)
    body = (f'<?xml version="1.0" encoding="utf-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.2f}mm" '
            f'height="{H:.2f}mm" viewBox="0 0 {W:.2f} {H:.2f}">\n'
            f'<title>Lyre-harp FIT LADDER - the same joint at '
            f'{len(LADDER)} fits</title>\n'
            f'<desc>1 user unit = 1mm. The same finger joint cut at FIT '
            f'{", ".join(f"{f:g}" for f in LADDER)}, to find the one that '
            f'needs a mallet. Each coupon is engraved with its fit in '
            f'HUNDREDTHS: 15 is 0.15. The panel is marked {tag} and is the '
            f'same part at every rung - panel() reads only the kerf, so the '
            f'mortice is the only thing that changes - so push the ONE panel '
            f'into each coupon in turn, softest first, and take the rung that '
            f'needs persuading. The second panel is a spare. {B.THICK:g}mm '
            f'ply, {KERF:g}mm kerf. Blue #0000ff engraves, orange #ff8000 '
            f'cuts the mortices first, black #000000 frees the parts.</desc>\n'
            f'  <g id="numbers" fill="none" stroke="{B.MARK}" '
            f'stroke-width="0.2">\n'
            + '\n'.join(f'    <path d="{d}"/>' for d in marks)
            + f'\n  </g>\n  <g id="slots" fill="none" stroke="{B.INNER}" '
              f'stroke-width="0.2">\n'
            + '\n'.join(f'    <path d="{d}"/>' for d in holes)
            + f'\n  </g>\n  <g id="outlines" fill="none" stroke="{B.CUT}" '
              f'stroke-width="0.2">\n'
            + '\n'.join(f'    <path d="{d}"/>' for d in cuts)
            + '\n  </g>\n</svg>\n')
    ok = []
    ok.append((len(rungs) == len(LADDER) and len(cuts) == len(LADDER) + 2,
               'one coupon a rung, and the panel',
               f'{len(LADDER)} coupons, 2 panels, {len(holes)} mortices'))
    # Read the DRAWN mortices, not the formula that placed them: the rungs have
    # to actually differ, and differ the right way round.
    wide = [max(x for x, y in r[3][0]) - min(x for x, y in r[3][0])
            for r in rungs]
    deep = [max(y for x, y in r[3][0]) - min(y for x, y in r[3][0])
            for r in rungs]
    down = all(wide[i] > wide[i + 1] + 1e-9 for i in range(len(wide) - 1))
    span = LADDER[-1] - LADDER[0]
    # 2:1 and 1:1, and the sheet is the place that proves it. Along the tooth
    # BOTH faces of the joint are drawn, so the notch gives up 2 * the fit;
    # across the ply the other face is the plywood and it gives up 1 *. Written
    # as 2 * span and 1 * span rather than as one number, because an earlier
    # version of this check asserted a single spread and failed a sheet that
    # was right -- the geometry was correct and the expectation was not.
    ok.append((down and abs((wide[0] - wide[-1]) - 2 * span) < 1e-6,
               'every rung is tighter than the last',
               f'mortice {wide[0]:.3f} down to {wide[-1]:.3f}mm, '
               f'{wide[0] - wide[-1]:.3f} across {span:.2f} of fit, 2:1'))
    ok.append((all(deep[i] > deep[i + 1] + 1e-9 for i in range(len(deep) - 1))
               and abs((deep[0] - deep[-1]) - span) < 1e-6,
               'and across the ply, at half the rate',
               f'{deep[0]:.3f} down to {deep[-1]:.3f}mm, '
               f'{deep[0] - deep[-1]:.3f} across {span:.2f} of fit, 1:1'))
    # the tab does NOT change; that is the claim the sheet is built on
    same = all(rungs[i][2] == rungs[0][2] for i in range(len(rungs)))
    ok.append((same, 'the panel is the same at every rung',
               f'{len(pan)} points, identical across {len(LADDER)} builds'))
    off = 0
    for i, (f, q, pn, sl) in enumerate(rungs):
        oy = m + i * (rowh + gap) + rowh / 2
        ox = m + cw / 2
        for s in sl:
            for x, y in s:
                if not (abs(x) < cw / 2 - 1e-9 and abs(y) < ch / 2 - 1e-9):
                    off += 1
    ok.append((off == 0, 'every mortice sits inside its coupon',
               f'{sum(len(r[3]) for r in rungs)} mortices, {off} corners out'))
    ok.append((W <= B.BED_W and H <= B.BED_H, 'the sheet fits the P2S bed',
               f'{W:.0f} x {H:.0f}mm against {B.BED_W:.0f} x {B.BED_H:.0f}'))
    print('lyre-harp FIT LADDER   the same joint at '
          f'{len(LADDER)} fits: {", ".join(f"{f:g}" for f in LADDER)}')
    print(f'  panel {tag}, {L0:.1f}mm, 2 teeth - the joint there are 22 of\n')
    bad = 0
    for good, what, detail in ok:
        print(f'  {"pass" if good else "FAIL"}  {what:<44} {detail}')
        bad += not good
    if bad:
        print(f'\n  {bad} check(s) failed. Nothing written.')
        return 1
    with open(path, 'w') as f:
        f.write(body)
    print(f'\n  wrote {os.path.basename(path)}  {W:.0f} x {H:.0f}mm')
    return 0


def back_numbers(path):
    """The slot numbers alone, mirrored, to engrave on cheek A's BACK face.

    The panels go in while the front plate lies face DOWN, which puts the
    numbers already on it against the table. Engraving lives on whichever face
    was up in the machine and no amount of mirroring moves it through the ply,
    so the numbers have to be put on the other face in a second, cut-free pass.

    THE PART IS WHAT MAKES THIS WORK. cheek() is mirror-symmetric about the
    instrument's long axis -- 49 outline points and all 115 mortices have their
    mirror, checked below -- so the plate can be turned over about that axis
    and lands back on its own footprint. FLIPPED THE OTHER WAY IT WILL NOT: the
    string hole is not symmetric end for end, and that is the mistake this file
    can neither detect nor survive.

    So each label is reflected y -> -y and its angle negated, while label()
    goes on drawing readable glyphs: mirrored placement, unmirrored digits.
    Same page and same 10mm margin as the cheek sheet, so the plate sits where
    it sat. Writes its own file and touches no other.
    """
    outer, hole, parts = build()
    ckp = cheek(outer, hole)
    x0, x1 = min(p[0] for p in ckp), max(p[0] for p in ckp)
    y0, y1 = min(p[1] for p in ckp), max(p[1] for p in ckp)
    m = B.MARGIN_S
    dx, dy = m - x0, m - y0
    w, h = x1 - x0 + 2 * m, y1 - y0 + 2 * m
    off = B.THICK / 2 + 1.5      # ribbon_bore's own label offset, into the duct
    # Kept per panel, so the checks below can read the ink that was actually
    # DRAWN instead of the positions this loop meant to draw it at. Checking
    # the intention passes whatever the drawing does -- proved it: with the
    # mirror taken out altogether, an earlier version of every check here
    # still said pass.
    marks, drawn = [], []
    for q in parts:
        ox, oy = q['out']
        lx, ly = q['mid'][0] - ox * off, q['mid'][1] - oy * off
        mine = B.label(q['tag'], lx + dx, -ly + dy, 2.0, -q['ang'])
        # mine[-1] is label()'s baseline tick, a two-point segment laid exactly
        # along the label's angle. It is the one piece of the ink whose
        # direction is the label's direction whatever the digits happen to be,
        # which a glyph's own extent is not: on a tall narrow numeral the
        # furthest ink from the centre sits across the baseline, not along it.
        tick = [(float(a), float(b)) for a, b in
                (t.split(',') for t in mine[-1][2:].split(' L '))]
        drawn.append((q['tag'], q['ang'], tick, [
            (float(a), float(b)) for d in mine
            for a, b in (t.split(',') for t in d[2:].split(' L '))]))
        marks += mine
    body = (f'<?xml version="1.0" encoding="utf-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{w:.2f}mm" '
            f'height="{h:.2f}mm" viewBox="0 0 {w:.2f} {h:.2f}">\n'
            f'<title>Lyre-harp cheek A - BACK FACE NUMBERS - engrave only, '
            f'no cutting</title>\n'
            f'<desc>1 user unit = 1mm. The {len(parts)} slot numbers for cheek '
            f'A, mirrored for its BACK face. ENGRAVE ONLY - there is not a cut '
            f'line in this file. Cut cheek A from its own sheet first, then '
            f'turn it over ABOUT ITS LONG AXIS - the 400mm one, so the sound '
            f'hole stays at the same end - lay it in the same place on the bed '
            f'and run this. Each number then sits beside its own mortice and '
            f'reads the right way round while you drop the panels in. Turning '
            f'the plate the other way, end for end, does NOT work: the string '
            f'hole is not symmetric that way. Blue #0000ff engraves.</desc>\n'
            + f'  <g id="numbers" fill="none" stroke="{B.MARK}" '
              f'stroke-width="0.2">\n'
            + '\n'.join(f'    <path d="{d}"/>' for d in marks)
            + '\n  </g>\n</svg>\n')
    ok = []
    ink = [(float(a), float(b)) for d in marks
           for a, b in (t.split(',') for t in d[2:].split(' L '))]
    ok.append((len(marks) > 0 and len(drawn) == len(parts),
               'one number for every panel',
               f'{len(drawn)} numbers for {len(parts)} panels, '
               f'{len(marks)} strokes'))
    # The plate is turned over, so the mirror of the cheek is what the ink has
    # to land on. It is only the same polygon BECAUSE the part is symmetric --
    # test it, do not assume it, since the whole file rests on that.
    sym = ({(round(px, 4), round(-py, 4)) for px, py in ckp}
           == {(round(px, 4), round(py, 4)) for px, py in ckp})
    ok.append((sym, 'the plate is symmetric about its long axis',
               f'{len(ckp)} outline points, mirror '
               f'{"present for every one" if sym else "MISSING for some"}'))
    # B.inside, NOT B.in_poly: in_poly takes n = len(poly) - 1 edges, so it
    # wants the first point repeated as the last and silently skips the closing
    # edge otherwise. slots_for() returns bare four-point rectangles, so
    # in_poly tested three sides of each and called this file's own numbers
    # bad, and the shipped sheet's numbers with them. inside() wraps around.
    rings = sorted(B.contours(ckp), key=lambda c: (max(p[0] for p in c)
                                                   - min(p[0] for p in c)))
    inner, band = rings[0], rings[-1]

    def on_material(x, y):
        """The plate is a ring: inside the outer rim and outside the hole."""
        return B.inside(band, x, y) and not B.inside(inner, x, y)
    off_mat = sum(not on_material(px - dx, -(py - dy)) for px, py in ink)
    ok.append((off_mat == 0, 'every number lands on the plate',
               f'{len(ink)} ink points, {off_mat} off the material'))
    slots = [sl for q in parts for sl in B.slots_for(q)]
    in_slot = sum(any(B.inside(sl, px - dx, -(py - dy)) for sl in slots)
                  for px, py in ink)
    ok.append((in_slot == 0, 'no number lands in a mortice',
               f'{len(ink)} ink points against {len(slots)} mortices, '
               f'{in_slot} inside one'))
    ok.append(('<path' in body and 'stroke="#000000"' not in body,
               'engrave only, no cut line', 'no black stroke in the file'))
    # THE ONE THAT MATTERS. Everything above can pass while a number sits
    # beside the wrong mortice, which is worse than no number at all: it would
    # be believed. Reflection is its own inverse, so a back label reflected is
    # its front label, and the mortice nearest that point has to be one of the
    # panel's own. Distances are taken to slot CENTRES, and the runner-up is
    # reported so a near miss cannot hide behind a pass.
    owner, worst, wtag = {}, 1e9, None
    for q in parts:
        for sl in B.slots_for(q):
            owner[(round(sum(p[0] for p in sl) / len(sl), 4),
                   round(sum(p[1] for p in sl) / len(sl), 4))] = q['tag']
    wrong = 0
    for tag, _, _tick, pts in drawn:
        # the ink as drawn, turned back over the way the plate will be
        cx = sum(a for a, b in pts) / len(pts) - dx
        cy = -(sum(b for a, b in pts) / len(pts) - dy)
        ranked = sorted(owner, key=lambda c: (c[0] - cx) ** 2 + (c[1] - cy) ** 2)
        if owner[ranked[0]] != tag:
            wrong += 1
        d0 = math.dist(ranked[0], (cx, cy))
        other = next((c for c in ranked if owner[c] != tag), None)
        if other is not None and math.dist(other, (cx, cy)) - d0 < worst:
            worst, wtag = math.dist(other, (cx, cy)) - d0, tag
    ok.append((wrong == 0, 'each number names the mortice beside it',
               f'{len(drawn)} numbers, {wrong} nearer another panel\'s '
               f'mortice; tightest margin {worst:.2f}mm on panel {wtag}'))
    # A number can sit in the right place and still be laid along the wrong
    # line: reflect the anchor but forget to negate the angle and it reads
    # across its own mortice. The baseline tick is the furthest ink from the
    # centre, so the centre-to-furthest vector gives the direction actually
    # drawn, and turned back over it has to be the panel's own -- mod pi,
    # since which end of the baseline is furthest is not the question.
    skew = 0.0
    for tag, ang, tick, pts in drawn:
        (ax, ay), (bx, by) = tick
        got = math.atan2(-(by - ay), bx - ax)       # turned back over
        d = abs((got - ang + math.pi) % (2 * math.pi) - math.pi)
        skew = max(skew, d)
    # Half a degree, because the tick is 0.22 * 2mm = 0.44mm long and its ends
    # are written to three decimals: 0.0005mm of rounding on a 0.44mm arm is
    # 0.065 degrees before anything is wrong. The failure this guards against
    # is an angle that was not negated, which is 2 * the panel's own angle --
    # tens of degrees nearly everywhere on a curve built from 15 degree facets.
    ok.append((skew < math.radians(0.5), 'each number lies along its own panel',
               f'worst direction error {math.degrees(skew):.4f} degrees, '
               f'against 0.5 allowed for rounding'))
    print(f'lyre-harp cheek A, BACK FACE NUMBERS   engrave only, no cutting')
    print(f"  {len(drawn)} numbers, mirrored about the long axis, "
          f'{w:.0f} x {h:.0f}mm page\n')
    bad = 0
    for good, what, detail in ok:
        print(f'  {"pass" if good else "FAIL"}  {what:<44} {detail}')
        bad += not good
    if bad:
        print(f'\n  {bad} check(s) failed. Nothing written.')
        return 1
    with open(path, 'w') as f:
        f.write(body)
    print(f'\n  wrote {os.path.basename(path)}')
    return 0


def test_piece(path):
    """Write the arch test piece on its own, without the whole frame."""
    outer, hole, parts = build()
    rims = B.contours(cheek(outer, hole))
    ink = []
    written = test_sheet(parts, rims, path, True, ink)
    here, span = test_parts(parts)
    print(f'lyre-harp arch test   panels {TEST_FROM}-{TEST_TO} of the string '
          f'hole wall and the {sum(1 for q in here if q["wall"] == "outer")} '
          f'outer panels beside them')
    print(f'  {math.degrees(span[1] - span[0]):.0f} degrees of arch, '
          f'{TEST_MARGIN:g}mm of cheek past the outermost mortice, '
          f'{KERF:g}mm kerf, joints cut to the fit of {FIT:g}')
    for name, w, h, k, note in written:
        print(f'    {name:<58}{k:>3} parts  {w:.0f} x {h:.0f}mm')
    return 0


def drawing(path):
    """The drawing the design started from, redrawn from the parts as cut.

    Everything on it is read from the same build() the cut files come from --
    the cheek's two rims, both walls' air faces, the knot where it is placed --
    and the dimensions are measured off those, so the drawing cannot say one
    size while the sheets cut another. Stood upright, arch at the top, the way
    the sketch was drawn. Not a cut file.
    """
    outer, hole, parts = build()
    rims = B.contours(cheek(outer, hole))
    face_o, face_i = faces(outer, hole)
    g = geometry()
    (kx, ky), (lo, hi) = knot_centre()
    knot_cut, knot_ink = knot_paths()

    def up(pts):
        """Laid down -> upright, arch at the top. A quarter TURN: swapping x
        and y instead is a reflection, and drew the knot as its mirror image."""
        return [(-y, x) for x, y in pts]

    def d(pts, close=True):
        return B.path(up(pts), close)

    xs = [q[0] for r in rims for q in up(r)]
    ys = [q[1] for r in rims for q in up(r)]
    x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys)
    W, L = x1 - x0, y1 - y0
    grey, blue = '#6b6b6b', '#1f6fb2'
    m = 60.0
    vb = (x0 - m, y0 - m, W + 2 * m + 70, L + 2 * m + 30)

    def dim(a, b, text, tx, ty, rotate=False):
        t = (f' transform="rotate(90 {tx:.2f} {ty:.2f})"' if rotate else '')
        return (f'<line x1="{a[0]:.2f}" y1="{a[1]:.2f}" x2="{b[0]:.2f}" '
                f'y2="{b[1]:.2f}" stroke="{grey}" stroke-width="0.4" '
                f'marker-start="url(#ar)" marker-end="url(#ar)"/>'
                f'<text x="{tx:.2f}" y="{ty:.2f}" text-anchor="middle"{t}>'
                f'{text}</text>')

    # measured, upright: arch apex at the top (small y), resonator below
    ax = 0.0                                           # the long axis, x = 0
    top_o, bot_o = y0, y1
    f_up = up(face_o)
    i_up = up(face_i)
    hole_rim = min(rims, key=lambda r: sum(B.seglen(u, v)
                                           for u, v in zip(r, r[1:])))
    h_up = up(hole_rim)
    hole_w = max(q[0] for q in h_up) - min(q[0] for q in h_up)
    hole_top = min(q[1] for q in h_up)
    side_band = min(q[0] for q in h_up) - x0
    top_band = hole_top - top_o
    k_up = up([(kx, ky)])[0]
    reso_lo, reso_hi = -lo, -hi          # upright y of the air's bottom, top
    band_y = hole_top + g['a'] + B.THICK + 0.6 * g['hs']   # on the parallel sides

    body = []
    for r in rims:
        body.append(f'<path d="{d(r)}" fill="none" stroke="#111" '
                    f'stroke-width="0.9"/>')
    for f in (face_o, face_i):
        body.append(f'<path d="{d(f)}" fill="none" stroke="{blue}" '
                    f'stroke-width="0.5" stroke-dasharray="1.5 2"/>')
    for loop in knot_cut:
        body.append(f'<path d="{d([(x + kx, y + ky) for x, y in loop])}" '
                    f'fill="#e9e2cf" stroke="#111" stroke-width="0.3"/>')
    _, block = hitch_block(face_o)
    body.append(f'<path d="{d(block)}" fill="#d8b98b" fill-opacity="0.55" '
                f'stroke="#7a5a36" stroke-width="0.5"/>')
    _, tune = tuning_block(face_o, faces(outer, hole)[1])
    body.append(f'<path d="{d(tune)}" fill="#d8b98b" fill-opacity="0.55" '
                f'stroke="#7a5a36" stroke-width="0.5"/>')
    for q in up(pin_spots(face_o) + tune_spots()):
        body.append(f'<circle cx="{q[0]:.2f}" cy="{q[1]:.2f}" '
                    f'r="{PIN_DIA / 2:g}" fill="#3a3f44"/>')
    t_up = up(tune)
    body.append(f'<text x="{ax:.2f}" '
                f'y="{min(q[1] for q in up(face_i)) + 15:.2f}" '
                f'text-anchor="middle" font-size="8">tuning-pin block, '
                f'{TUNE_LAYERS} x {B.THICK:g} = {TUNE_LAYERS * B.THICK:g}mm'
                f'</text>')
    b_up = up(block)
    # just above the block's inner edge, in the open resonator
    body.append(f'<text x="{ax:.2f}" '
                f'y="{max(q[1] for q in b_up) - HITCH_WIDTH - 13:.2f}" '
                f'text-anchor="middle" font-size="8">hitch-pin block, '
                f'{HITCH_LAYERS} x {B.THICK:g} = {HITCH_LAYERS * B.THICK:g}mm'
                f'</text>')
    body.append(dim((x0, top_o - 22), (x1, top_o - 22), f'{W:.0f}',
                    (x0 + x1) / 2, top_o - 27))
    body.append(dim((x1 + 30, top_o), (x1 + 30, bot_o), f'{L:.0f}',
                    x1 + 38, (top_o + bot_o) / 2, rotate=True))
    hx0 = min(q[0] for q in h_up)
    body.append(dim((hx0, band_y - 14), (hx0 + hole_w, band_y - 14),
                    f'hole {hole_w:.0f}', ax, band_y - 19))
    body.append(dim((x0, band_y), (hx0, band_y), f'{side_band:.0f}',
                    (x0 + hx0) / 2, band_y - 4))
    body.append(dim((x1 - side_band, band_y), (x1, band_y),
                    f'{side_band:.0f}', x1 - side_band / 2, band_y - 4))
    body.append(dim((ax, top_o), (ax, hole_top), f'{top_band:.0f}',
                    ax + 9, (top_o + hole_top) / 2 + 3))
    body.append(dim((ax - 70, reso_hi), (ax - 70, reso_lo),
                    f'resonator {reso_lo - reso_hi:.0f}', ax - 78,
                    (reso_hi + reso_lo) / 2, rotate=True))
    body.append(dim((ax + 55, k_up[1]), (ax + 55, reso_lo),
                    f'{reso_lo - k_up[1]:.1f} = 2/3', ax + 63,
                    (k_up[1] + reso_lo) / 2, rotate=True))
    body.append(f'<line x1="{ax - 40:.2f}" y1="{k_up[1]:.2f}" '
                f'x2="{ax + 60:.2f}" y2="{k_up[1]:.2f}" stroke="{grey}" '
                f'stroke-width="0.3" stroke-dasharray="4 2"/>')
    notes = [
        f'Lyre-harp frame, {W:.0f} x {L:.0f}mm, drawn from lyre_harp.py: the '
        f'same build() as the cut files.',
        f'Solid: the cheek\'s two rims as cut. Dotted blue: the faces the air '
        f'touches.',
        f'The duct is {BORE:g}mm deep throughout, {BORE:g}mm wide over the arch '
        f'and down the sides ({side_band:.0f}mm outside), and opens below the '
        f'hole into the resonator.',
        f'Sound hole: the 2-lead 7-bight knot, r30, front cheek only, centred '
        f'on the axis 2/3 up the resonator.',
        f'Hitch-pin block: {HITCH_LAYERS} laminations, {HITCH_WIDTH:g}mm wide '
        f'against the bottom wall over {HITCH_FACETS} facets; {PINS} '
        f'{PIN_DIA:g}mm hitch pins {PIN_SPACING:g}mm apart.',
        f'Tuning-pin block: {TUNE_LAYERS} laminations filling the duct at the '
        f'arch, one tuning pin above each hitch pin.',
    ]
    # Wrapped to the sheet. Each note was one <text> line, and the longest ran
    # well past the right edge of the viewBox, so a browser cut them off
    # mid-word. NOTE_CHARS is what fits from x0 to the sheet's right edge at
    # font-size 8 in Helvetica, whose average glyph is about 4.3 units there;
    # the sheet then grows to hold however many lines that makes.
    import textwrap
    room = vb[0] + vb[2] - x0 - 10
    lines = [w for t in notes for w in textwrap.wrap(t, int(room / 4.5))]
    for k, t in enumerate(lines):
        body.append(f'<text x="{x0:.2f}" y="{bot_o + 22 + 12 * k:.2f}" '
                    f'font-size="8">{t}</text>')
    tall = max(vb[3] + 20, bot_o + 22 + 12 * (len(lines) - 1) + 12 - vb[1])
    svg = (f'<?xml version="1.0" encoding="UTF-8"?>\n'
           f'<svg xmlns="http://www.w3.org/2000/svg" width="{vb[2]:.1f}mm" '
           f'height="{tall:.1f}mm" viewBox="{vb[0]:.2f} {vb[1]:.2f} '
           f'{vb[2]:.2f} {tall:.2f}" '
           f'font-family="Helvetica, Arial, sans-serif" font-size="9" '
           f'fill="{grey}">\n'
           f'<title>Lyre-harp frame, {W:.0f} x {L:.0f}mm</title>\n'
           f'<desc>Drawing for review, not a cut file. 1 unit = 1mm. Drawn '
           f'from lyre_harp.py\'s build(), the geometry the cut files in '
           f'lyre-harp-bore30-188x400mm/ come from.</desc>\n'
           f'<defs><marker id="ar" viewBox="0 0 10 10" refX="5" refY="5" '
           f'markerWidth="4" markerHeight="4" orient="auto-start-reverse">'
           f'<path d="M0,2 L10,5 L0,8 z" fill="{grey}"/></marker></defs>\n'
           f'<rect x="{vb[0]:.2f}" y="{vb[1]:.2f}" width="{vb[2]:.2f}" '
           f'height="{tall:.2f}" fill="#ffffff"/>\n'
           + '\n'.join(body) + '\n</svg>\n')
    open(path, 'w').write(svg)
    print(f'  drew {os.path.basename(path)}: {W:.1f} x {L:.1f}mm, hole '
          f'{hole_w:.1f}, sides {side_band:.1f}, top {top_band:.1f}, '
          f'resonator {reso_lo - reso_hi:.1f}, knot {reso_lo - k_up[1]:.1f} '
          f'up')
    return 0


RENDER_PAGE = r'''<!doctype html>
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
</style>
<script>
/* Follow the theme the site's pages store, so a framed viewer matches the page around it. */
(function () {
  var r = document.documentElement;
  function apply() { var t = null; try { t = localStorage.getItem('ctg-theme'); } catch (e) {}
    if (t === 'light' || t === 'dark') r.setAttribute('data-theme', t); else r.removeAttribute('data-theme'); }
  apply(); addEventListener('storage', function (e) { if (e.key === 'ctg-theme') apply(); });
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
    <div class="stage" id="stage" aria-label="3D view of the lyre-harp frame. Drag to turn, scroll to zoom.">
      <span class="hint">Drag to turn &middot; scroll or pinch to zoom</span>
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

  // view state
  const home = {yaw:-0.62, pitch:0.38, dist:760};
  let yaw=home.yaw, pitch=home.pitch, dist=home.dist;
  const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  function place(){
    frame.rotation.set(pitch, yaw, 0);
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

  const pts = new Map(); let pinch0 = 0, dist0 = dist;
  stage.addEventListener('pointerdown', e=>{ stage.setPointerCapture(e.pointerId); pts.set(e.pointerId,{x:e.clientX,y:e.clientY});
    if (pts.size===2){ const [a,b]=[...pts.values()]; pinch0=Math.hypot(a.x-b.x,a.y-b.y); dist0=dist; } });
  stage.addEventListener('pointermove', e=>{
    if (!pts.has(e.pointerId)) return;
    const prev = pts.get(e.pointerId); pts.set(e.pointerId,{x:e.clientX,y:e.clientY});
    if (pts.size===1){
      yaw += (e.clientX-prev.x)*0.008;
      pitch = Math.max(-1.45, Math.min(1.45, pitch + (e.clientY-prev.y)*0.008));
    } else if (pts.size===2){
      const [a,b]=[...pts.values()]; const d=Math.hypot(a.x-b.x,a.y-b.y);
      if (pinch0) dist = Math.max(260, Math.min(1800, dist0*pinch0/d));
    }
    render();
  });
  const up = e=>{ pts.delete(e.pointerId); if (pts.size<2) pinch0=0; };
  stage.addEventListener('pointerup', up); stage.addEventListener('pointercancel', up);
  stage.addEventListener('wheel', e=>{ e.preventDefault(); dist=Math.max(260,Math.min(1800,dist*(1+Math.sign(e.deltaY)*0.08))); render(); }, {passive:false});

  function goTo(target){
    if (reduce){ ({yaw,pitch,dist}=target); render(); return; }
    const s={yaw,pitch,dist}, t0=performance.now();
    (function step(now){
      const k=Math.min(1,(now-t0)/450), e=k<.5?2*k*k:1-Math.pow(-2*k+2,2)/2;
      yaw=s.yaw+(target.yaw-s.yaw)*e; pitch=s.pitch+(target.pitch-s.pitch)*e; dist=s.dist+(target.dist-s.dist)*e;
      render(); if(k<1) requestAnimationFrame(step);
    })(t0);
  }
  document.getElementById('b-reset').addEventListener('click', ()=>goTo(home));
  document.getElementById('b-front').addEventListener('click', ()=>goTo({yaw:0,pitch:0,dist:720}));
  const bind=(id,obj)=>document.getElementById(id).addEventListener('change',e=>{obj.visible=e.target.checked; render();});
  bind('t-front',front); bind('t-back',back); bind('t-walls',walls); bind('t-block',block); bind('t-block',pins);
  size();
})();
</script>
</body>
</html>
'''


def render(path):
    """The interactive 3D page, from the same build() as the cut files.

    Cheeks with their holes, the knot in the front one, and every panel as a
    board at its own mid, angle and length. Written in upright coordinates, y
    up, the way the drawing stands. three.js r128 from cdnjs draws it.
    """
    outer, hole, parts = build()
    rims = B.contours(cheek(outer, hole))
    rims.sort(key=lambda r: -sum(B.seglen(u, v) for u, v in zip(r, r[1:])))
    (kx, ky), (lo, hi) = knot_centre()
    kcut, _ = knot_paths()

    def up(p):                         # laid -> upright, y up
        return (round(-p[1], 3), round(-p[0], 3))
    data = {
        'rim': [up(p) for p in rims[0][:-1]],
        'hole': [up(p) for p in rims[1][:-1]],
        'knot': [[up((x + kx, y + ky)) for x, y in loop] for loop in kcut],
        'panels': [{'x': up(q['mid'])[0], 'y': up(q['mid'])[1],
                    'len': round(q['len'], 3), 'wall': q['wall'],
                    'tag': q['tag']} for q in parts],
        'panels_dir': [{'dx': round(-math.sin(q['ang']), 6),
                        'dy': round(-math.cos(q['ang']), 6)} for q in parts],
        'bore': BORE, 't': B.THICK,
        'hitch': [up(p) for p in hitch_block(faces(outer, hole)[0])[1][:-1]],
        'hitch_layers': HITCH_LAYERS,
        'pins': [up(p) for p in pin_spots(faces(outer, hole)[0])],
        'tune': [up(p) for p in tuning_block(*faces(outer, hole))[1][:-1]],
        'tune_pins': [up(p) for p in tune_spots()],
        'tune_layers': TUNE_LAYERS,
        'pin': [PIN_DIA, PIN_SPACING],
    }
    xs = [p[0] for p in data['rim']]
    ys = [p[1] for p in data['rim']]
    data['size'] = [round(max(xs) - min(xs), 1), round(max(ys) - min(ys), 1)]
    data['reso'] = round(hi - lo, 1)
    data['knot_up'] = round(-kx - lo, 1)
    data['hole_w'] = round(max(p[0] for p in data['hole'])
                           - min(p[0] for p in data['hole']), 1)
    import json
    open(path, 'w').write(RENDER_PAGE.replace(
        '__DATA__', json.dumps(data, separators=(',', ':'))))
    print(f'  rendered {os.path.basename(path)}: {len(parts)} panels, '
          f'{len(kcut)} knot cuts')
    return 0


if __name__ == '__main__':
    a = sys.argv[1:]
    for x in a:
        if not (x == '--no-write'
                or x.startswith(('--out=', '--drawing=', '--render=',
                                 '--test=', '--fit=', '--back-numbers=',
                                 '--ladder=', '--kerf-test='))):
            raise SystemExit(f'error: {x} is not a flag this generator reads. '
                             f'It takes --out=, --drawing=, --render=, '
                             f'--test=, --fit=, --back-numbers=, --ladder=, '
                             f'--kerf-test= and --no-write.')
    # --fit= is the friction knob, for walking a test piece up to the fit that
    # wants a mallet. It is the kerf the JOINTS are drawn as if cut at; the beam
    # stays at the measured KERF and every line that is not a joint is unmoved.
    ft = [x for x in a if x.startswith('--fit=')]
    if ft:
        try:
            FIT = float(ft[-1].split('=', 1)[1])
        except ValueError:
            raise SystemExit(f'error: {ft[-1]} is not a number.')
        # Below the kerf the notch would be drawn WIDER than the tooth, which is
        # a clearance fit asked for in the language of an interference one.
        if FIT < KERF:
            raise SystemExit(
                f'error: --fit={FIT:g} is under the {KERF:g}mm kerf, so it '
                f'would loosen the joint, not tighten it. --fit={KERF:g} is '
                f'the neutral value.')
    hit = [x for x in a if x.startswith('--out=')]
    if hit:
        OUT = hit[0].split('=', 1)[1]
    try:
        kt = [x for x in a if x.startswith('--kerf-test=')]
        if kt:
            sys.exit(kerf_test(kt[0].split('=', 1)[1]))
        ld = [x for x in a if x.startswith('--ladder=')]
        if ld:
            sys.exit(ladder(ld[0].split('=', 1)[1]))
        bn = [x for x in a if x.startswith('--back-numbers=')]
        if bn:
            sys.exit(back_numbers(bn[0].split('=', 1)[1]))
        ts = [x for x in a if x.startswith('--test=')]
        if ts:
            sys.exit(test_piece(ts[0].split('=', 1)[1]))
        rd = [x for x in a if x.startswith('--render=')]
        if rd:
            sys.exit(render(rd[0].split('=', 1)[1]))
        dr = [x for x in a if x.startswith('--drawing=')]
        if dr:
            sys.exit(drawing(dr[0].split('=', 1)[1]))
        sys.exit(main(write='--no-write' not in a))
    except ValueError as e:
        print(f'error: {e}')
        sys.exit(1)
