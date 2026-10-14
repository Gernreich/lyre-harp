# lyre_harp.py design notes

The reasoning behind `lyre_harp.py`. It was written beside `ribbon_bore.py`, in
the swept-curve folder of trumpet-elbows-allowed, and these notes were a section
of that folder's NOTES.md until 2026-10-14, when the swept-curve trumpet bores
became the [swept-curve](https://github.com/Gernreich/swept-curve) repository and
this generator came here. The lyre-harp is its own instrument, not a swept-curve
trumpet; it only borrows that generator's parts.

## A closed duct whose width changes

Not a `--shape`, because every shape in `ribbon_bore.py` is two walls a fixed
bore either side of one centreline. The lyre-harp frame (2026-09-16) is two
independent closed walls: a stadium 400mm long outside, and a string hole with
a concentric arch, parallel sides and a bump between tangent corners. Over the
arch and down the sides the duct is 30 x 30mm; below the hole it opens to the
whole gap, about 200mm along the axis, which is the resonator. It imports the
panel, mortice, label, packer, sheet and ring-contour code from `ribbon_bore`
and adds only the two outlines and the checks a constant bore made
unnecessary. Its cut files go to this repository, whose
`.repro` records the command for each and is gated since 2026-10-13.

The front cheek carries a sound hole: the 2-lead 7-bight knot from
`knotwork-soundholes`, read from that repository's cut file and placed
unchanged, centred on the axis two thirds of the way up the resonator's air
from the far end. So the cheeks are two parts now, A with the knot and B
without, each cut once. The knot's waste is cut with the mortices and its
crossing marks engraved. Two of those marks end 0.0007 and 0.0018mm past the
edge of a waste cut, rounding in the knot file, so a mark counts as in a hole
only when it is deeper than half a kerf.

`--drawing=PATH` draws the review drawing in this repository from the same `build()`: the
cheek's rims as cut, the air's faces, the knot as placed, and dimensions
measured off them. It stands the part upright by a quarter turn, not by
swapping x and y, which is a reflection and drew the knot mirrored.

`--render=PATH` writes the interactive 3D page that sits beside the cut files
in this repository: both cheeks with their holes and the knot,
and every panel as a board at its own position, angle and length, drawn with the
Three library, release r128, loaded from cdnjs. Tabs, slots and engraving are left off.

The hitch-pin block: loop-end strings go over hitch pins driven through the
bottom wall, and 3mm of ply cannot hold a pin, so six 3mm laminations are glued
up inside the bottom end, 18mm deep. In plan each lies against the outer wall's
face and is 30mm wide from it, over 8 facets' length centred on the axis. The
bottom end has an odd number of facets with one centred on the axis, so 8
whole facets cannot be symmetric: the span is 7 whole facets and a half at each
end, which also keeps the block's ends off the panel joints. Its checks compare
against 8 and 30 as asked, not against the constants that drew it.

Seven hitch pins, 2mm, 16mm apart across the axis and centred on it, go
through the front cheek into the block. Each sits on the block's mid-line, 15mm
in from the wall face, so the row follows the curve of the bottom end. They
are ENGRAVED on cheek A as drilling marks -- a 2mm circle with a cross through
it -- not cut: a hole in the cheek alone would not guide a drill on into the
block. A spacing the block cannot reach across is refused.

The tuning-pin block is the same idea at the arch: 6 laminations filling the
duct there, touching the outer wall's face and the string hole's, 30mm between
them. It spans 120 degrees -- out to the facet midpoint past the last pin,
which leaves 33.7mm of wood beyond it -- and carries one tuning pin above each
hitch pin, on the mid-line of the band and in line with it across the
instrument. Those marks are a cross with no circle: a tuning pin's diameter is
the builder's choice, and a ring drawn at a guess reads as the hole to drill.

Both blocks are written by one lamination_sheet(). Each span ends where a face
has a vertex -- the string hole's wall carries one every facet -- so both
builders drop a repeated point first: a zero-length run has no direction to
offset along, and the kerf offset came back as a 300mm spike.

**The lyre-harp keeps the measured 0.13mm kerf and tightens its joints with
`FIT`, 0.25.** The test came off the bed with loose finger joints (2026-09-20),
and the kerf was first raised to 0.17 and then 0.20 to tighten them. That
worked, because an overstated kerf draws each tab fat and each mortice thin,
but it also drew every other line 0.07mm out: outlines oversize, holes
undersize, the duct and the knot with them. So the kerf went back to the truth
and the interference moved to `FIT`, which only the joints read, then up to
0.25 for a fit that needs a rubber mallet. `lyre_harp.KERF` sets
`ribbon_bore.BURN` for its own run and nothing else. Raising `BURN` in ribbon_bore.py would
redraw every shipped swept-curve sheet, and those are the record of parts already cut.

`--test=PATH` writes an ARCH TEST PIECE: the string-hole wall's panels
TEST_FROM..TEST_TO, every outer panel that overlaps the arc they cover, and a
sector of the cheek for each side carrying their mortices, with TEST_MARGIN of
rim past the outermost one. It is for trying the joints before committing a
whole frame. Two things the first cut of it got wrong: angles wrap at the
resonator end, so a panel down there read as an arch angle and joined the list
until the outer run was pinned to the arch side; and choosing outer panels by
their midpoint dropped the two at the ends of the arc, which share their joint
with the inner run, so they are chosen by overlap instead. It refuses to write
if a mortice corner falls outside the sector.

The faces are drawn and the walls derived from them. Drawn the other way the
duct over the arch came out 29.72mm (on-circle vertices on the outer wall),
then 29.987mm (the hole face's mitred vertices), and `the duct is never
narrower than the bore` caught both.

