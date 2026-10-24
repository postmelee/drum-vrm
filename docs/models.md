# Preparing character models

[한국어](models.ko.md) · [README](../README.md) · [Engineering evidence](research/vrm-models.md)

Checked on **2026-10-24**. This is a preparation guide for the VRM work in #17/#18/#30. The public v0.2.0 app still uses its built-in skeleton and does **not** import character VRM files. The sample files below have had their metadata inspected locally; their animation and export compatibility with Drum VRM has not yet been tested. No third-party avatar is bundled with this project.

## Choose a source

| Source | What to use it for |
|---|---|
| [VRoid Studio](https://vroid.com/en/studio) | Create an editable character and export the required VRM version. Best option for making two controlled body proportions. |
| [VRoid Hub](https://hub.vroid.com/en/) | Find models and use the owner's download/consent flow. Check each model's format and conditions. |
| [Official VRM samples](https://github.com/vrm-c/vrm-specification/tree/master/samples) | Obtain documented sample models for development. Each model has its own notice and embedded settings. |
| [BOOTH](https://booth.pm/) | Find creator assets. Check the specific package contents and terms; a free price does not establish redistribution permission or VRM compatibility. |

The official Hub pages for [AvatarSample_A](https://hub.vroid.com/en/characters/2843975675147313744/models/5644550979324015604) and [AvatarSample_B](https://hub.vroid.com/en/characters/7939147878897061040/models/2292219474373673889) currently list **VRM 0.0**. They are not the intended VRM 1.0 exports. Their [sample conditions](https://vroid.pixiv.help/hc/en-us/articles/4402394424089-VRoidPreset-A-Z) are custom conditions, not CC0.

For developers, a separate [AvatarSample_B in pixiv's archived ChatVRM](https://github.com/pixiv/ChatVRM/blob/b542aa00e19dccf9fc48ba340cf7eee011d2329a/public/AvatarSample_B.vrm) is VRM 1.0. The [Twist sample](https://github.com/vrm-c/vrm-specification/tree/master/samples/VRM1_Constraint_Twist_Sample) and [Seed-san](https://github.com/vrm-c/vrm-specification/tree/master/samples/Seed-san) are also 1.0 candidates. The inspected copies contain the required body bones and both hands' finger bones. These findings establish format/capabilities only. Twist has its own VRM Public License settings; it does not inherit three-vrm's MIT license. Seed-san requires credit. File identities and source-specific conditions are in the [engineering record](research/vrm-models.md).

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

Read the model's embedded license links and the source page together. The [VRM Public License](https://vrm.dev/licenses/1.0/) incorporates per-model settings. Imported clothing, textures, and special presets may add conditions; see [VRoid's model-use guidance](https://vroid.pixiv.help/hc/en-us/articles/4405813333657-Can-I-use-the-models-created-with-VRoid-Studio-Stable-Ver-for-commercial-purposes). This repository's MIT license applies to its own code, not to your character assets.

Keep acquired models outside Git. Developers use ignored `.local/vrm-test-models/`; do not put third-party avatar binaries in public assets, test fixtures, or release attachments.

## What the next implementation must verify

The planned model information panel must distinguish detected VRM version, required bones, finger capability, model terms, and actual compatibility status. Missing information should remain visible as unknown.

Once character import is available in a verified build, compare the same MIDI on both proportions: seated placement, hand grip, actual rendered stick-tip contact, feet/pedals, and unreachable targets. Seek forward/backward and compare the same timestamps with fixed-time video export. A model that parses successfully has not yet passed these checks. Consult the release notes for the build's actual support before expecting the public app to load a character.
