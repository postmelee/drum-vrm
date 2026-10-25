# Motion model and inspection-tool adoption gates

Review: 2026-10-25. Scope: #21 model/data adoption and the #23 inspection-tool
comparison. This is an engineering decision record, not a reproduced learned-model
experiment. [R01–R07](references.md) retain the original authors, source identities
and earlier rights findings; this follow-up records what was rechecked and how to
decide the next implementation step.

## Recommendation for the browser-performance milestone

Continue the independent deterministic planner and measured-body correction path.
Fix physical kit clearance, coordinated arm/torso routes and phrase assignment before
introducing a learned motion prior. A learned prior may improve preparation and
style, but does not replace source-event preservation, per-avatar fitting, contact
constraints or collision inspection. Current difficulties are observable geometric
and scheduling failures, not evidence that training is necessary.

Implement the minimum browser trace tooling independently. Reuse ARDrum's conceptual
distinction between motion recording, replay and tip-path inspection, with credit,
but do not translate its Unity code or import its SDKs, clips or recorded sessions.
No external code, weights, motion, scores or dataset files are added by this review.

## Rechecked candidates and boundaries

| Candidate | Evidence checked | Decision for this milestone |
| --- | --- | --- |
| R01 DRUMS, Kyriakou / Charalambous / Aristidou | [Author page](https://www.theodoroskyriakou.com/projects/drums/index.html) still links paper/video/DOI, with no execution package or weights found there. [Paper](https://www.theodoroskyriakou.com/projects/drums/Paper.pdf), p.1: CC BY 4.0 for the paper; §5: fixed-layout dependence and no kit geometry for contact verification. | Keep modular hand/body/pedal and separate evaluation ideas. Do not claim the published model solves our variable-kit/body collision problem. Code, weights and training-data grants are not established by the paper license. |
| R02 MMIP | [Official license page](https://mmip.cs.ucy.ac.cy/License.html) search result still states CC BY-NC-SA 4.0; direct access timed out again. No download agreement or individual data files inspected. | No product data/clip/training adoption. The earlier partial-verification status remains. A research-only trial would need actual artifact terms and a recorded permitted use first. |
| R04 Disney, Iñesta / Ryffel / Bermano / Sumner / Guay | [Official page](https://studios.disneyresearch.com/2026/08/18/generalized-audio-driven-synthesis-of-precise-drummer-motion/) and [arXiv v1](https://arxiv.org/abs/2608.19055v1) describe audio-driven diffusion and separate body/contact objectives. Official links expose PDF/video; no executable code, checkpoints or data license located. Disney reserves rights and restricts reposting. | Cite the separation of spatial precision and timing metrics. Do not replace exact MIDI timing with audio inference or advertise its reported precision as ours. No adoption experiment is executable from the confirmed materials. |
| R06 ARDrum-VirtualHand | Repository `main` resolves to the previously reviewed [`dafb6b1`](https://github.com/wayne0419/ARDrum-VirtualHand/tree/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4). [LICENSE](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/LICENSE) is Apache-2.0. Specific recorder/playback/path files inspected below. | Useful inspection reference; choose independent browser implementation. Repository-level licensing is not an audit of SDKs, samples, recorded performances or participant permissions. |
| R07 Google Magenta GMD | [Official page](https://magenta.withgoogle.com/datasets/groove) confirms CC BY 4.0, v1.0.0 downloads/hash, TD-11-specific note mapping and CC4 data. | Suitable future MIDI/controller diversity input after an explicit TD-11 profile; not hand-selection or whole-body ground truth. No archive downloaded. Do not substitute Magenta drum-pattern generation for preserved-input animation. |

The absence of a link in these inspected sources is not proof that no private or
future package exists. Reopen the acquisition check when authors provide a concrete
artifact. Paper, code, weights, data, assets and output permissions require separate
entries; none inherits a grant solely because another component is open.

## ARDrum: selective port versus independent trace tooling

The inspected source files are pinned to the commit above:

- [TransformRecorder.cs](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/TransformRecorder.cs): captures three transforms, timestamps and per-instrument hit values. Its foot channel can be overwritten with an animation clip, and triggered limb labels begin as unspecified. Treat this as a transform log with authored processing, not pristine full-body mocap.
- [TransformPlayBacker.cs](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/TransformPlayBacker.cs): loads its Unity-oriented recording structure for replay.
- [HandMovementPathRenderer.cs](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/HandMovementPathRenderer.cs): obtains tool-tip paths from recorded transforms and local tip anchors, then highlights path landmarks. Those landmarks do not establish drum contact or collision clearance.
- [Package manifest](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Packages/manifest.json): includes Meta XR 65, Unity animation rigging/input/rendering and MIDI packages. They are not dependencies of our browser trace feature.

| Work | Selective source port | Independent implementation |
| --- | --- | --- |
| Clock and identity | Replace Unity frame time and instrument fields with immutable source IDs and song time. | Use the existing absolute-time engine and hit identity directly. |
| Pose and contact | Adapt three tracked transforms and Unity anchors to four limbs, semantic VRM tools and shared world coordinates. | Sample existing rig attachment markers after applying each pose; retain both desired and rendered points. |
| Replay and inspection | Replace scene/input/file/LineRenderer behavior; validate recording interpolation and timestamp correspondence. | Re-evaluate any recorded song time and compare bounded line traces/diagnostic samples. |
| Verification | New browser tests remain necessary; the source does not supply our body/layout or seek invariants. | The same tests can directly exercise the product's existing boundaries. |
| Rights and maintenance | Track every copied file/change and retained notice; audit any imported SDK/clip/session separately. | Maintain source-credit links for ideas; no imported-code dependency or SDK migration. |

This is a qualitative dependency/coupling comparison, not a measured delivery-time
estimate. Both options require most of our product-specific work, so a port offers
little reduction in the current scope. If a later selective port is chosen, preserve
the Apache-2.0 license copy, relevant notices and modification notices under
[§4](https://www.apache.org/licenses/LICENSE-2.0). No standalone NOTICE was found in
the inspected tree; that does not waive notices supplied with a future artifact.

## Minimum trace needed before model selection

Store a local, versioned setup snapshot and bounded sampled observations. Keep uploaded
assets/private event streams out of the public repository. A useful record includes:

1. Build/planner version, input fingerprint, selected interpretation, body measurements,
   seat/layout and fit state, sample song time and associated source-event identity.
2. Assigned limb, intended surface/region, desired contact, **rendered** stick/sole
   marker, Euclidean contact error and actual temporal alignment.
3. Whole-arm/torso/stick proxy clearance, support observations, bone-length error and
   unresolved constraints. Mark proxy geometry and sampling coverage explicitly.
4. Sequential-versus-shuffled evaluation differences and local WAV start offset.
   Match arbitrary seeks against the same song clock; do not infer motion quality
   from an encoder completing or from a target compared against itself.

Velocity, acceleration and jerk estimates can reveal discontinuities, but require the
sample rate/filter to be recorded. Motion-speed peaks are not proof of stick/surface
contact. Skin/clothing observations and semantic playing plausibility remain separate
from numerical proxy distances. For this milestone browser playback is the acceptance
surface; advanced video output is deferred.

## Gate for a future learned or recorded candidate

| Gate | Evidence required before passing |
| --- | --- |
| Acquisition | Downloadable runnable code/checkpoint or motion file; fixed version/hash, preprocessing and expected rig/kit/time conventions. |
| Rights | Separate code/data/weights/assets grants; participant/session provenance as applicable; permitted research, product distribution and generated-output uses. Unknown terms remain unknown. |
| Residual benefit | A reproducible baseline failure on held-out phrases/bodies that the proposed model plausibly addresses; no change to source note times or silent omission of events. |
| Contact correction | Candidate trajectories feed a constrained body/kit solver. Compare raw and corrected motion; retain original timing and diagnose unreachable contacts instead of stretching bones. |
| Quality and runtime | Improved blinded visual assessment plus no regression in contact, clearance, support, continuity, deterministic seek, browser latency or memory on the same fixtures. |
| Distribution | Only approved artifacts enter notices and deployment; a local research experiment is not automatic product inclusion. |

The first executable comparison should be independently authored single/double/tom
fills and crossed-hand transitions, then the private corrected benchmark, several
body proportions and changed layouts. Keep candidate outputs local and compare to
the procedural baseline with identical contacts, timing and WAV. This protocol is
specified, not yet run for DRUMS/Disney/MMIP: confirmed runnable artifacts and rights
are missing. The research/adoption decision can be reviewed now; it must not be
reported as successful model integration or completion of those experiments.
