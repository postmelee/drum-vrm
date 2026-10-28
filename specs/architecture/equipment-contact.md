# Equipment geometry and musical contact

**English** · [한국어](equipment-contact.ko.md)

This contract connects visible drum bodies, shared hardware, playable regions and
the performer's complete tools. [Calibration](calibration.md) owns body fitting;
[MIDI interpretation](midi-interpretation.md) owns source meaning. A geometrically
valid layout is one prerequisite for a performance, not a certificate that every
motion frame or item of clothing is clear.

## One numerical geometry boundary

[The default kit](../../shared/default-kit.json) is the single seed used by Rust,
the browser and default pedal contacts. It retains nominal 14-inch snare/hi-hats,
12/13/16-inch toms, 16/18-inch crashes and a 20-inch ride/kick. The measured Studio
appearance starts with clear body bounds, head-approach regions and generated
supports, including the directly connected hi-hat pedal. This physical seed is
not a universal avatar fit: changed bodies or proportions still require inspection.

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
| `surface.radialProfile` | Optional native cymbal surface: local origin, unit quaternion, increasing radius/height samples and an optional explicit `bellRadius` |
| `surface.motion` | Optional semantic translation capability: unit axis and absolute travel in metres, independent of shell fit scale |

For anchor `p` and scale `s = piece.radius / referenceRadius`, a local point becomes
`p + s × localPoint`; box half-extents and surface radius also scale by `s`.
The normal remains a unit direction. Descriptor validation rejects nonfinite values,
unknown versions/sources, reference or surface radii outside 0.01–1 m, body
half-extents outside 0.001–2 m, local centres beyond ±3 m, and scale outside 0.05–5.

Tagged surface meshes identify the projected playing footprint; otherwise body
bounds provide an approximation. A downward ray samples the displayed surface's
contact and transformed interpolated normal (face normal fallback), and the descriptor's centre lies on that
plane. A missing intersection falls back to the anchor-height horizontal plane.
The static scale/rotation and sampled tilt are retained. Native acoustic cymbals
also supply the lathe profile that generated their mesh. Articulation contacts use
that profile's piecewise-linear height and normal after the same rigid transform;
a single sampled bow plane no longer substitutes for its bell or edge. The profile
has 2–32 increasing radius/height samples, a finite local origin within ±3 m, and a
unit quaternion. Radii lie in 0–1 m and heights in ±1 m before fit scaling.
When present, `bellRadius` must be positive and no larger than the profile's outer
radius. Native geometry supplies its bell boundary; unknown imported profiles do
not acquire an inferred bell region.
Imported surfaces without this semantic profile remain an explicit planar
approximation; this field does not infer arbitrary curved geometry from a box.
Legacy entries without a descriptor use documented procedural dimensions; this
fallback declares a flat cymbal profile and a bell boundary at 24% of its radius.
It does not identify their imported appearance or add that boundary to an explicitly
measured planar import. Historical projects do not acquire
new measured geometry without a new measurement transaction.

## Layout, hardware and access

The shared result contains `bodies`, `supports`, `pedals`, `mechanisms` and
`unresolvedPieceIds`. Boxes use metre-valued world `center` and local `halfExtents`.
Body boxes are axis-aligned; pedal footboard/base boxes also carry an optional
xyzw `rotation` (identity when absent). Segment queries transform into that local
frame, and box separation uses the oriented axes. Supports are finite capsules
with endpoints and radii. Closed/open hi-hat aliases share one
body; pedal contacts remain distinct. Own mount attachments and linked hi-hat
hardware are intentional overlaps, not blanket exclusions for nearby instruments.

Static layout checks combine:

- A 5 mm body separation margin and ground bounds.
- Distinct projected playing regions for snare/tom heads. Raising one drum above
  another does not make their shared striking area playable.
- A bounded approach corridor above each head, extending 0.12 m along its normal.
  Its radius is `min(0.45 × headRadius, 0.09 m)`; it tests nearby body boxes.
- Generated stand, basket, collar, tripod, pedal and kick-hardware capsules against
  other bodies, pedal boxes and unrelated supports. An expanded-box broad phase
  precedes exact segment-distance checks. Intentional common-holder joints are
  explicit exemptions; an accidental shared endpoint is not one. A bounded
  mast/boom search may find a clear route; failure retains `unresolvedPieceIds`
  rather than hiding a support.

Support topology follows the physical role: a snare basket, bass-mounted rack-tom
holder/L-arms when a kick is present, three shell-mounted floor-tom legs, and
angled boom cymbal stands. A standalone rack tom may use a stand when the layout
has no kick assembly. Other stand candidates use offsets up to 0.30 m and bounded
tripod rotations. Failed attachment routes remain unresolved; a lower collision
count does not certify load-bearing stability.
The shared rack holder is generated once on the kick: a shell socket, telescopic
post, locking collar, yoke and two ball joints. Each rack tom uses a horizontal
L-rod section and a vertical insertion section into an external side clamp;
suspension brackets connect that clamp to the shell. It is not a diagonal rod
terminating at the head or underside centre. Bounded arm length and rise reject
unreachable mounts instead of stretching their connectors. Floor-tom brackets
join three independent legs to the shell. The snare basket retains a connected
neck and tilter above its tripod.

Snare baskets use a 225 mm tripod radius; boom cymbal and direct hi-hat stands
use 300 mm. These are shared engineering dimensions. Optional
`KitPiece.standRotation` fixes the first tripod leg's absolute world-Y phase in
radians (`−π…π`), with X/Z direction `[sin(phase), cos(phase)]`. It is supported on
snare, hi-hat, crash and ride stands. If absent, routing selects a bounded phase;
if present, it may move the stand base within existing limits but cannot silently
select another phase. This lets fitting test the real tripod adjustment against
seated legs while preserving the instrument's position. It is not a collision
exemption. Saved layouts retain the accepted phase.

An optional `mount: {kind: 'standClamp', supportPieceId: 'ride'}` selects a
suspended floor tom. Only `tomLow` may depend on a distinct ride/crash stand
without its own mount; absence retains the three floor legs. Dependency order
routes that host before the tom. A physical clamp, 50 mm socket, ball joint,
horizontal/vertical L-rod and shell bracket replace the legs. The clamp accepts
15.9–28.6 mm pipe diameters; the arm is bounded to 25–380 mm horizontally,
80–300 mm rise and 35 mm receiver insertion. Only the clamp sleeve and short
socket may overlap their named host's vertical tube. Both the cymbal and tom
body-centre projections must lie inside the host tripod triangle. Other supports,
the shell and the rod retain collision checks. A failed route stays unresolved;
the preset does not hide legs and draw an unchecked connector.

Boom stands use three nested tube diameters, locking collars, memory locks, a
boom pivot, a straight boom with a rear tail, a separate cymbal tilter, sleeve and
felt, and double-braced tripod legs with rubber feet. One leg points toward the
boom within the bounded tripod-rotation search. A projected cymbal-centre check
keeps the load within the support triangle;
it is a geometric plausibility constraint, not a mass, torque or tipping analysis.
Native cymbal mounts use the radial profile's local axis, not the sloped bow's
contact normal. This keeps the mounting spindle aligned with the bell hole.

[Manufacturer references](../../docs/research/drum-hardware.md) explain the
topology and distinguish published dimensions from our compact engineering
bounds. [equipment_hardware.rs](../../crates/drum-engine/src/equipment_hardware.rs)
owns those bounds and routes. Its semantic `role` metadata selects details in
[hardware-parts.ts](../../src/hardware-parts.ts); endpoints and capsule radii still
own their placement and maximum envelope. Screws and collars stay inside those
envelopes, so visual detail does not introduce an independent collision model.
Fixed hardware is merged by material per instrument; the kick beater remains
separately articulated. Detail therefore does not add a draw call per screw.

The direct hi-hat mast is constrained to the pedal's toe-side axis, 0.12 m ahead
of the sole anchor **along its own horizontal direction**. Hi-hat tripod legs
rotate with this frame: one leg points toward the toe side, leaving a gap toward
the heel rather than running a leg through the footboard. The bounded route
search may rotate the tripod further; body/leg clearance remains a separate test.
Measured body offsets determine the corresponding hi-hat
contact anchor. A mismatch above 3 mm remains unresolved; no long lateral linkage
or implicit remote hi-hat conceals it. Common fitting keeps this relationship.

Native appearance defaults tilt the snare 4°, rack toms 8°, ride 10°, and crashes
12° toward the player; floor tom and hi-hat remain level. These are editable
engineering defaults, not prescribed technique angles. The appearance limit is
±30°. Body measurement includes the actual tilt, and project files retain it;
older projects without a tilt field retain 0°. Same-assembly self-collision beyond
the constructed joints, load/weight stability and complete moving-part swept
volumes remain separate verification work. Top-view overlap between an elevated
cymbal and a drum is not automatically invalid; the bodies, supports and head
approach corridor must remain clear in three dimensions.

Appearance replacement preserves the identity of the instrument and its source
events. Because a replacement can change bounds, a new geometry/layout transaction
must be inspected; an old fit result is not evidence for a new mesh. Deliberate fit
may adjust bodies and rebuild supports together. Merely shrinking the visible mesh
without updating the planning descriptor cannot resolve a planning collision.

## Deterministic pedal and cymbal mechanisms

[kit-mechanisms.ts](../../src/kit-mechanisms.ts) binds semantic moving parts to
Rust's numerical `mechanisms` frames. Each frame supplies a rear footboard hinge,
a transverse axle, a physical contact and normal, and bounded travel. It is a
kinematic mechanism evaluated at absolute song time, not a frame-integrated
physics simulation.

`KitPiece.pedalForward` is an optional horizontal unit vector on `pedalHat`.
Omitting it preserves the legacy world +Z direction. Values must be finite,
have `abs(y) ≤ 1e-8` and unit-length error ≤ `1e-6`; a kick cannot carry this
field because its beater and bass assembly retain their common fixed axis.
The direction maps semantic local +Z from heel toward mast and local +X to the
transverse hinge. Automatic placement derives it from the seated left leg and
mast, without avatar-name rules. Saved layouts preserve it; score changes and
seeking do not reorient the accepted setup.

- With sole anchor `T` and horizontal yaw `Q`, the footboard hinge is
  `H = T + Q · [0, −0.012, −0.13]`. At pitch `θ`, the shared sole point is
  `H + Q · Rx(θ) · [0, 0.012, 0.13]`. `mechanisms.boardRotation` carries `Q`.
  The engine rotates the sole contact about the same hinge and solves the leg
  without changing bone lengths. At zero pitch, the original contact is exact.
  The renderer consumes the semantic foot quaternion rather than inventing a
  second foot animation. Neutral profile foot rotations remain unchanged;
  evaluation composes layout yaw once. Automatic seat height, ready pose,
  contact feasibility, footboard rendering and collision boxes use that frame.
- The kick's beater sphere has radius 28 mm. At impact its front surface meets the
  batter plane, separately from the footboard contact. Its shaft and sphere share
  a pivot at the pedal axle. A smooth 70 ms approach and 100 ms recovery envelope
  reaches contact at the original assigned kick times, with a 0.45 rad rest angle.
  The measured rendered surface marker is distinct from the intended contact.
  Arbitrary imported batter planes still use the body's rear bound approximation.
- Native hi-hat bells face outward, with a 2 mm closed rim gap. Only the upper
  cymbal translates along the shaft, by up to 25 mm from the common hi-hat state.
  The lower cymbal stays fixed. Library adapters retain their existing semantic
  upper-cymbal identification through material batching, so the two parts are not
  welded together at runtime. Measurement emits `surface.motion` only for a
  separated semantic upper part. Electronic pads and unknown imported bodies
  without that capability remain static; planning does not lift a contact off an
  immovable mesh. A unit motion axis and travel in 0–25 mm are validated. Native
  geometry-less legacy layouts use the native 25 mm fallback; measured imports
  do not gain mechanism compatibility from successful file loading.

The same hi-hat state moves the planned contact and the upper rendered surface.
Static equipment bounds describe the closed/impact arrangement; animation does
not turn them into a continuous collision certificate for every moving assembly.
Cosmetic feedback is separate from physical pedal movement and cannot move an
immutable source hit to conceal contact error.

## Shared contact records and hi-hat state

[contact_events.rs](../../crates/drum-engine/src/contact_events.rs) derives
`PerformancePlan.contactEvents` from validated hits, assignment, equipment and
accepted count-in cues. Source identity, microsecond time and velocity are retained;
loading an old or prepared plan does not make an imported contact record authoritative.
Preparation regenerates the records. The same geometry and state first resolve
motion targets, then supply feedback and inspection through
[contact-events.ts](../../src/contact-events.ts).

| Contact field | Contract |
| --- | --- |
| `sourceEventId`, `hitId`, `timeUs`, `time`, `velocity` | Original event identity and timing; count-in has no invented hit ID |
| `limb`, `toolPart`, `role` | Assigned limb and `stickTip`, `sole` or `beater`; primary, mechanism or paired-tool role |
| `pieceId`, `surfaceId` | Physical piece and intended head, bow, bell, edge, pedal or batter surface; paired tools have no instrument |
| `frame`, `localPoint`, `localNormal` | Reference-scaled piece point and unit normal, or a world point for paired tools; paired tips do not invent a surface normal |
| `mechanismOffset` | Full translation vector already included at impact, in absolute metres |
| `status` | `planned` or `unassigned`; neither asserts measured mesh contact |

Kick events distinguish the sole on the footboard from the beater at the batter
head. A count-in supplies two paired stick-tip contacts at one common world point.
Only assigned stick/beater surface events create instrument feedback. An unassigned
note, foot pressure or paired-stick cue cannot produce a misleading drum impact ring.
The ring follows the resolved point and normal, with a 3 mm display offset to avoid
surface flicker. Its radius does not inherit a model's appearance scale. Body feedback
pivots around the current contact; the hi-hat uses its mechanical translation instead
of an additional cosmetic wobble that would move its playing surface independently.

The actual rendered stick, sole or beater marker is measured separately at the
event's exact time. The resulting distance, normal and tangential error have scope
`actual-marker-to-planned-contact`. A small error proves agreement of those markers;
it does not certify skin clearance, triangle contact or a complete striking technique.

[hihat_motion.rs](../../crates/drum-engine/src/hihat_motion.rs) derives a single
absolute-time opening track. Literal open notes request full opening; closed and
pedal-close notes request closure. Explicit short-open controls retain their release,
open and close endpoints and chosen openness. A simultaneous closure takes priority.
Changes use smoothstep interpolation over at most 100 ms. An isolated pedal close
can prepare an opening within the preceding 160 ms, bounded after the previous
authoritative endpoint; its peak is `0.35 + 0.65 × velocity / 127`. These are procedural
motion defaults, not measured acoustic openness or recovery of a player's technique.

The shared opening drives the left foot pitch, rendered upper cymbal and impact
target. `surface.motion` determines whether and how far that actual surface moves;
the track does not grant movement to a fixed electronic pad or an unknown imported
mesh. `HIHAT_SURFACE_STATIC` reports a requested opening without that capability.
The contact's impact displacement is included once, and a fading marker follows
the same surface as it closes. Playback, shuffled seeking and prepared restoration
sample the same track; source note times never move.

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
accept a legacy anchor or a bounded point on the described playing surface; subsequent
profile preparation derives the current articulation region again.

For a radial profile with an explicit bell boundary, the nominal bell fraction
is moved outward as needed to clear the fixed felt radius, full stick radius and
posture margin, then capped at `bellRadius`. The bell boundary scales with the
instrument; the mounting hardware does not. A bell too small for that clearance
remains a failed equipment test, rather than silently becoming a bow strike or
exempting its own mount. Profiles without this metadata retain their nominal
contact and explicit approximation limits.

The tip is still the principal contact marker. A near-edge point does **not** yet
implement a simultaneous head/rim rimshot, a cross-stick butt strike, or a shoulder-on-
cymbal-edge crash. Those require separate tool contact
frames and multiple constraints. A native bell's surface geometry is represented
by its radial profile; correct geometry alone does not implement every striking
technique. Hi-hat openness and tip/shoulder contact mode are
independent concepts; an open note does not prove which part of the stick was used.

## Complete-tool approach and motion

[equipment_motion.rs](../../crates/drum-engine/src/equipment_motion.rs) checks arm
capsules, the measured/fallback hand volume, and the stick from its butt to its tip.
`assess_hand` keeps physical-tool and measured-body results separate: the whole
stick/equipment result is a hard candidate condition; arm/hand/equipment and
stick/body envelopes are advisory costs and warnings. A large sleeve cannot veto
an otherwise valid tool contact. The raw combined query is retained for inspection.
The executable [shared stick profile](../../shared/stick-profile.json) owns both
rendering and queries. The butt extends 105 mm behind the grip; the terminal
radius is 0, 3, 4 and 6 mm at 0, 8, 25 and 70 mm back from the contact marker,
then remains 6 mm through the shaft and butt. This piecewise taper is an
engineering approximation, not a measured commercial stick. The actual rendered
endpoint coincides with the marker. Own hand/tool attachment is permitted;
other equipment is still an obstacle.

Only the terminal 20 mm can enter the explicitly intended approximate playing
surface. The target piece comes from the active stroke/hold branch, never from
spatial coincidence with an arbitrary nearby surface. Open hi-hat aliases resolve
only to a closed hi-hat at the same physical anchor and radius. The remaining shaft
continues to check that same body using the shared taper. For an approach within
a planar playing region, that shaft query clips the conservative body by the
surface plane; otherwise a rim's highest point in an AABB would fill the entire
playable area. A declared radial profile instead uses the complete upper surface,
as described below. The intended surface permits tangent contact without a free-space margin;
other obstacles retain the full maximum-radius capsule and 2 mm safety margin.
The terminal exception remains explicit: it does not prove tangent contact of a
rounded tip, actual rim/bell clearance, or exact triangle contact along the shaft.
Unrelated bodies with an authored radial profile use that curved upper surface,
with the full 6 mm shaft radius and no terminal exemption. Flat heads retain their
full body boxes, including the rim; unknown bodies, hardware and pedal boxes also
retain their full envelopes. World surfaces are cached per equipment context.

[radial_clearance.rs](../../crates/drum-engine/src/radial_clearance.rs) queries a
tapered shaft against the authored cymbal profile in its rigid local frame. It
does not extend the tangent at one bell contact across the bow. For local height
`h(r)`, outer radius `R` and maximum profile slope magnitude `L`, each shaft
interval obtains this conservative clearance lower bound:

`max((min(y) − max(h(r))) / sqrt(1 + L²), min(r) − R, box-face bounds) − max(shaft radius)`.

Radial extrema are exact along the segment; maximum profile height includes all
knots in that radial range. Box-face bounds retain the measured body envelope.
The centre hole and space under the upper profile are deliberately filled, so
an underside approach cannot pass as a valid strike. Intervals subdivide only
within depth 16 and a 256-visit subdivision budget; unresolved bounds never
become clearance. This may conservatively reject near-tangent paths. The query
does not infer curvature for planar imports, change arm/hand body-box checks,
or certify triangle, skin, clothing or continuous motion clearance.

Fitting and score assignment share one neutral-torso feasibility query. It checks
impact, 55 mm preparation and 110 mm recovery probes with the same bounded pitch
and elbow candidates, anatomical limits and complete-tool equipment query.
Measured body proxies rank candidates but do not veto reach. Distinct
playing regions are checked separately: snare centre/rim and ride bell/bow/edge
cannot inherit each other's reach result. A static witness does not certify the
route between contacts. Optional torso turns must preserve the actual selected
impact pitch; a hypothetical different pitch or turn cannot validate the stored
motion. Fit and assignment therefore do not use incompatible reach gates.

Bounded lift and wrist-pitch candidates seek a clear approach. Lift is zero at
an actual impact, so a correction cannot move a source hit into the air or shift
its time. Both torso/elbow coordination and equipment inspection use these
absolute-time samples. Failed searches remain diagnostics; a least-bad trajectory
is not a collision-free result. There is no rule to raise every elbow: available
motion depends on the measured body, target region and complete shaft approach.

Impact pitch is scheduled before projecting the intervening samples onto the
2 rad/s correction limit. A nine-state bounded search first maximizes the number
of retained neutral posture/equipment witnesses, then prefers comfortable paths.
An incompatible fill therefore does not discard valid anchors elsewhere in the
song. `CONTACT_PITCH_LIMIT` reports unsupported impact witnesses from their first
source time; it does not certify the retained trajectory as clear. If no reachable
pitch path exists, the previous diagnosed track is retained. Torso preservation
checks the same fixed-pitch anatomical/tool-equipment witnesses at correction
samples, reducing yaw where it would destroy an existing valid neutral candidate.
Measured-body overlap remains advisory in both paths.
A candidate elbow still needs the later joint temporal planner; interpolation and
skin/clothing remain separately inspected.

The correction track retains its mandatory 30 Hz grid and exact hand impacts.
Up to three refinement passes inspect interval midpoints, adding failed samples
when the interval is at least 1/240 s long, within the same 16,000-sample total.
Lift stays in 0–0.20 m and pitch offsets in ±0.60 rad; adjacent corrections are
limited to 1.2 m/s and 2 rad/s before linear interpolation. These limits bound the
added correction, not the velocity of the complete musical stroke. Source impact
time wins over an almost-equal grid time. The [prepared Worker transfer](engine-contract.md#preparation-and-worker-boundary)
preserves mandatory times and bounded extra controls, so installation does not repeat search.

After torso planning and before elbow coordination,
[tool_pair_path.rs](../../crates/drum-engine/src/tool_pair_path.rs) jointly repairs
both tools through the existing pitch/lift track. It inspects each control and the
quarter, half and three-quarter points between controls. At a failure it tries
16 pitch/lift delta pairs per hand, for at most four passes and 250,000
changed-region sample evaluations. Minimal rate-limited envelopes retain the same
amplitude/rate limits, exact targets and zero lift at impacts. A candidate must
improve the inspected result without turning a previously valid anatomical,
equipment or tool-pair sample into a failure. This introduces no new transfer field.
Elbow swivel alone cannot move a target-oriented shaft centreline; the later joint
elbow planner remains necessary for anatomical coordination. Either bounded search
can finish with residual diagnostics; refined samples are not a continuous-path certificate.

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
Without a feasible common anatomical/tool-equipment contact candidate, the cue remains in the
source and receives `COUNT_IN_GESTURE_UNAVAILABLE`.

Opposing stick inclines are derived from the actual terminal taper and measured
horizontal opening between the hands. The cue contact point and time stay fixed.
Preparation checks 40 approach/rebound phases against actual tool geometry.
Underlying equipment corrections blend out at cue contact and back in on exit,
so a correction cannot displace the intentional contact.

Inter-stick checks use both complete butt-to-tip profiles from the shared rendered
taper, including during count-in. Conservative capsule bounds subdivide uncertain
terminal bands within a finite budget; an unresolved bound does not become a pass.
The thinner terminal geometry introduces no new contact exception.

Contact is intentional only when the tips meet within 2 mm and each lies within
2 mm of the cue contact point. Only the terminal 40 mm of both sticks receives
that exception: each remaining shaft still requires 2 mm from the other complete
stick. During the close approach/rebound, tool-pair clearance may be below the
ordinary 2 mm margin but must remain nonnegative; penetration is never allowed.
Arm/body/opposite-hand measurements remain advisory. Equipment retains its 2 mm
margin throughout the cue, with no intended drum surface or playing-surface
exception. Cue start/contact/end times enter the same bounded timeline even when
a body cannot perform the gesture. Evaluation and prepared-worker restoration use
absolute song time.

## Inspection categories

`inspect_pose_at` retains `equipmentClearanceMeters`, the combined minimum across
arms, hands and tools. Its optional `toolEquipmentClearanceMeters` reports only the
complete physical sticks against equipment; `toolPairClearanceMeters` reports
complete stick pairs. Whole-tool checks must be present, finite and nonnegative for
a sampled tool-clearance pass. Missing optional fields are unmeasured, not a pass.
Advisory body overlap can coexist with a valid tool sample, and a clear body proxy
cannot hide a stick intersecting a tom or another stick.

`TOOL_EQUIPMENT_PATH_CLEARANCE` and `TOOL_PAIR_PATH_CLEARANCE` distinguish residual
physical-tool intersections from the retained combined `EQUIPMENT_PATH_CLEARANCE`
and advisory `BODY_PATH_CLEARANCE`. Bounded motion search may still report failures
after static calibration succeeds. The [engine contract](engine-contract.md) owns
inspection/review fields; none is a continuous triangle-level collision proof.

## Evidence and next constraints

[Articulation research](../../docs/research/articulation-and-tool-contact.md)
separates primary teaching sources from engineering choices. Source material is
linked and credited, not bundled or relicensed. Tests cover region distinctions,
blocked access, shaft contacts and count-in source/timing/restore invariants.
Actual body/skin and rendered-tool comparisons remain necessary across presets
and VRM proportions. Sampled proxy checks are not continuous swept-volume or
triangle-level collision certificates.
