# Preparing character models

[한국어](models.ko.md) · [README](../README.md) · [Engineering evidence](research/vrm-models.md)

Checked on **2026-10-24**. The current development branch adds local **VRM 1.0** character import for #17/#18/#30. It is **unreleased**: the public v0.2.0 app still uses its built-in skeleton and cannot import VRM characters. No third-party avatar is bundled. Passing format inspection does not establish animation, contact or video-export compatibility; the verification status below distinguishes those checks.

## Import in the development build

1. Open the **Character / 캐릭터** panel and choose **Import VRM / VRM 불러오기…**. Select one self-contained `.vrm` file of at most **50 MiB**. Textures and buffers must be inside the file. Files stay in the browser; the application does not upload the avatar.
2. The importer checks VRM 1.0, the 15 required semantic bones, bone references/hierarchy, finite transforms and image budgets before loading. It supports embedded PNG/JPEG images, at most 64 images, at most 4096 pixels on either side and 64 million pixels across all images. These are application limits, not a guarantee that every device can comfortably render the largest accepted model.
3. Read the displayed model name, author, finger-bone counts and **Model usage conditions / 모델 이용 조건**. The rights fields and additional conditions come from the file. Missing optional rights fields are labeled with the VRM 1.0 specification default; they are not treated as permission. Metadata links are not opened or fetched automatically.
4. Load the demo or a MIDI file and inspect the model and skeleton using their separate visibility controls. The adapter measures the model's limb lengths, uses its semantic hierarchy and attaches sticks to its hands. Finger pose is applied only where finger bones are present. The contact-error display measures the rendered tool sockets; it does not prove that hands, clothes or the body never intersect the kit.
5. Use **Default rig / 기본 리그** to return to the built-in semantic skeleton. Both imports and reset replan the current MIDI for the selected body; original hit times stay unchanged.

`.vroid` is an editing source and is not accepted by the importer. VRM 0.x is detected and rejected with conversion guidance; renaming the file cannot change its format. Re-export an editable source as VRM 1.0 in VRoid Studio, or follow the [official UniVRM migration guide](https://vrm.dev/en/univrm1/migrate_vrm0/) where the model's conditions allow it. Arbitrary character GLB, FBX and other rig formats are not implemented by this importer. The separate drum-part GLB importer has a different purpose and a 25 MiB limit.

## Choose a source

| Source | What to use it for |
|---|---|
| [VRoid Studio](https://vroid.com/en/studio) | Create an editable character and export the required VRM version. Best option for making two controlled body proportions. |
| [VRoid Hub](https://hub.vroid.com/en/) | Find models and use the owner's download/consent flow. Check each model's format and conditions. |
| [Official VRM samples](https://github.com/vrm-c/vrm-specification/tree/master/samples) | Obtain documented sample models for development. Each model has its own notice and embedded settings. |
| [BOOTH](https://booth.pm/) | Find creator assets. Check the specific package contents and terms; a free price does not establish redistribution permission or VRM compatibility. |

The official Hub pages for [AvatarSample_A](https://hub.vroid.com/en/characters/2843975675147313744/models/5644550979324015604) and [AvatarSample_B](https://hub.vroid.com/en/characters/7939147878897061040/models/2292219474373673889) currently list **VRM 0.0**. They are not the intended VRM 1.0 exports. Their [sample conditions](https://vroid.pixiv.help/hc/en-us/articles/4402394424089-VRoidPreset-A-Z) are custom conditions, not CC0.

For developers, a separate [AvatarSample_B in pixiv's archived ChatVRM](https://github.com/pixiv/ChatVRM/blob/b542aa00e19dccf9fc48ba340cf7eee011d2329a/public/AvatarSample_B.vrm) is VRM 1.0. The [Twist sample](https://github.com/vrm-c/vrm-specification/tree/master/samples/VRM1_Constraint_Twist_Sample) and [Seed-san](https://github.com/vrm-c/vrm-specification/tree/master/samples/Seed-san) are also 1.0 candidates. The inspected copies contain the required body bones and both hands' finger bones. Their limited browser verification is recorded below; it is not a guarantee for other files with the same model name. Twist has its own VRM Public License settings; it does not inherit three-vrm's MIT license. Seed-san requires credit. File identities and source-specific conditions are in the [engineering record](research/vrm-models.md).

## Keep the editing source and runtime export

Use `.vroid` as your editable VRoid Studio source and `.vrm` as the exported runtime model. Keep both. A downloaded `.vrm` does not replace the original body-parameter editing source. Newer Studio dress-up features can open VRM data, so that distinction should not be confused with a blanket claim that Studio cannot open any VRM. [Official formats](https://vroid.pixiv.help/hc/en-us/articles/23731198070809-About-file-formats-used-by-VRoid-Studio), [dress-up VRM support](https://vroid.pixiv.help/hc/en-us/articles/39513464329881-What-is-VRM)

To prepare two proportions for future testing:

1. Open a new or permitted editable base in Studio and retain its clothing/texture terms.
2. Save two `.vroid` copies. Make one shorter and one taller, with different body proportions, while keeping clothing comparable. Record the body settings and Studio version.
3. Use **Export VRM**, choose **VRM 1.0**, and enter the model/creator information. Keep both `.vroid` sources. Follow the [official export guide](https://vroid.pixiv.help/hc/en-us/articles/15760756822297-I-want-to-learn-more-about-the-VRM-export-feature).
4. Retain hand/finger bones for the grip check. Inspect the exported files after any bone-reduction setting instead of assuming those bones remain.

This pair-creation procedure has been documented, not executed as part of the current review.

## Check permissions for the intended use

| Intended action | What to check |
|---|---|
| Import and evaluate locally | The source's download/use terms, allowed users, and any additional conditions. |
| Publish a rendered performance video | Video/avatar/model use, commercial use if relevant, content restrictions, and required credit. |
| Share the `.vrm`, textures, or a modified model | Separate redistribution/modification conditions. A video-use permission does not establish permission to distribute the source model. |

Read the model's embedded license links and the source page together. The [VRM Public License](https://vrm.dev/licenses/1.0/) incorporates per-model settings. Imported clothing, textures, and special presets may add conditions; see [VRoid's model-use guidance](https://vroid.pixiv.help/hc/en-us/articles/4405813333657-Can-I-use-the-models-created-with-VRoid-Studio-Stable-Ver-for-commercial-purposes). This repository's MIT license applies to its own code, not to your character assets. The video exporter does not automatically insert model credits; include any required credit in the published video or its accompanying attribution as the model's conditions require.

Keep acquired models outside Git. Developers use ignored `.local/vrm-test-models/`; do not put third-party avatar binaries in public assets, test fixtures, or release attachments.

## Current verification and limits

Generated test fixtures exercise the input boundary and metadata handling. The pinned Twist, Seed-san and ChatVRM AvatarSample_B files passed a local browser harness using the actual application importer, VRM adapter, WebGL materials and Rust/WASM evaluator. Each model was evaluated over **128 frames at 30 fps**, then revisited in alternating forward/backward order. Measured raw-bone matrices and poses repeated exactly in that run, and assigned stick/sole socket contacts stayed within **0.000001 m** of the evaluated targets. See the [engineering evidence](research/vrm-models.md#actual-browser-harness-results--2026-10-24) for per-model measurements and scope.

This is a limited pass: **11 of 14 test hits were assigned** for each body, including the kick hits. Notes **41 (low floor tom), 49 (crash) and 51 (ride)** remained unreachable in that kit/body arrangement and were explicitly reported. All original hit events/times were retained. The result does not mean that all 14 hits were physically performed, or that clothing and skin never intersect the kit. In the main app, the Twist sample's import, playback, player viewpoint and model/skeleton visibility were checked. A Twist performance also completed a 923-frame, 30.77-second WebM encoding with synthesized audio. This confirms the reported encoding path, not independent file playback: the browser automation provider could not retrieve the new blob download for a separate player or ffprobe check. See the [application/export verification record](verification/vrm-studio-unreleased.md). The controlled short/tall VRoid pair described above has not been created. These results precede the new [seat/pedal/hardware calibration batch](calibration.md), which needs its own integration evidence.

For your own model, inspect seated placement, hand grip, actual rendered stick tips, feet/pedals and unreachable targets with the intended MIDI. Compare arbitrary forward/backward seeks with fixed-time video export. The current adapter resets spring motion and applies a zero-delta update for each pose; dynamic hair/clothing simulation is not part of the deterministic playback contract. Detailed collision avoidance, per-avatar manual grip/sole calibration, physical telescoping stands and photorealistic hand contact are not established by a successful import.

Consult the branch's verification record and release notes for the tests actually completed. This guide does not announce a release or a Pages deployment.
