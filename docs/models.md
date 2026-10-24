# Preparing character models

**English** · [한국어](models.ko.md) · [Documentation](README.md)

Load a local VRM 1.0 character, inspect its terms and fit it to the drum kit. This guide describes the repository build; see [version availability](../README.md#which-version-am-i-using) before using the public app. No external avatar is bundled.

## Supported files

| Property | Application limit |
| --- | --- |
| Character format | Self-contained VRM 1.0 `.vrm`, at most 50 MiB |
| Humanoid | 15 required semantic bones; optional finger bones are reported |
| Scene | At most 10,000 nodes; valid bone references and hierarchy; finite transforms |
| Images | Embedded PNG/JPEG; at most 64 images, 4096 pixels on either side and 64 million pixels in total |
| Resources | Embedded buffers and textures; no external file or URL references |

Bone references must be valid and non-duplicated; the hierarchy cannot contain cycles or multiple parents. These are acceptance limits, not a promise that every device can comfortably render the largest accepted model. See the [rig input contract](../specs/architecture/rig-contract.md) for technical details.

VRM 0.x needs conversion or re-export as VRM 1.0 where the model's terms permit it; renaming the file does not convert it. Use an editable source in VRoid Studio or the [UniVRM migration guide](https://vrm.dev/en/univrm1/migrate_vrm0/). `.vroid` is an editing source, not the runtime import format. Generic character GLB, FBX, OBJ, BLEND and GLTF mapping is not supported. Static drum GLB replacement is a [separate workflow](calibration.md#replace-drum-bodies-while-keeping-hardware) with a 25 MiB limit.

## Import and inspect

1. Open **캐릭터** (Character), choose **VRM 불러오기…** (Import VRM), and select one `.vrm`. The app processes it in the browser without uploading it.
2. Read the model name, author, version, required-bone and finger-bone counts, and **모델 이용 조건** (Model usage conditions). Missing optional rights fields are identified with the VRM 1.0 specification default; absence is not treated as permission. Metadata links are not opened or fetched automatically.
3. Load a demo or MIDI and inspect the separate model/skeleton visibility controls. The adapter measures limb lengths and uses the same semantic-bone solver as the built-in skeleton. Available finger bones receive a grip pose; missing optional fingers do not prevent import.
4. [Adjust the seat and targets](calibration.md), then compare playback and arbitrary forward/backward seeks. Check the visible grip, attached stick tips, feet and pedals. Contact measurements use tool/foot sockets, not all skin, glove or shoe vertices.
5. Choose **기본 리그** (Default rig) to restore the skeleton. Import and reset replan the current MIDI for the selected body without changing original hit times.

Spring motion is reset for each evaluated pose so seeking and output are repeatable. Dynamic hair/clothing simulation, collision-free seating, precise skin/sole calibration and natural wrists at every target are not established by a successful import. Model/calibration choices last for the current session.

## Find a model

| Source | What to use it for |
| --- | --- |
| [VRoid Studio](https://vroid.com/en/studio) | Create a character and export VRM 1.0; useful for controlled body-proportion comparisons. |
| [VRoid Hub](https://hub.vroid.com/en/) | Find downloadable models and follow the creator's download/consent flow. Check each file's version and conditions. |
| [Official VRM samples](https://github.com/vrm-c/vrm-specification/tree/master/samples) | Test documented standard features. Each model has its own notice and embedded settings. |
| [BOOTH](https://booth.pm/) | Find creator assets. Check package contents and terms; a free price does not establish redistribution permission or compatibility. |

The inspected Hub entries for [AvatarSample_A](https://hub.vroid.com/en/characters/2843975675147313744/models/5644550979324015604) and [AvatarSample_B](https://hub.vroid.com/en/characters/7939147878897061040/models/2292219474373673889) were VRM 0.0, with [custom sample conditions](https://vroid.pixiv.help/hc/en-us/articles/4402394424089-VRoidPreset-A-Z), not CC0. Verify the actual downloaded version.

For developers, the pinned [ChatVRM AvatarSample_B](https://github.com/pixiv/ChatVRM/blob/b542aa00e19dccf9fc48ba340cf7eee011d2329a/public/AvatarSample_B.vrm), [Twist sample](https://github.com/vrm-c/vrm-specification/tree/master/samples/VRM1_Constraint_Twist_Sample) and [Seed-san](https://github.com/vrm-c/vrm-specification/tree/master/samples/Seed-san) are VRM 1.0 test candidates. The inspected files contain the required bones and both hands' finger bones. Twist has separate VRM Public License settings; it does not inherit three-vrm's MIT license. Seed-san requires credit. Exact file identities, terms and test outcomes are preserved in the [model research record](research/vrm-models.md).

## Keep an editing source and runtime export

Retain `.vroid` for editing body parameters and `.vrm` for runtime use. A downloaded `.vrm` does not replace its original editable source. Studio's dress-up features can open VRM data; see the [official file formats](https://vroid.pixiv.help/hc/en-us/articles/23731198070809-About-file-formats-used-by-VRoid-Studio) and [dress-up VRM support](https://vroid.pixiv.help/hc/en-us/articles/39513464329881-What-is-VRM).

To prepare a controlled pair for body-fit testing:

1. Start from a permitted editable base and keep its clothing/texture terms.
2. Save short and tall `.vroid` variants with different limb proportions and comparable clothing. Prefer visible wrists and legs. Record the body settings and Studio version.
3. Export both as **VRM 1.0** with model/creator information, following the [official export guide](https://vroid.pixiv.help/hc/en-us/articles/15760756822297-I-want-to-learn-more-about-the-VRM-export-feature).
4. Inspect the exported hand/finger bones after any bone-reduction settings. Compare the same MIDI, contacts, seat fit, arbitrary seeks and output frames for both models.

This controlled pair is a documented test procedure, not a completed compatibility result.

## Check permissions for the intended use

| Intended action | Check |
| --- | --- |
| Import and evaluate locally | Download/use terms, allowed users and additional conditions. |
| Publish a rendered video | Avatar/video use, commercial use if relevant, content restrictions and required credit. |
| Share a `.vrm`, textures or modified model | Separate redistribution/modification terms. Video-use permission does not establish model-redistribution permission. |

Read embedded terms together with the source page. The [VRM Public License](https://vrm.dev/licenses/1.0/) incorporates per-model settings; clothing, textures and presets may have additional terms. See [VRoid model-use guidance](https://vroid.pixiv.help/hc/en-us/articles/4405813333657-Can-I-use-the-models-created-with-VRoid-Studio-Stable-Ver-for-commercial-purposes). This project's MIT license does not apply to your avatar. The exporter does not insert model credits automatically; include required credit in the published video or accompanying attribution as the terms require.

Keep acquired models outside the repository's tracked/public files. `.local/vrm-test-models/` is an ignored local storage option, not a source of redistributable fixtures.

## What has been verified?

Pinned Twist, Seed-san and ChatVRM AvatarSample_B files have been exercised with the actual importer, VRM adapter, WebGL renderer and Rust/WASM evaluator. The records include assigned and unreachable hits, stick/foot-socket contact measurements, and repeatability under forward/backward seeking. They do not certify every strike, visible skin/clothing clearance or other files with the same model name.

- [Original model inspection and browser measurements](research/vrm-models.md)
- [Application import, playback and output checks](verification/vrm-studio-unreleased.md)
- [Seat/contact/shared-hardware checks](verification/calibration-unreleased.md)
- [Palm orientation checks and wrist limitations](verification/palm-orientation-unreleased.md)

The recorded VRM output runs completed encoding/muxing, but the browser provider could not retrieve those blob downloads for independent player/ffprobe verification. The controlled short/tall VRoid pair remains untested. Check your own model with the intended MIDI; successful format inspection alone is not a performance or video-compatibility guarantee.

[Body and kit fitting](calibration.md) · [VRM compatibility design](../specs/architecture/rig-compatibility.md)
