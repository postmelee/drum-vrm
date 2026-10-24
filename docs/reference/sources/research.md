# Drummer motion research, tools and data

**English** · [한국어](research.ko.md)

This register preserves R01–R07 from the **2026-10-24** source audit. No learned drummer model, research motion data or code listed here is integrated merely because it is referenced. Full original audit details remain in the [historical research record](../../../mydocs/tech/research/2026-10-24/references.md).

## R01 — DRUMS: Drummer Reconstruction Using Midi Sequences

**Theodoros Kyriakou, Panayiotis Charalambous and Andreas Aristidou**, MIG 2025, 7 pages. [DOI](https://doi.org/10.1145/3769047.3769066), [author project](https://www.theodoroskyriakou.com/projects/drums/index.html), [author PDF](https://www.theodoroskyriakou.com/projects/drums/Paper.pdf).

The PDF identifies the paper as CC BY 4.0. That is not evidence of data/weight permission. The method combines BiLSTM hand trajectories, upper-body/expression retrieval, procedural feet and IK, using about 35 minutes of MMIP drum data (§3.1). Section 5 reports difficulty with complex fills/unseen patterns and dependence on a fixed kit; missing drum geometry prevented surface-contact accuracy evaluation.

Use the modular decomposition and evaluation limits as research guidance. Velocity/MIDI alignment is not actual stick-contact error. The audited project page exposed paper/DOI/video links but no confirmed executable-code or weight download. [CC BY 4.0 terms](https://creativecommons.org/licenses/by/4.0/).

## R02 — Multi-Modal Instrument Performances (MMIP)

[Official dataset](https://mmip.cs.ucy.ac.cy/), [license page](https://mmip.cs.ucy.ac.cy/License.html), [paper DOI](https://doi.org/10.1111/cgf.70025), [institutional paper record](https://zenodo.org/records/15830081).

Rights verification is partial: the official license page's search index stated CC BY-NC-SA 4.0 and a commercial-license section, but direct page access timed out. Download agreements, actual data and exceptions were not inspected. The Zenodo paper PDF is not the dataset itself or proof of dataset permission.

No MMIP motion clips/training data are adopted. Any later use needs exact files/version and terms for the intended training, distribution and output purposes. Do not infer all weights or rendered videos automatically inherit one license. [CC BY-NC-SA 4.0 explanation](https://creativecommons.org/licenses/by-nc-sa/4.0/).

## R03 — Implementation of a 3D Virtual Drummer

**Martijn Kragtwijk, Anton Nijholt and Job Zwiers**, Computer Animation and Simulation 2001, pp. 15–26. [DOI](https://doi.org/10.1007/978-3-7091-6240-8_2), [University of Twente record](https://research.utwente.nl/en/publications/implementation-of-a-3d-virtual-drummer/), [author PDF](https://research.utwente.nl/files/83238062/Kragtwijk2001implementation.pdf), [publisher](https://link.springer.com/chapter/10.1007/978-3-7091-6240-8_2).

Publisher rights identify © 2001 Springer-Verlag Wien. Access to an author copy does not establish reusable source code or an open license. The work discusses event classification, hand preference/alternation, preparation/contact/rebound and velocity/elasticity-dependent motion.

The author PDF §4.2.3 (PDF p. 8) explicitly says the shortest-path approach was not yet implemented. This project's beam planner is independent, and the paper's event deletion/transformation policies do not replace our source-event preservation. No reusable author source package/license was established.

## R04 — Generalized Audio-Driven Synthesis of Precise Drummer Motion

**Álvaro G. Iñesta, Mattia Ryffel, Amit H. Bermano, Robert W. Sumner and Martin Guay**. [Disney Research Studios announcement, 2026-08-18](https://studios.disneyresearch.com/2026/08/18/generalized-audio-driven-synthesis-of-precise-drummer-motion/), [arXiv 2608.19055v1, 2026-08-19](https://arxiv.org/abs/2608.19055v1), [DOI](https://doi.org/10.48550/arXiv.2608.19055).

The audio-conditioned diffusion work separates body plausibility and stick precision, including hit-distance and audio-motion measures. It is not a drop-in replacement for the MIDI engine. The paper's arXiv link uses the [nonexclusive distribution license](https://arxiv.org/licenses/nonexclusive-distrib/1.0/license.html); Disney limits paper reposting without permission. These are not broad rights to code, data or weights.

The audit confirmed paper/video links, not a licensed executable/weights/dataset package. Use the evaluation distinction without claiming the paper's reported precision as this app's result.

## R05 — MIDIAnimator

Audited commit: [059b1e2b80baf6171762f3d6a3c3f938d44ed46c](https://github.com/jamesa08/MIDIAnimator/tree/059b1e2b80baf6171762f3d6a3c3f938d44ed46c).

- [LICENSE.txt](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/LICENSE.txt): GPL version 3 text.
- [Python notice](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/MIDIAnimator/__init__.py#L1-L12): version 3 or later.
- The Tauri manifest declares `GPL-3.0`: [Tauri manifest](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/rust-impl/motionkeys/src-tauri/Cargo.toml), [Blender-derived curve notice](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/rust-impl/motionkeys/src-tauri/src/utils/fcurve.rs#L1-L4), [overlap calculation](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/rust-impl/motionkeys/src-tauri/src/utils/animation.rs#L514-L557).

The audited Rust code exists and computes MIDI/velocity keyframes and overlap handling, but its desktop/Blender context was not browser/full-body validated. Static analysis suggests the rest-value calculation can postpone a contact key; this is not an executed benchmark. Direct copying/translation is not adopted. Do not label the whole repository uniformly GPL-only/or-later or assume moving code into WASM resolves licensing.

## R06 — ARDrum-VirtualHand

Audited commit: [dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4](https://github.com/wayne0419/ARDrum-VirtualHand/tree/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4). Repository [Apache-2.0 license](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/LICENSE).

This Unity recording/learning system relates to MR.Drum/CHI 2025. Inspected files include [TransformRecorder](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/TransformRecorder.cs), [TransformPlayBacker](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/TransformPlayBacker.cs), and [HandMovementPathRenderer](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/HandMovementPathRenderer.cs).

Tracked hands/foot transforms are not full-body ground truth or a MIDI-to-body model. SDKs, Unity packages, models, audio and recordings were not all audited. It remains a candidate for recording/debug tools, not integrated code. Any adoption must preserve file-level attribution, changes, license and supplied NOTICE under [Apache-2.0 §4](https://www.apache.org/licenses/LICENSE-2.0).

## R07 — Groove MIDI Dataset (GMD)

The [Google Magenta source](https://magenta.tensorflow.org/datasets/groove) identifies Google LLC data as CC BY 4.0. It supplies performed MIDI and aligned synthesized audio, not full-body joints or hand assignments.

Artifact: [groove-v1.0.0-midionly.zip](https://storage.googleapis.com/magentadata/datasets/groove/groove-v1.0.0-midionly.zip). Published SHA-256: `651cbc524ffb891be1a3e46d89dc82a1cecb09a57c748c7b45b844c4841dcc1e`. The audit checked the official URL/hash, not a downloaded ZIP or its contents. The MIDI+WAV bundle is separate.

Roland TD-11 note mapping can differ from GM and includes CC4 hi-hat information. The public demo is independently generated; GMD remains a possible test input with per-file provenance, split, hash and transformations required on adoption. It is not a motion-naturalness ground truth.

[Sources](README.md)
