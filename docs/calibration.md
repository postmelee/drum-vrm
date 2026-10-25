# Seat, contact points, and drum parts

**English** · [한국어](calibration.ko.md) · [Documentation](README.md) · [Character models](models.md)

Fit the performer and kit by adjusting the seat and instrument contacts, then customize the visible drum bodies. This guide describes the repository build; see [version availability](../README.md#which-version-am-i-using) for the public app. The [calibration design](../specs/architecture/calibration.md) explains the shared measurements and solver.

## Fit a character to the kit

1. Load a VRM 1.0 character through **캐릭터** (Character), or use the built-in skeleton. See the [model guide](models.md) for formats and limits. Both use measured bone lengths and the same solver; the app does not select a different animation implementation by avatar name.
2. Start with **자동 맞춤** (Automatic fit). In Character settings, adjust **seat height** from **30 to 120 cm** and **forward/back position** from **−50 to 50 cm**. Positive forward values move the seated body toward the kit. Press **좌석 적용** (Apply seat) after editing. Automatic fit recalculates a starting seat for the current body and pedal arrangement.
3. If the hand-played kit is too large or distant, press **키트를 체형에 맞춤** (Fit kit to body) in Character settings. It adjusts the hand-played contacts and sizes together while leaving kick/pedal contacts fixed. Inspect the result and reach message; this is not collision-free fitting. **기본 배치** (Default layout) restores the kit layout.
4. Open **드럼** (Drums), select a component in the view or selector, and adjust its **X/Y/Z contact position** in centimetres. X is side-to-side (**−200–200 cm**, positive toward the player’s right), Y is height and Z is forward/back (**−50–250 cm**, positive forward). Closed/open hi-hat share the cymbal position. The hi-hat pedal is adjusted separately. Kick's contact is the foot pedal, not the kick drum's front head. Pedal contact height is limited to **4–20 cm** for the current shared hardware; other target heights allow **2–250 cm**. Press **타점 적용** (Apply target) to use the edited position.
5. Inspect both feet at rest, then play the same MIDI and check the sticks at contact. Adjust seat and targets together until the important contacts are reachable. Seat position alone may not fit an avatar with very different arm and leg proportions.
6. Read the updated reach diagnostics. Unreachable hits keep their original audio and timing; the app does not lengthen bones or move notes to hide the problem. An unreachable pedal is reported even when the MIDI contains no pedal note. Revisit arbitrary times to inspect the calibrated pose.

Changes apply to the current session and are **not saved across reloads**. Keep notes of useful settings until project persistence is available. Preparation, character loading and video export may temporarily disable calibration controls to keep the current operation consistent.

## Feet and hands

Feet start on the calibrated kick and hi-hat pedal contacts. During a hit, the ankle stays in place and the foot makes a small toe-up preparation before pressing back to the contact. The same time-based evaluation is used for playback, seeking and video output. This simple ankle-only motion does not reproduce heel-up, heel-toe or double-pedal technique, and it is not a mechanical pedal simulation.

In Character settings, **손바닥 방향** (Palm direction) selects **위** (up, the default), **안쪽 · 엄지 위** (inward / thumbs up), or **아래** (down). The setting applies to both hands and is retained while changing the seat or importing another character in this session. It pauses playback, recalibrates the grip and invalidates an older video download.

The grip uses a diagonal hand-local stick socket and available finger bones. The solver follows each target with a neutral hand/forearm relationship and uses forearm roll for palm orientation. Hands normally wait above snare/hi-hat, or over the same piece between repetitions. Velocity affects lift and wrist motion; fast snare passages can use both available hands. These are common procedural rules, not complete push-pull, anatomical limits or collision avoidance.

These are palm orientations, **not complete traditional-grip styles**: traditional grip uses a different underhand left-hand finger arrangement and an overhand right hand. Matched grips commonly use downward or inward-facing palms; see [Yamaha’s grip guide](https://hub.yamaha.com/music-educators/instruments/perc/a-guide-to-proper-stick-grips/). Models without finger bones still load, with reduced grip detail. The contact inspector measures actual stick-tip and foot-socket transforms; it does not measure every skin or shoe vertex. Check the visible grip with your model, especially thick gloves, large hands or unusual shoes.

For a short open-hi-hat/foot-close pattern, the explicit interpretation option drives a small left-foot opening/closing motion. It does not infer intent automatically or animate a physical cymbal gap. See [MIDI interpretation](../specs/architecture/midi-interpretation.md) for the option and retained source evidence.

The planner prefers **right-hand hi-hat and left-hand snare**, but may choose another hand to handle reach or timing. Explicit crossed/open-handed style selection, arm/stick collision avoidance and manual hand overrides remain follow-up work. A current preference is not a guarantee that the arms always cross naturally.

## Replace drum bodies while keeping hardware

The shared-hardware workflow keeps stands and pedals when switching drum presets or individual parts. Appearance choices change the shell/head/cymbal; finish, scale, rotation and hit response do not change the musical target or synthesized sound. Deliberately moving a contact point changes the kit layout and its related hardware placement.

For a local replacement, prepare a **body-only, self-contained static `.glb`**, in GLB 2.0 format with embedded resources and a maximum size of **25 MiB**. Skinned models and files without visible geometry are rejected. Convert GLTF, FBX, OBJ or BLEND assets to a self-contained static GLB first. Remove included stands, pedals and unrelated parts in a 3D editor before export. The importer cannot reliably distinguish arbitrary hardware from the playable component; otherwise duplicate hardware may appear. Automatic size fitting is approximate, so inspect the visible surface against its contact point.

Presets do not include every source model's original hardware. This common mounting approach also does not guarantee exact clamps, telescoping stands or photorealistic connections. Imported models retain their own license conditions. Appearance replacement does not import sound samples or change the current audio.

## Presets and hit feedback

Kit presets are **Studio, Vintage and Electronic** procedural geometry, plus **Innerscene, Batoski and smoj** models. The separate standard/deep-shell/compact choices change the selected procedural part's shape. [Bundled model credits](../public/models/README.md) and [asset research](research/drum-assets.md) record the external models and terms; these choices do not add new audio samples. Crash Left and Crash Right are separate physical pieces with their own contacts; switching a preset preserves that mapping.

In Drums, enable **타격 반응** (Hit response) and adjust **반응 강도** (Strength). Close drum editing to see the response during playback: editing and the operating system's reduced-motion preference suppress body movement. The effect follows hit velocity, while musical contact targets and timing remain fixed. It is visual feedback, not a physical simulation of the kit.

## Remaining limits

General humanoid GLB mapping, precise skin/sole calibration, automatic collision-free seating, a controlled short/tall VRoid test pair and project persistence remain open work. Hair/clothing dynamics still reset for repeatable seeking and output.

[Calibration and hardware verification](verification/calibration-unreleased.md) · [Palm orientation verification](verification/palm-orientation-unreleased.md) · [Rig compatibility design](../specs/architecture/rig-compatibility.md)
