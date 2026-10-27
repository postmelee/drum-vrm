# Seat, contact points, and drum parts

**English** · [한국어](calibration.ko.md) · [Documentation](README.md) · [Character models](models.md) · [Open the app](https://postmelee.github.io/drum-vrm/?lang=en)

Fit the performer and kit by adjusting the seat and instrument contacts, then customize the visible drum bodies. This guide describes the repository build; see [version availability](../README.md#which-version-am-i-using) for the public app. The [calibration design](../specs/architecture/calibration.md) explains the shared measurements and solver.

## Fit a character to the kit

1. Load a supported VRM character through **Character**, or use the built-in skeleton. See the [model guide](models.md) for formats and limits. Both use measured bone lengths and the same solver; the app does not select a different animation implementation by avatar name.
2. New characters automatically prepare a pedal-aware seat and a body-fit kit candidate. A complete candidate is applied and the current MIDI is replanned. The completion message describes fitting, not guaranteed skin/clothing clearance. Inspect persistent warnings in **Analysis**.
3. **Automatic fit** repeats fitting; **Undo fit** restores the preceding setup. **Manual adjustment** exposes seat height **30–120 cm**, forward/back **−50–50 cm**, palm direction and default-layout reset. Positive forward moves the seated body toward the kit. **Auto-adjust seat only** changes the seat only. Reopening a saved project restores its recorded setup without automatic fitting.
4. Open **Drums**, select a component in the view or selector, and adjust its **X/Y/Z contact position** under **Manual layout / contacts** in centimetres. X is side-to-side (**−200–200 cm**, positive toward the player’s right), Y is height and Z is forward/back (**−50–250 cm**, positive forward). Closed/open hi-hat share the cymbal position. The hi-hat pedal is adjusted separately. Kick's contact is the foot pedal, not the kick drum's front head. Pedal contact height is limited to **4–20 cm** for the current shared hardware; other target heights allow **2–250 cm**. Press **Apply target** to use the edited position.
5. Inspect both feet at rest, then play the same MIDI and check the sticks at contact. Adjust seat and targets together until the important contacts are reachable. Seat position alone may not fit an avatar with very different arm and leg proportions.
6. Read the updated reach diagnostics. Unreachable hits keep their original audio and timing; the app does not lengthen bones or move notes to hide the problem. An unreachable pedal is reported even when the MIDI contains no pedal note. Revisit arbitrary times to inspect the calibrated pose.

Changes apply to the current session. Use [project save/open](projects.md) to preserve a setup across reloads; there is no autosave. A local review JSON records settings and measurements for troubleshooting and is separate from a reopenable project. Preparation, character loading and video export may temporarily disable calibration controls to keep the current operation consistent.

## Compare and keep a fitted layout

After loading a character, close the settings panel and select **Top**, **Front**,
**Left side**, **Right side** and **Player** above the scene. Left/right are the performer's
sides. Top and side views keep their direction; **Stage** permits free orbit.
Each view selection frames the current equipment without changing its placement.

Check the two rack toms together ahead of the snare, the floor tom on the right,
and the hi-hat's pedal linkage. Some overhead cymbal overlap is expected; use the
side views to inspect height and stick access. Compare the three public models
with the same drum preset. A completed fit is not a whole-motion collision pass.

**Drums → Hardware layout** switches between a floor tom on **Floor tom · three legs** and
**Suspended floor tom · ride stand** mounting. This is independent of the kit's appearance.
The suspended version removes the floor legs and uses a bounded clamp/rod
assembly on the ride stand; fitting can reject a setup that cannot support that
connection. Return to Floor tom · three legs to compare the default arrangement.

Loading another MIDI/demo, preparing it again, or seeking retains the accepted
kit and seat. Character/physical-equipment changes and explicit fitting/manual
adjustments may change them. Save a project to keep the setup across reloads.

## Try a demo score

In **Files / Audio → Demo**, choose a score under **Demo selection**, then press **Load**.

| Score | Purpose |
| --- | --- |
| Posture study · 12 s | A ready-to-ending comparison of groove, tom transfers, ride and simultaneous crashes; [review with public samples](models.md#compare-the-same-performance-across-models). |
| Studio Groove · 30 s | A short, independently authored groove for a quick playback check. |
| Interpretation check · 8 s | An independently authored pattern for count-in, hi-hat control, fills and explicit event inspection. |
| Transparent Girl 1 · 3 min 13 s | A full-song example for comparing body fitting, fast fills and transitions with the same score. |

The full-song demo loads explicit Logic Drum Kit Designer GM mapping, note-75 count-in and short open/close hi-hat interpretation at 20%. Its final extra snare is excluded from the arrangement so the ending uses **both crashes and kick**. The original MIDI bytes remain unchanged; inspect the retained note and omission in **Analysis**, or restore it through the note editor. The source has 1,760 positive note-ons; the configured arrangement has 1,720 strikes. This accounting does not certify natural posture or collision-free movement. No recorded WAV or VRM character is included.

The MIDI is included with permission for this project's public distribution, separately from the application's MIT license. See its [provenance](../public/demos/transparent-girl-1-provenance.json) for the exact file and permission scope, and the [MIDI contract](../specs/architecture/midi-interpretation.md#bundled-demo-selections) for the selection boundary. Other uploaded files remain local.

## Inspect a problem passage

In **Analysis → Pose and synchronization review**, enter an exact song time or move to the previous/next hit. **Inspect pose** shows actual tool/foot marker error and the engine's approximate torso/arm/stick clearance; a negative clearance is a possible proxy intersection, not a measured skin penetration.

**Review song** compares poses evaluated in time order and in a reproducibly shuffled order, using a 30 fps grid plus exact impacts and samples 60 ms before/after. It reports the maximum pose difference and sampled proxy intersections. Sampling cannot prove clearance between every pair of samples. Use the first reported time and several camera views to inspect visible behavior.

**Current stick paths** draws 37 actual hand-local tip-marker samples around the paused time, over at most 0.6 seconds. Left is orange and right is blue. The paths remain fixed until refreshed or hidden; they are inspection overlays, excluded from video export, not collision results.

**Save review JSON** downloads the current setup, body measurements, aggregate event counts, current pose/contact measurements and any completed review. It excludes uploaded file bytes and the source note stream. The report remains local unless you share it; check model/setup identifiers before publishing it. Changing the body/layout or preparing another performance clears the prior sampled result.

## Feet and hands

Feet follow the calibrated footboards around their rear hinges; the legs solve to those moving sole contacts. The kick beater and hi-hat upper cymbal follow the shared time-based mechanism. Playback, seeking and video output use the same evaluation. This is repeatable kinematics, not a complete heel-up, heel-toe, pedal-force or double-pedal model. See the [equipment contract](../specs/architecture/equipment-contact.md).

In Character → Manual adjustment, **Palm direction** selects **Natural / oblique** (the default), **Up**, **Inward / thumbs up**, or **Down**. The setting applies to both hands and is retained while changing the seat or importing another character in this session. It pauses playback, recalibrates the grip and invalidates an older video download.

The grip uses a diagonal hand-local stick socket and available finger bones. The solver follows each target with a neutral hand/forearm relationship and uses forearm roll for palm orientation. Hands normally wait above snare/hi-hat, or over the same piece between repetitions. Velocity affects lift and wrist motion; fast snare passages can use both available hands. These are common procedural rules, not complete push-pull, anatomical limits or collision avoidance.

These are palm orientations, **not complete traditional-grip styles**: traditional grip uses a different underhand left-hand finger arrangement and an overhand right hand. Matched grips commonly use downward or inward-facing palms; see [Yamaha’s grip guide](https://hub.yamaha.com/music-educators/instruments/perc/a-guide-to-proper-stick-grips/). Models without finger bones still load, with reduced grip detail. The contact inspector measures actual stick-tip and foot-socket transforms; it does not measure every skin or shoe vertex. Check the visible grip with your model, especially thick gloves, large hands or unusual shoes.

For a short open-hi-hat/foot-close pattern, the explicit interpretation option drives a small left-foot opening/closing motion. It does not infer intent automatically. The upper cymbal moves only when the selected model provides a compatible moving part. See [MIDI interpretation](../specs/architecture/midi-interpretation.md) for the option and retained source evidence.

The planner prefers **right-hand hi-hat and left-hand snare**, but may choose another hand to handle reach or timing. A current preference and skeletal proxy clearance checks do not guarantee that the arms always cross naturally or that skin stays clear.

Click a piano-roll note, or use **Note arrangement → Notes near current time**. The inspector lists all simultaneous source notes, including omitted notes. Choose **Auto / Left / Right** for a mapped hand strike, or **Omit this note** to omit it, then apply. Unchecking omission restores automatic interpretation; **Reset all note edits** clears the entire arrangement. Infeasible forced hands remain diagnosed rather than silently switching hands. Edits retain original MIDI/WAV and timing, use source-event IDs and can be saved in a [local project](projects.md). They change derived motion and synthesized preview, not an imported WAV recording.

## Replace drum bodies while keeping hardware

The shared-hardware workflow keeps common stands and pedals when switching drum bodies. Model, scale or rotation changes remeasure the body and automatically prepare a matching layout and performance; associated supports may move during fitting. Finish and hit response remain cosmetic. Original MIDI timing and synthesized sound stay unchanged. Inspect remaining path warnings in **Analysis**; successful static fitting does not guarantee a clear moving stick. See the [equipment/contact contract](../specs/architecture/equipment-contact.md).

For a local replacement, prepare a **body-only, self-contained static `.glb`**, in GLB 2.0 format with embedded resources and a maximum size of **25 MiB**. Skinned models and files without visible geometry are rejected. Convert GLTF, FBX, OBJ or BLEND assets to a self-contained static GLB first. Remove included stands, pedals and unrelated parts in a 3D editor before export. The importer cannot reliably distinguish arbitrary hardware from the playable component; otherwise duplicate hardware may appear. Automatic size fitting is approximate, so inspect the visible surface against its contact point.

Presets do not include every source model's original hardware. This common mounting approach also does not guarantee exact clamps, telescoping stands or photorealistic connections. Imported models retain their own license conditions. Appearance replacement does not import sound samples or change the current audio.

## Presets and hit feedback

Kit presets are **Studio, Vintage and Electronic** procedural geometry, plus **Innerscene, Batoski and smoj** models. The separate standard/deep-shell/compact choices change the selected procedural part's shape. [Bundled model credits](../public/models/README.md) and [asset research](research/drum-assets.md) record the external models and terms; these choices do not add new audio samples. Crash Left and Crash Right are separate physical pieces with their own contacts; switching a preset preserves that mapping.

In Drums, enable **Hit response** and adjust **Strength**. Close drum editing to see the response during playback: editing and the operating system's reduced-motion preference suppress body movement. The effect follows hit velocity, while musical contact targets and timing remain fixed. It is visual feedback, not a physical simulation of the kit.

## Remaining limits

General humanoid GLB mapping, precise skin/sole calibration, automatic collision-free seating and a controlled short/tall VRoid test pair remain open work. Hair/clothing dynamics still reset for repeatable seeking and output.

[Calibration and hardware verification](verification/calibration-unreleased.md) · [Palm orientation verification](verification/palm-orientation-unreleased.md) · [Rig compatibility design](../specs/architecture/rig-compatibility.md)
