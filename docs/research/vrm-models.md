# VRM acquisition and metadata review

Reviewed: **2026-10-24 (Asia/Seoul)**. Repository baseline: `28a50a2c19c2b5109c490384f0b03e554f4dea1e` (documentation/release-policy PR #31). Public application baseline remains v0.2.0, `3e8be9333cedbc9845dbc3e4cebd73961af697b4`.

Original acquisition-review scope: official-source acquisition guidance and static inspection of locally downloaded GLB/VRM metadata and rest transforms for #17/#18/#30. **That acquisition step did not perform application compatibility testing, feature implementation, third-party model publication or release.** The later implementation/preflight record below is separate and remains unreleased. Model files and full extracted metadata are confined to ignored `.local/vrm-test-models/`. Public automated fixtures must remain independently generated.

User guidance: [English](../models.md) · [한국어](../models.ko.md). Architecture context: [rig/motion audit](rig-and-studio-audit.md).

## Source and version findings

| Candidate/source | Verified acquisition/version facts | Intended role and limits |
|---|---|---|
| [VRM Consortium samples](https://github.com/vrm-c/vrm-specification/tree/94e82dd346fa6cf0337c4421728640e5252dd38e/samples) | Official index distinguishes practical avatars from individual feature tests | Choose an actual humanoid for rig checks. A material/expression fixture is not automatically a useful drummer. Inspect each model's own notice and metadata. |
| [Twist sample in three-vrm](https://github.com/pixiv/three-vrm/blob/3caf7a21ee68f0d5dfa363ab3d577450bbf60136/packages/three-vrm/examples/models/VRM1_Constraint_Twist_Sample.vrm) | Downloaded VRM 1.0; bytes match the [Consortium copy](https://github.com/vrm-c/vrm-specification/blob/94e82dd346fa6cf0337c4421728640e5252dd38e/samples/VRM1_Constraint_Twist_Sample/vrm/VRM1_Constraint_Twist_Sample.vrm) exactly | Non-identity rest rotations and node constraints make it a useful later compatibility case. The model is not licensed under the surrounding library's MIT license. |
| [Seed-san](https://github.com/vrm-c/vrm-specification/tree/94e82dd346fa6cf0337c4421728640e5252dd38e/samples/Seed-san) | Downloaded VRM 1.0 with a different skeleton/profile and constraint set | Additional official body/optional-bone case; credit is required by its embedded metadata. |
| [AvatarSample_A on Hub](https://hub.vroid.com/en/characters/2843975675147313744/models/5644550979324015604) | The specified official page says **VRM 0.0**, checked on the review date | Page/conditions reviewed; Hub binary not downloaded. Do not label it a verified VRM 1.0 fixture. |
| [AvatarSample_B on Hub](https://hub.vroid.com/en/characters/7939147878897061040/models/2292219474373673889) | The specified official page also says **VRM 0.0** | Same limitation. A model name alone does not identify format/version. |
| [AvatarSample_B in pixiv's archived ChatVRM](https://github.com/pixiv/ChatVRM/blob/b542aa00e19dccf9fc48ba340cf7eee011d2329a/public/AvatarSample_B.vrm) | Separately downloaded official public artifact; embedded VRM 1.0, model version 1.1, generator VRoid Studio 1.22.0 | A useful initial candidate with fingers and no node-constraint extension. This is a different artifact from the Hub 0.0 listing, not a conversion performed here. |
| [VRoid Studio](https://vroid.com/en/studio) | Official authoring tool; the [standard exporter](https://vroid.pixiv.help/hc/en-us/articles/15760756822297-I-want-to-learn-more-about-the-VRM-export-feature) explicitly offers VRM 0.0/1.0 | Recommended controlled two-proportion workflow from editable `.vroid` sources. No Studio-generated pair has been created or tested in this review. |
| [VRoid Hub](https://hub.vroid.com/en/) / [BOOTH](https://booth.pm/) | Discovery platforms requested by the owner | Evaluate individual downloads and terms. No blanket license, format compatibility, or third-party listing approval is inferred from the platform or a zero price. |

The GitHub connector cannot download binary resources. Public, commit-pinned raw files were fetched directly without login or access-control bypass. Hub's account/consent flow was not bypassed. Local acquisition records include source URLs, byte sizes, and SHA-256 in `acquisition.json`; extracted GLB JSON and the static summary are stored alongside them.

## File identities and inspection results

Source commits:

- three-vrm: `3caf7a21ee68f0d5dfa363ab3d577450bbf60136`.
- vrm-specification: `94e82dd346fa6cf0337c4421728640e5252dd38e`.
- ChatVRM: `b542aa00e19dccf9fc48ba340cf7eee011d2329a`.

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| Twist, both official copies | 10,776,032 | `12c2b97e95e700783a6a550dc0eee2d7880aeedccef9ae67bc4c5a2f0f2631a2` |
| Seed-san | 10,917,800 | `624d0d554bc205bbdc33e22a68a2c3c20edebb3e573011ead8878a65e5329b23` |
| AvatarSample_B, ChatVRM copy | 21,047,556 | `ffbd8c92a9e67c0a948f69c7a2eec91e5c282c9ae70e9184309fc164d74cbc27` |

GLB header/chunk parsing and JSON inspection found the following. Required entries refer to the 15 required semantic bones in the [VRM 1.0 humanoid specification](https://github.com/vrm-c/vrm-specification/blob/94e82dd346fa6cf0337c4421728640e5252dd38e/specification/VRMC_vrm-1.0/humanoid.md). Presence is not a formal full-file validation or proof of correct deformation.

| Observation | Twist | Seed-san | AvatarSample_B, ChatVRM |
|---|---:|---:|---:|
| `VRMC_vrm.specVersion` | 1.0 | 1.0 | 1.0 |
| Nodes / skins | 171 / 3 | 147 / 5 | 138 / 3 |
| Semantic humanoid bone entries | 54 | 51 | 54 |
| Missing required bone entries | 0 | 0 | 0 |
| Finger bones, left / right | 15 / 15 | 15 / 15 | 15 / 15 |
| Human nodes with non-identity local rest rotation | 28 | 51 | 36 |
| Nodes with `VRMC_node_constraint` | 14 | 23 | 0 |
| External buffer/image URIs | 0 | 0 | 0 |

All three also use MToon and spring-bone extensions. Their scene/node transforms were composed to measure rest-pose joint positions. The left upper arm/hand is at positive X in all three, consistent with the official +Z-facing convention and the migration concern in the rig audit.

| Measured left-side segment, metres | Twist | Seed-san | AvatarSample_B, ChatVRM |
|---|---:|---:|---:|
| Upper arm | 0.219853 | 0.255073 | 0.245305 |
| Forearm | 0.214685 | 0.219944 | 0.219359 |
| Upper leg | 0.352984 | 0.342469 | 0.343157 |
| Lower leg | 0.415494 | 0.388121 | 0.386029 |

These are distances between declared bone nodes, not mesh height, sole height, physical anthropometry, or validated IK settings. They demonstrate why the current fixed 0.43 m segments cannot simply be assigned to imported models. Twist and Seed-san offer different proportions, but they do not replace a controlled two-export experiment from one editable base model.

## Conditions and rights evidence

The [Twist notice](https://github.com/vrm-c/vrm-specification/blob/94e82dd346fa6cf0337c4421728640e5252dd38e/samples/VRM1_Constraint_Twist_Sample/README.md) identifies VRM Public License 1.0 and pixiv's 2022 copyright. [Seed-san's notice](https://github.com/vrm-c/vrm-specification/blob/94e82dd346fa6cf0337c4421728640e5252dd38e/samples/Seed-san/README.md) identifies the same license family and VirtualCast. Under the [license text](https://vrm.dev/licenses/1.0/), the embedded settings are part of the conditions. A license-family name alone is insufficient.

| Embedded field | Twist | Seed-san | AvatarSample_B, ChatVRM |
|---|---|---|---|
| Author | pixiv Inc. | VirtualCast, Inc. | VRoid Project |
| `licenseUrl` | VRM Public License 1.0 | VRM Public License 1.0 | VRM Public License 1.0 |
| `avatarPermission` | `everyone` | `everyone` | `everyone` |
| `commercialUsage` | `corporation` | `corporation` | `corporation` |
| `creditNotation` | `unnecessary` | **`required`** | `unnecessary` |
| `allowRedistribution` | `true` | `true` | `true` |
| `modification` | `allowModificationRedistribution` | `allowModificationRedistribution` | `allowModificationRedistribution` |
| `allowAntisocialOrHateUsage` | `false` | `true` | `false` |
| `otherLicenseUrl` | absent | absent | VRoid sample-model conditions |

Other usage flags and complete metadata remain in the local inspection report. These are observed field values, not a promise that any proposed use satisfies all terms. In particular, additional conditions must not disappear behind a permissive-looking enum.

[AvatarSample A/B and VRoidPreset A–Z conditions](https://vroid.pixiv.help/hc/en-us/articles/4402394424089-VRoidPreset-A-Z) permit commercial and noncommercial use, including videos, without required credit. They are **not CC0**. The conditions prohibit CC0 relabeling, fee redistribution of the supplied sample data, and certain character-creation-service uses; modified-character examples and additional conduct restrictions are described in the original terms. Preserve that link when selecting or modifying a sample. The [official sample classification](https://vroid.pixiv.help/hc/en-us/articles/4402614652569-Do-VRoid-Studio-s-sample-models-come-with-conditions-of-use) separately lists older CC0 samples; those names must not be confused with A/B/C.

Private evaluation, publishing rendered videos, and redistributing the actual model are different actions. The project currently chooses local-only acquisition even for a model whose settings permit redistribution. Nothing is placed in `public/`, tests, GitHub Releases, or Pages. Future public demo assets require an explicit per-artifact decision and notices. The repository's MIT license and three-vrm's code license do not relicense avatars.

## Controlled two-proportion workflow

Recommended experiment, not completed evidence:

1. Start from a `.vroid` source the user may edit. Prefer a new base or an explicitly permitted preset, with clear clothing/texture provenance. Retain the source file and the applicable terms.
2. Save two copies, for example `drummer-short.vroid` and `drummer-tall.vroid`. Change body proportions within the editor's normal controls while keeping outfit and export settings comparable. Record the values and Studio version. This is an engineering experiment design, not an official preset recipe.
3. Export each as **VRM 1.0**, with author/model information and rights metadata. Keep enough hand/finger bones for grip verification; inspect the export rather than assuming every reduction option preserves them. See the [official export guide](https://vroid.pixiv.help/hc/en-us/articles/15760756822297-I-want-to-learn-more-about-the-VRM-export-feature) and [save shortcuts](https://vroid.pixiv.help/hc/en-us/articles/900006050066-I-want-to-learn-more-about-keyboard-shortcuts-in-VRoid-Studio-for-Windows-and-macOS).
4. Retain `.vroid` as the parametric editing source and `.vrm` as the runtime artifact. The newer [dress-up workflow can accept VRM](https://vroid.pixiv.help/hc/en-us/articles/39513464329881-What-is-VRM), but it does not make a downloaded `.vrm` a replacement for its original editable body-parameter source.
5. Once the adapter is implemented, use the same MIDI/event times on both models. Record seat placement, hand-local grip, rendered stick-tip contact, feet/pedals, reach diagnostics, arbitrary-seek reproducibility, and fixed-time export. Compare numerical segment lengths, not only apparent height or clothing.

[VRoid's model-use guidance](https://vroid.pixiv.help/hc/en-us/articles/4405813333657-Can-I-use-the-models-created-with-VRoid-Studio-Stable-Ver-for-commercial-purposes) distinguishes creative output from separately restricted character-making tools and preserves third-party/special asset conditions. This task concerns animation and video output, not an avatar authoring/export service; any later model-generation or redistribution feature needs its own review.

## Proposed model information UI and acceptance boundary

For #17, display the detected VRM version, model name/version/author, required-bone validation, finger capability, and model terms before calling a model usable. Link `licenseUrl` and `otherLicenseUrl`; preserve unknown/missing information as unknown. Keep private import, video publishing, and model redistribution distinct in the explanation. Do not equate successful parsing or all required entries being present with a verified drummer.

For #30/#18, acceptance evidence must come from the actual rendered stick and foot transforms after rest-basis conversion and calibration. It must include two proportions, exact original hit times, pause/seek order independence, and export consistency. Twist/Seed-san constraint behavior and secondary spring motion need separate treatment from the deterministic primary pose.

The original review supplied acquisition provenance and candidates only. Its results did not establish constraint evaluation, materials, skinning, grip, retargeting or export compatibility. Subsequent checks are recorded separately below.


## Unreleased implementation and input review — 2026-10-24

Branch: `codex/vrm-humanoid-grip`. This section records implementation, preflight and the later limited browser evidence below, not a deployment. Public v0.2.0 is unchanged. No third-party avatar files were added to Git, public assets, release attachments or automated public fixtures.

`src/vrm-import.ts` inspects a local self-contained VRM 1.0 GLB before GLTFLoader runs. The application limits a file to 50 MiB, nodes to 10,000 and embedded PNG/JPEG images to 64, with each dimension at most 4096 and total decoded pixels at most 64,000,000. PNG/JPEG headers are inspected without image decoding. Buffer-view ranges, external URI references, GLB chunks, required/duplicate/out-of-range human-bone references, hierarchy cycles and nonfinite/malformed node transforms are rejected. The loading manager independently rejects external resource requests. Header inspection is a resource-budget check, not a complete image or glTF validator; actual decoding/loading may still fail.

For context, the private original samples contain 29,687,936 PNG pixels in 33 images (ChatVRM AvatarSample_B), 22,347,776 in 19 images (Twist), and 10,813,440 in 15 images (Seed-san). Maximum sides are 2048, 2048 and 1024 respectively. All pass the new image budget without modifying the source. Compressed file size alone would not establish these decoded-memory bounds.

The importer identifies VRM 0.x and rejects it with instructions to re-export VRM 1.0. The official [UniVRM migration documentation](https://vrm.dev/en/univrm1/migrate_vrm0/) describes an upgrade route; it does not make a model's permissions or appearance equivalent after conversion. No conversion was performed here.

### Metadata behavior

The panel uses text nodes for model-supplied strings. File metadata and references are retained; links are not automatically visited or used as texture sources. The application code's MIT and three-vrm license do not alter avatar terms. Model name, author, model version, references, copyright, third-party conditions and additional-license fields must remain distinguishable from technical compatibility status.

The pinned [VRM 1.0 metadata specification](https://github.com/vrm-c/vrm-specification/blob/94e82dd346fa6cf0337c4421728640e5252dd38e/specification/VRMC_vrm-1.0/meta.md) defines restrictive defaults for omitted usage settings: author-only avatar use, personal non-profit commercial scope, required credit, no redistribution or modification, and false content-permission flags. The inspector labels omission and its specification default separately; an explicitly supplied value is preserved. Missing name/author information or a nonstandard/missing license URL produces metadata warnings in the inspection result. These warnings do not bypass the downstream VRM loader's validation, nor certify model rights. Additional conditions such as AvatarSample_B's `otherLicenseUrl` remain relevant even beside permissive enum values.

### Completed checks and open evidence

- `npx vitest run tests/vrm-import.test.ts`: 11 tests passed, using independently generated JSON/GLB/image headers. They cover preserved metadata/defaults, malformed-rights warnings, VRM 0.x rejection, external URI rejection without fetch, file/header/transform/hierarchy/bone-reference failures, and image count/dimension/aggregate/range failures.
- `npx vitest run .local/vrm-budget.test.ts`: one ignored local test passed for the three pinned private artifacts. All report VRM 1.0, 15 required bones, 15 finger bones per hand and no metadata warnings. This check calls `inspectVRM` only; it does not parse the renderable scene or claim animation/export compatibility.
- `npm run typecheck`: passed after the importer changes. Later branch-wide changes require the normal integration checks.

The working application adds model/skeleton visibility, a default semantic-bone rig with fingers, hand-local sticks, actual socket/contact error readout and restoration to the default rig. Limb lengths come from the imported normalized humanoid; the current seat/grip policy is application-defined. The VRM adapter resets spring state and updates with zero elapsed time per evaluated pose, avoiding accumulated secondary dynamics. This policy does not simulate physical hair/clothing motion. The browser harness results below cover actual model/material loading, assigned-contact retargeting and arbitrary-seek repeatability for the inspected files. Main-app UI interactions and encoded video output still require separate integration results. A controlled two-export VRoid body pair has still not been created.


### Actual browser harness results — 2026-10-24

The local harness `.local/vrm-review.html` used the application's `loadLocalVRM`, `VRMHumanoidRigAdapter` and Rust/WASM evaluator, with real model/material loading and a WebGL scene. Full numeric output is retained privately in `.local/vrm-browser-evidence.txt`; the input files match the SHA-256 identities above. No texture-stripped test copy was used for this browser run. This is a renderer/pose audit, not a video-encoding audit or a claim of complete performance coverage.

Each file was evaluated at 128 fixed 30 fps frame timestamps, then those same timestamps were revisited in alternating low/high order. The audit compared the pose array and raw humanoid world matrices, including actual hand-local stick markers and foot sole sockets. All values were finite, transforms were positive, and the original 14 hit events/times remained unchanged.

| Original file | Assigned hits | Maximum assigned-hit contact error, m | Maximum frame marker/pose error, m | Maximum seek matrix difference | Maximum frame pose difference |
|---|---:|---:|---:|---:|---:|
| Twist | 11 / 14 | 1.501937966314068e-7 | 1.5507947017802673e-7 | 0 | 0 |
| ChatVRM AvatarSample_B | 11 / 14 | 1.3622455557686942e-7 | 1.515037591555418e-7 | 0 | 0 |
| Seed-san | 11 / 14 | 3.89007133075741e-7 | 4.3199744038452377e-7 | 0 | 0 |

Every kick hit was assigned. In each model, note 41 at 1.50 s (low floor tom), note 49 at 1.75 s (crash) and note 51 at 2.00 s (ride) remained explicitly unresolved/unreachable for the current kit/body layout. They were not silently shifted, removed or counted as performed. Consequently the harness pass means that assigned contacts and repeated poses met the audit's checks; it does not mean all 14 hits were successfully played. Changing kit placement or calibration requires a new reach/pose check.

The run exercised Twist's 14 and Seed-san's 23 node constraints; AvatarSample_B has none. Spring joints were loaded (57, 19 and 58 respectively) under the adapter's reset/zero-delta policy, without accumulated dynamic simulation. Small socket errors establish the numerical tool-contact contract for these samples, not skin/finger surface contact, collision-free clothing, correct shoe-sole geometry or physical realism. Matrix equality was measured in this browser run and does not establish bitwise identity across browsers or machines.

Main-app import/reset controls and an encoded video export with an imported avatar are still pending separate verification. The two differently proportioned Studio exports from one editable source have not been made; these three official examples do not replace that controlled experiment. The development branch remains unreleased and the public application remains v0.2.0.


Actual app follow-up: the Twist sample loaded with metadata, played with the demo, supported model/skeleton visibility and player view, and completed the 923-frame audio/video encoding path. A newly downloaded blob was not independently inspected; see the exact [verification scope and limitation](../verification/vrm-studio-unreleased.md). No release or Pages deployment occurred.
