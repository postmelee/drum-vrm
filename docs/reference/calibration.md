# Calibration controls

**English** · [한국어](calibration.ko.md)

These controls belong to the development capabilities in [availability](availability.md). Follow [the fitting procedure](../how-to/fit-character.md) to use them.

| Control | Range or meaning |
|---|---|
| Seat height | 30–120 cm |
| Seat forward/back | −50–50 cm; positive toward the kit |
| Contact X | −200–200 cm; positive to the player's right |
| Contact Y | 2–250 cm; pedal contacts use 4–20 cm |
| Contact Z | −50–250 cm; positive forward |
| Palm direction | Up (default), inward/thumbs up, down; both hands together |
| Automatic fit | Recalculate a starting seat from measured body and pedal placement |

Closed/open hi-hat share a cymbal target. Hi-hat pedal is separate. Kick contact means the foot pedal, not its drum head. Display controls use centimetres; the solver uses metres.

Seat and contact changes replan motion for the measured body. Appearance scale/rotation does not change reach targets or sound. Bones and source MIDI hit times are not stretched or shifted. Invalid/unreachable configurations report diagnostics.

The grip is a hand-local stick socket with available finger bones. Palm direction is not a complete traditional grip. Feet remain based at the pedal contacts and use ankle strokes; mechanical pedal boards, full heel-up/heel-toe and collision-free posture are not provided.

Settings are session-only. Loading/preparation/export can disable the controls. Changing the palm setting pauses playback and invalidates an older video download. Hair/clothing springs reset during time evaluation for seek/output repeatability.

The [rig contract](../../specs/architecture/rig-contract.md) defines the implementation boundary. [Performance principles](../explanation/performance-principles.md) distinguish current approximations from planned techniques.

[Reference](README.md)
