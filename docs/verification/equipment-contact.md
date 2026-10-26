# Equipment and contact verification

This report records verification of the equipment/contact batch. It does not
replace earlier [body-clearance measurements](body-clearance-measurements.md).
The current contract is [equipment/contact](../../specs/architecture/equipment-contact.md);
[source research](../research/articulation-and-tool-contact.md) distinguishes teaching
evidence from implementation choices. Publication and release state require their
own recorded candidate and authorization.

## Count-in invariants

Executed locally: `cargo test -p drum-engine count_in::tests -- --nocapture`.
All six module tests passed on the working implementation before the final batch
candidate was frozen; repeat against that candidate before publication.

| Check | Observed result |
| --- | --- |
| Explicit interpretation | Note 75 alone does not produce a cue; only a retained positive note-on interpreted as `CountIn` does |
| Original score | Cue preparation leaves source events and actual drum hits unchanged |
| Occupancy | An unassigned real hand strike still blocks the overlapping cue window |
| Unsupported profile | Missing target-dependent hand frames produces a diagnostic and no fabricated gesture |
| Exact cue pose | Both solved tip targets meet at the same deterministic contact point; candidate body/opposing-arm/shaft checks pass on the authored profile |
| Contact exception | Nonterminal shaft overlap remains a failure; separated tips and nonintentional contact do not receive the exception |
| Coordinates and boundaries | Translating the common profile translates the cue; phrase outer boundaries return the original tips continuously |
| Actual evaluator and transfer | Authored SMF → plan → profile Motion → prepared restoration preserves source events, one original kick and exact cue times; both evaluated tips are within 1 μm of the cue point and shuffled evaluations match exactly |

These fixtures are independently authored and do not establish compatibility with
every imported VRM, natural contact force, continuous skin clearance or a physically
simulated shaft-click. The implemented cue is deliberately tip-to-tip.

## Prepared equipment-path transfer

Executed locally: `cargo test -p drum-engine prepared::tests -- --nocapture`.
Five tests passed on the working implementation. The added equipment fixture
serializes its actual precomputed approach track, restores it exactly and compares
poses/inspection at shuffled times. Restoration rejects missing/oversized tracks,
altered time knots, out-of-range or negative lift, excessive pitch/rates and nonzero
impact lift, including edits with a recomputed noncryptographic checksum.

A separate regression retains original `0.1`-second source time over the nearby
`18 / 30 − 0.5` grid value, which differs in floating-point representation. This
prevents grid deduplication from turning an exact impact into a nonimpact approach
sample. The same source-time priority applies to torso/elbow control times.

## Contact region and intended-surface regressions

`cargo test -p drum-engine contact::tests -- --nocapture` passed four tests.
They cover distinct articulation regions, an offset tilted surface at 1.6× scale,
translation invariance, finite tangent fallback when the observer aligns with any
principal/diagonal normal, legacy-anchor acceptance, and radius/plane-distance/
nonfinite/pedal rejection boundaries.

The dedicated `surface_contact_requires_the_intended_physical_piece` regression
also passed. A stick may use the terminal playing-surface exception only for its
explicit current target. The same coordinates with a different target, an unknown
target or no target remain collisions. An open hi-hat aliases only a closed hi-hat
at the same anchor and radius. The six count-in tests also check phrase-boundary
activation; integration supplies no drum target throughout an active phrase. This closes
the earlier spatial-coincidence loophole; a nearby surface is not inferred as the
musical target.

## Measured body and short-performance audit

Local WASM SHA-256:
`cdcce294bb1ac06c13cbe13e0003996b9d1d69a6e20450f5ba848a9937f5de2c`.
The same actual displayed Studio geometry was used for all four profiles. Imported
VRMs were loaded with their original skinned geometry, semantic bones and constraints;
textures/materials were removed only from the in-memory Node test copy. The input
files stayed private and unchanged. Model identities and licensing are tracked in
[model research](../research/vrm-models.md). The built-in rig uses the
explicit skeletal fallback; the three VRMs report `mesh-estimate` body geometry.

Each profile used the common automatic seat calculation and palms-up control frame,
then the bounded full-equipment fit. These are observed local wall times, including
other concurrent checks, not a performance guarantee.

| Profile | Fit result | Uniform scale | Candidates | Limit reached | Fit time |
| --- | --- | --- | --- | --- | --- |
| Built-in rig | Complete | 100% | 349 | No | 0.06 s |
| VRM1 Constraint Twist Sample | Not found; original layout retained | 100% retained | 56,104 | No | 4.01 s |
| Seed-san | Complete | 92% | 45,193 | No | 1.89 s |
| AvatarSample B | Complete | 66% | 57,320 | No | 3.82 s |

The Twist result fails the required snare-ready/crossed-groove checks in the retained
original layout. It is an unresolved bounded-search result, not a claim that the
model cannot play drums. No performance-clearance success is reported for that
failed fit. A complete result requires the designated single-hand probes and two
required pairs, not all 64 possible simultaneous surface combinations.

For each complete fit, the independently authored 8-second interpretation demo was
prepared with the documented Logic drum mapping, explicit count-in and short
open/close policy. All 82 arranged hits remained assigned, and source events,
source IDs and hit times matched the pre-fit source plan. The audit evaluated 585
unique timestamps: a 60 Hz grid including preparation/tail, exact hit times and
exact count-in times. It applied each evaluated pose to the real adapter and also
compared reversed-order evaluation and prepared-format-2 restoration.

| Profile | Planning / prepared restore | Actual cue contacts | Maximum cue error | Maximum rendered marker error | Reversed / restored pose differences |
| --- | --- | --- | --- | --- | --- |
| Built-in rig | 1.35 s / 17.3 ms | 3/3 | 0.000033 mm | 0.000124 mm | 0 / 0 |
| Seed-san | 1.43 s / 4.9 ms | 3/3 | 0.000033 mm | 0.000459 mm | 0 / 0 |
| AvatarSample B | 1.13 s / 6.4 ms | 3/3 | 0.000005 mm | 0.000196 mm | 0 / 0 |

All three cue contacts occur at their original `0`, `0.333333`, `0.666666` seconds.
Their minimum inspected equipment clearance is respectively 39.1, 56.8 and 26.7 mm
for built-in/Seed-san/AvatarSample B. Candidate placement accounts for measured arm
length **and calibrated tool length**; an arms-only forward distance had incorrectly
placed short-armed rigs' wrists behind the body. This is a common numerical rule.

### Residual paths are not cleared by a static fit

A negative value below −0.001 mm counts as an overlapping proxy sample. Body minima
combine torso, opposing arms, sticks and opposite-arm/tool checks. Equipment minima
include bodies and shared hardware. These observations do not test moving skin or
provide continuous swept-volume certificates.

| Profile | Negative body samples / 585 | Minimum body clearance | Negative equipment samples / 585 | Minimum equipment clearance |
| --- | --- | --- | --- | --- |
| Built-in rig | 29 | −10.75 mm | 12 | −6.00 mm |
| Seed-san | 47 | −52.32 mm | 18 | −6.00 mm |
| AvatarSample B | 170 | −32.97 mm | 18 | −6.00 mm |

The engine's own narrower path diagnostics remain visible: built-in has five
30 Hz/impact equipment samples (maximum 0.9 mm penetration); Seed-san has eight
(maximum 6.0 mm) plus one body-failing impact (52.3 mm); AvatarSample B has eleven
(maximum 6.0 mm) plus 24 body-failing impacts (19.2 mm). The independent 60 Hz table
also catches between-impact states, so its counts and minima need not match those
diagnostics. Exact marker contact and deterministic restoration do **not** mean the
performance is natural or collision-free. These residuals and the Twist fit remain
follow-up constraints; they were not hidden by moving source times or altering bones.

## Displayed preset and full-song checks

With the same WASM above, all four distinct preset geometries completed native-rig
fitting: Studio, Innerscene, Batoski and smoj. Local fit wall times were 0.06, 0.19,
0.16 and 18.13 seconds while other tests ran. Actual public GLB mesh vertices stayed
inside their measured boxes; Rust descriptors matched the descriptors measured after
rendering the fitted layouts exactly. No generated support **centerline** intersected
another rendered body. This ray test excludes own mounts and does not certify full
capsule thickness, support-to-support crossings or moving skin. No source GLB changed.

The authorized bundled `투명소녀1.mid` (SHA-256
`615127ef5d0bfe7c4e934222349243e8d7c9c8309fa9e1a56e0b19bb0e41f5cf`)
was prepared using its catalog selection and the native rig with Studio geometry.
The approved final-snare omission remains explicit selection data. All **1,720**
arranged hits were assigned; original source events, IDs and times were unchanged.
Fit took 60 ms at 100% scale, preparation 25.43 s, and prepared restore 44.4 ms.
Across 6,862 grid/exact-impact/cue/tail samples, reversed evaluation and restored
poses had zero differences. The three original count-in cues at 9.188181,
9.514268 and 9.840355 seconds were performed, with inspected equipment clearance
39.1–56.1 mm. These are local measurements, not browser performance guarantees.

The full-song independent scan still found 170 equipment proxy intersections
(maximum 7.88 mm) and 117 body/paired-tool intersections (maximum 13.64 mm).
The engine's own grid has 171 equipment candidates and seven body-failing impacts.
The slightly different sample sets explain the different counts. This batch adds
checks and bounded avoidance; it does **not** establish collision-free completion
of the reference song.

## Repository and browser verification

- `cargo test --workspace --locked`: 115 passed, none ignored.
- `cargo fmt --all -- --check` and strict workspace clippy: passed.
- Rebuilt WASM, `npm test`: 25 files / 187 tests passed.
- Typecheck/production web build, generated license notices and `docs:check`: passed.
- Local IAB on `http://127.0.0.1:5189/drum-vrm/`: startup automatic fitting reports
  100%; the 8-second interpretation demo has zero unmapped/unassigned hits and
  retains its visible residual path warning. At song time 0.333 s, actual rendered
  sticks meet in front of the body and the inspector identifies intentional
  count-in contact (body 83 mm, equipment 54 mm clearance, marker errors rounded
  to 0.00 mm). A local screenshot is retained under `.local/equipment-contact/`.
- Changed the active preset to Innerscene with a prepared score: geometry and plan
  updated together, and the 8-second demo reached its end. Loaded the full bundled
  song in that preset: 1,720 hits / zero unassigned, three count-in cues, original
  approved one-note omission. At song time 9.514 s the rendered sticks meet and
  equipment clearance reads 57 mm. Its separate equipment path diagnostic remains
  visible (142 samples, maximum 8.6 mm); ten body-failing impacts remain. Browser
  error logs were empty. These Innerscene results are not the Studio audit above.

The candidate is on `codex/equipment-contact-planning`, stacked after PR #47.
No merge, Pages deployment, release tag or GitHub Release was performed for this
batch. These measurements do not establish what the public application runs.
