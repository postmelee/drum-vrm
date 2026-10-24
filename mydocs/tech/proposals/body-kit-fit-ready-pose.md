# Body/kit fit and instrument-ready motion

Status: proposed follow-up. Scope is owned by [#18](https://github.com/postmelee/drum-vrm/issues/18)
and [#20](https://github.com/postmelee/drum-vrm/issues/20); this is not an implementation or release claim.

## Problem and evidence

The [baseline assessment](../research/2026-10-24/snare-ready-and-kit-fit.md) identifies
repeated snare departures from a generic rest policy and body/kit reach mismatch.
The [palm measurement](../../report/verification/palm-orientation-2026-10-24.md) shows
that accurate tool contact can coexist with large wrist bends. Preserve both findings
when judging a replacement trajectory.

## Proposed order

1. Establish coherent dimensions and measured seat/target fitting across body proportions.
2. Use instrument-specific ready positions and target-dependent wrist/forearm strokes.
3. Add explicit sticking/grip/pedal policies and shared clearance checks.
4. Add optional expressive movement only after a feasible restrained baseline.

Do not branch on avatar identity, stretch bones, move MIDI times or conflate visual
body scale with physical target fitting. A bounded compact-kit option may be needed
where placement alone cannot fit the measured body.

## Acceptance direction

Compare the same authored MIDI across controlled body variants. Include isolated and
repeated snare, hat/snare combinations, fills, unreachable contacts, first/last events,
random seeks and export timestamps. Measure actual tools, wrist posture and clearance,
not only target coordinates. Numerical thresholds require engineering justification.

[ADR 0006](../../../specs/decisions/0006-technique-and-expression-foundation.md)
preserves the broader technique/expression rationale and T01–T09 rule IDs.
Adoption should update the corresponding specification, leaving this proposal and its
baseline evidence traceable; active completion criteria remain in the linked issues.
