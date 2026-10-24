# Seat, contact points, and drum parts

**English** · [한국어](calibration.ko.md) · [README](../README.md) · [Character models](models.md)

Status: **implemented and locally verified, not released** — 2026-10-24. This guide describes the seat/target, planted-foot, grip and shared-hardware workflow implemented after PR #33. Public v0.2.0 on Pages does not include these controls. See the [verification record](verification/calibration-unreleased.md) for executed checks and limits.

## Fit a character to the kit

1. Load a VRM 1.0 character through **캐릭터** (Character), or use the built-in skeleton. See the [model guide](models.md) for formats and limits. Both use measured bone lengths and the same solver; the app does not select a different animation implementation by avatar name.
2. Start with automatic seat fitting. In Character settings, adjust **seat height** from **30 to 120 cm** and **forward/back position** from **−50 to 50 cm**. Positive forward values move the seated body toward the kit. The automatic action recalculates a starting seat for the current body and pedal arrangement.
3. Open **드럼** (Drums), select a component in the view or selector, and adjust its **X/Y/Z contact position** in centimetres. X is side-to-side, Y is height and Z is forward/back. Closed/open hi-hat share the cymbal position. The hi-hat pedal is adjusted separately. Kick's contact is the foot pedal, not the kick drum's front head. Pedal contact height is limited to **4–20 cm** for the current shared hardware; other target heights allow **2–250 cm**.
4. Inspect both feet at rest, then play the same MIDI and check the sticks at contact. Adjust seat and targets together until the important contacts are reachable. Seat position alone may not fit an avatar with very different arm and leg proportions.
5. Read the updated reach diagnostics. Unreachable hits keep their original audio and timing; the app does not lengthen bones or move notes to hide the problem. An unreachable pedal is reported even when the MIDI contains no pedal note. Revisit arbitrary times to inspect the calibrated pose.

Changes apply to the current session and are **not saved across reloads**. Keep notes of useful settings until project persistence is available. Preparation, character loading and video export may temporarily disable calibration controls to keep the current operation consistent.

## Feet and hands

Feet start on the calibrated kick and hi-hat pedal contacts. During a hit, the ankle stays in place and the foot makes a small toe-up preparation before pressing back to the contact. The same time-based evaluation is used for playback, seeking and video output. This simple ankle-only motion does not reproduce heel-up, heel-toe or double-pedal technique, and it is not a mechanical pedal simulation.

The grip update aligns the stick across the palm and curls available finger bones around it. Models without finger bones still load, with reduced grip detail. The contact inspector measures actual stick-tip and foot-socket transforms; it does not measure every skin or shoe vertex. Check the visible grip with your model, especially thick gloves, large hands or unusual shoes.

The planner already prefers **right-hand hi-hat and left-hand snare**, but may choose another hand to handle reach or timing. Explicit crossed/open-handed style selection, arm/stick collision avoidance and manual hand overrides remain follow-up work. A current preference is not a guarantee that the arms always cross naturally.

## Replace drum bodies while keeping hardware

The shared-hardware workflow keeps stands and pedals when switching drum presets or individual parts. Appearance choices change the shell/head/cymbal; finish, scale, rotation and hit response do not change the musical target or synthesized sound. Deliberately moving a contact point changes the kit layout and its related hardware placement.

For a local replacement, prepare a **body-only, self-contained static `.glb`**, with embedded resources and a maximum size of **25 MiB**. Remove included stands, pedals and unrelated parts in a 3D editor before export. The importer cannot reliably distinguish arbitrary hardware from the playable component; otherwise duplicate hardware may appear. Automatic size fitting is approximate, so inspect the visible surface against its contact point.

Presets do not include every source model's original hardware. This common mounting approach also does not guarantee exact clamps, telescoping stands or photorealistic connections. Imported models retain their own license conditions. Appearance replacement does not import sound samples or change the current audio.

## Remaining limits

General humanoid GLB mapping, precise skin/sole calibration, automatic collision-free seating, a controlled short/tall VRoid test pair and project persistence remain open work. Hair/clothing dynamics still reset for repeatable seeking and output. Release tags and Pages deployment require separate approval; main or a PR changing does not update the public site.
