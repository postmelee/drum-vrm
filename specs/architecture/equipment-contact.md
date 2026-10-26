# Equipment geometry and musical contact

**English** · [한국어](equipment-contact.ko.md)

This contract connects visible drum bodies, shared hardware, playable regions and
the performer's complete tools. [Calibration](calibration.md) owns body fitting;
[MIDI interpretation](midi-interpretation.md) owns source meaning. A geometrically
valid layout is one prerequisite for a performance, not a certificate that every
motion frame or item of clothing is clear.

## One numerical geometry boundary

[instrument-body.ts](../../src/instrument-body.ts) measures the static displayed
body after its appearance transform and before cosmetic hit response. It excludes
markers and the source model's designated hardware/pedals. Its planning-coordinate
descriptor travels with `KitPiece.geometry`; the Rust engine never reads scene or
avatar names. [equipment.rs](../../crates/drum-engine/src/equipment.rs) owns generated
equipment, and [kit-hardware.ts](../../src/kit-hardware.ts) renders its returned
supports and pedal boxes rather than maintaining another routing algorithm.

| Descriptor | Meaning |
| --- | --- |
| `version`, `source` | Version 1, `mesh-bounds` or `procedural`; neither means exact collision triangles |
| `referenceRadius` | Radius at which local dimensions were measured, in metres |
| `body.center`, `body.halfExtents` | Axis-aligned body box relative to the piece's planning anchor |
| `surface.center`, `surface.normal`, `surface.radius` | Approximate circular playing region and unit normal in the same frame |

For anchor `p` and scale `s = piece.radius / referenceRadius`, a local point becomes
`p + s × localPoint`; box half-extents and surface radius also scale by `s`.
The normal remains a unit direction. Descriptor validation rejects nonfinite values,
unknown versions/sources, reference or surface radii outside 0.01–1 m, body
half-extents outside 0.001–2 m, local centres beyond ±3 m, and scale outside 0.05–5.

Tagged surface meshes identify the projected playing footprint; otherwise body
bounds provide an approximation. A downward ray samples the displayed surface's
contact and transformed interpolated normal (face normal fallback), and the descriptor's centre lies on that
plane. A missing intersection falls back to the anchor-height horizontal plane.
The static scale/rotation and sampled tilt are retained, but a single plane does
not reconstruct a curved bow, raised bell or exact rim surface.
Legacy entries without a descriptor use documented procedural dimensions; this
does not identify their imported appearance. Historical projects do not acquire
new measured geometry without a new measurement transaction.

## Layout, hardware and access

The shared result contains `bodies`, `supports`, `pedals` and
`unresolvedPieceIds`. Body boxes and pedal footboard/base boxes use metres; supports
are finite capsules with endpoints and radii. Closed/open hi-hat aliases share one
body; pedal contacts remain distinct. Own mount attachments and linked hi-hat
hardware are intentional overlaps, not blanket exclusions for nearby instruments.

Static layout checks combine:

- A 5 mm body separation margin and ground bounds.
- Distinct projected playing regions for snare/tom heads. Raising one drum above
  another does not make their shared striking area playable.
- A bounded approach corridor above each head, extending 0.12 m along its normal.
  Its radius is `min(0.45 × headRadius, 0.09 m)`; it tests nearby body boxes.
- Generated stand, basket, collar, tripod, pedal and kick-hardware capsules against
  other bodies and pedal boxes. A bounded mast/boom search may find a clear route;
  failure retains `unresolvedPieceIds` rather than hiding a support.

The hi-hat's straight linkage is not silently converted into a remote boom stand.
Other stand candidates use offsets up to 0.5 m and bounded tripod rotations.
This is equipment-envelope routing, not a load-bearing stability or mechanical
joint simulation. Support-to-support crossings, exact mesh curvature and every
moving cymbal/pedal state remain separate verification work.

Appearance replacement preserves the identity of the instrument and its source
events. Because a replacement can change bounds, a new geometry/layout transaction
must be inspected; an old fit result is not evidence for a new mesh. Deliberate fit
may adjust bodies and rebuild supports together. Merely shrinking the visible mesh
without updating the planning descriptor cannot resolve a planning collision.

## Articulation chooses a region, not a universal nearest edge

[contact.rs](../../crates/drum-engine/src/contact.rs) projects the seated shoulder
midpoint toward each playing plane. This player-facing direction chooses a sector
within the articulation's region. For centre `c`, normal `n`, observer `o` and radius
`r`, let `u = normalize((o − c) − n × dot(o − c, n))`; the target is `c + α × r × u`.
The degenerate direction uses a fixed fallback. Kick and pedal contacts retain
their existing sole-contact anchors.

| Articulation | Engineering radial fraction `α` |
| --- | --- |
| Snare/tom `center` | 0 |
| Ride `bell`, `inner`, `outer` | 0.12, 0.38, 0.62 |
| `edge`, `rimshotEdge`, `sideStick` | 0.86 |
| `crash`, `china`, `splash` | 0.88 |
| Other stick-played hi-hat articulations | 0.62 |

These deterministic fractions are project approximations, not dimensions prescribed
by Yamaha or Zildjian. They do not add random dispersion. Exact source times,
identity, velocity and physical-piece identity remain unchanged. Restored plans
accept a legacy anchor or a bounded point on the described playing plane; subsequent
profile preparation derives the current articulation region again.

The tip is still the principal contact marker. A near-edge point does **not** yet
implement a simultaneous head/rim rimshot, a cross-stick butt strike, a shoulder-on-
cymbal-edge crash, or a bell's curved surface. Those require separate tool contact
frames and multiple constraints. Hi-hat openness and tip/shoulder contact mode are
independent concepts; an open note does not prove which part of the stick was used.

## Complete-tool approach and motion

[equipment_motion.rs](../../crates/drum-engine/src/equipment_motion.rs) checks arm
capsules, the measured/fallback hand volume, and the stick from its butt to its tip.
The butt is 0.105 m behind the calibrated grip and the stick radius is 0.006 m.
Own hand/tool attachment is permitted; other equipment is still an obstacle.

Only the terminal 20 mm can enter the explicitly intended approximate playing
surface. The target piece comes from the active stroke/hold branch, never from
spatial coincidence with an arbitrary nearby surface. Open hi-hat aliases resolve
only to a closed hi-hat at the same physical anchor and radius. The remaining shaft
continues to check that same body. For an approach within the
playing region, that shaft query clips the conservative body by the head plane;
otherwise a rim/bell's highest point in an AABB would fill the entire playable
area. This explicit approximation does not certify actual rim/bell clearance.
Unrelated bodies, hardware and pedal boxes retain their full envelopes.

Bounded lift and wrist-pitch candidates seek a clear approach. Lift is zero at
an actual impact, so a correction cannot move a source hit into the air or shift
its time. Both torso/elbow coordination and equipment inspection use these
absolute-time samples. Failed searches remain diagnostics; a least-bad trajectory
is not a collision-free result. There is no rule to raise every elbow: available
motion depends on the measured body, target region and complete shaft approach.

The correction track uses 30 Hz plus exact impact times, with at most 16,000
samples. Lift stays in 0–0.20 m and pitch offsets in ±0.60 rad; adjacent corrections
are limited to 1.2 m/s and 2 rad/s before linear interpolation. These limits bound
the added correction, not the velocity of the complete musical stroke. Source
impact time wins over an almost-equal floating-point grid time. The bounded track
is included in the [prepared Worker transfer](engine-contract.md#preparation-and-worker-boundary),
so installation does not repeat its search.

## Explicit stick-to-stick count-in

[count_in.rs](../../crates/drum-engine/src/count_in.rs) uses only retained events
explicitly interpreted as `CountIn`. It neither guesses from note number alone nor
adds drum `Hit` entries or synthesized audio. Its current gesture brings both
actual stick tips together in front of the body; it is not a simulation of one
stick striking the other shaft.

Each cue reserves both hands for 0.22 s before and after its original time.
Overlapping accepted cues share an approach/recovery phrase. An actual hand-played
hit, including an unassigned one, takes priority in this window. Nonduplicate cue
times closer than 0.12 s are diagnosed rather than forcing a faster gesture.
Without a feasible common body/equipment contact candidate, the cue remains in the
source and receives `COUNT_IN_GESTURE_UNAVAILABLE`.

Contact is intentional only when both tips actually meet within 2 mm. Only the
terminal 40 mm of both sticks receives that exception: each remaining shaft is
still checked against the other complete stick, and arm/body/opposite-hand checks
continue. The entire cue interval is not exempt. Throughout a cue phrase there is no intended
drum surface, so equipment contact receives no playing-surface exception. Cue start/contact/end times enter
the same bounded control timeline even when a body cannot perform the gesture.
Evaluation and prepared-worker restoration use absolute song time.

## Evidence and next constraints

[Articulation research](../../docs/research/articulation-and-tool-contact.md)
separates primary teaching sources from engineering choices. Source material is
linked and credited, not bundled or relicensed. Tests cover region distinctions,
blocked access, shaft contacts and count-in source/timing/restore invariants.
Actual body/skin and rendered-tool comparisons remain necessary across presets
and VRM proportions. Sampled proxy checks are not continuous swept-volume or
triangle-level collision certificates.
