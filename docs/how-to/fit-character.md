# Fit the seat and drum contacts

**English** · [한국어](fit-character.ko.md)

Use the calibration controls identified in [feature availability](../reference/availability.md). Both the built-in skeleton and imported VRM use measured bone lengths and the same solver, without avatar-name-specific animation branches.

1. Load your character, or keep the built-in skeleton. In **캐릭터** (Character), start with **자동 맞춤** (Automatic fit).
2. Adjust seat height and forward/back position, then press **좌석 적용** (Apply seat). A positive forward value moves the seated body toward the kit. See [control ranges](../reference/calibration.md).
3. Open **드럼** (Drums). Select a component in the scene or list and adjust its **X/Y/Z contact position**, then press **타점 적용** (Apply contact). X is left/right, Y is height, and Z is forward/back. The kick contact belongs to the foot pedal, not the front drum head. Open/closed hi-hat share a cymbal target; the hi-hat pedal has its own target.
4. Check the resting feet first. Play the intended MIDI, pause at important contacts, and inspect the actual stick tips. Adjust seat and targets together. Seat position alone may not accommodate very different arm and leg proportions.
5. Read reach diagnostics and revisit arbitrary times. Unreachable hits keep their original timing and audio; the app does not stretch bones or move notes to hide the problem. An unreachable pedal can be reported even without a pedal note in the MIDI.

## Inspect the grip

Choose **손바닥 방향** (Palm direction): **위** (up), **안쪽 · 엄지 위** (inward / thumbs up), or **아래** (down). The setting applies to both hands. Changing it pauses playback, recalibrates the grip and invalidates an earlier video download. The choice persists across character/seat changes in the same session.

This adjusts palm orientation, not a complete traditional-grip technique. Available finger bones curl around a diagonal hand-local stick socket. Forearm roll helps distribute rotation, but distant targets can still produce unnatural wrists. Inspect gloves, hands and shoes on the actual model; contact measurements do not inspect every skin or shoe vertex.

Feet start at the pedal contacts and use a small ankle-driven preparation/press. Full heel-up/heel-toe mechanics and collision-free crossed arms are not implemented. The planner prefers right-hand hi-hat and left-hand snare but can choose another hand for reach or timing.

Settings last for the current session and are lost on reload. Record useful values yourself. Controls can be disabled during preparation, character loading or export. See [calibration behavior and limits](../reference/calibration.md).

For the reasoning behind body proportions, seat fitting and contact placement, read the [calibration design](../../specs/architecture/calibration.md).

[Task guides](README.md)
