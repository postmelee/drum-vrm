# Character model sources

**English** · [한국어](models.ko.md)

These are acquisition and condition records from the **2026-10-24** review. They do not promise compatibility for another file with the same display name. The [historical model audit](../../../mydocs/tech/research/2026-10-24/vrm-models.md) preserves inspected commits, SHA-256 values, metadata, proportions and test scope. No candidate avatar is bundled.

| Candidate | Source and observed version | Conditions to retain |
|---|---|---|
| VRM1_Constraint_Twist_Sample | [Official sample at pinned commit](https://github.com/vrm-c/vrm-specification/tree/94e82dd346fa6cf0337c4421728640e5252dd38e/samples/VRM1_Constraint_Twist_Sample); VRM 1.0 | [Sample notice](https://github.com/vrm-c/vrm-specification/blob/94e82dd346fa6cf0337c4421728640e5252dd38e/samples/VRM1_Constraint_Twist_Sample/README.md): pixiv Inc., VRM Public License 1.0 and embedded settings; not three-vrm's MIT. |
| Seed-san | [Official sample at pinned commit](https://github.com/vrm-c/vrm-specification/tree/94e82dd346fa6cf0337c4421728640e5252dd38e/samples/Seed-san); VRM 1.0 | [Sample notice](https://github.com/vrm-c/vrm-specification/blob/94e82dd346fa6cf0337c4421728640e5252dd38e/samples/Seed-san/README.md): VirtualCast, Inc., VRM Public License 1.0; inspected metadata requires credit. |
| AvatarSample_B, ChatVRM artifact | [pixiv archived ChatVRM file](https://github.com/pixiv/ChatVRM/blob/b542aa00e19dccf9fc48ba340cf7eee011d2329a/public/AvatarSample_B.vrm); VRM 1.0, model version 1.1 | VRoid Project; VRM Public License and [additional sample conditions](https://vroid.pixiv.help/hc/en-us/articles/4402394424089-VRoidPreset-A-Z). |
| AvatarSample_A / B on Hub | [A](https://hub.vroid.com/en/characters/2843975675147313744/models/5644550979324015604), [B](https://hub.vroid.com/en/characters/7939147878897061040/models/2292219474373673889); pages listed VRM 0.0 at review | Hub binaries were not downloaded. Follow download/consent flow and sample terms; do not identify them with the separate ChatVRM 1.0 file. |

The VRoid A/B sample conditions are custom terms, not CC0. They permit commercial/noncommercial uses including videos without required credit, but restrict CC0 relabeling, paid redistribution of the supplied sample data and some character-creation-service uses. Read the full conditions and conduct restrictions. [Official sample classification](https://vroid.pixiv.help/hc/en-us/articles/4402614652569-Do-VRoid-Studio-s-sample-models-come-with-conditions-of-use) distinguishes older CC0 samples from A/B/C.

The [VRM Public License](https://vrm.dev/licenses/1.0/) incorporates per-model settings; a license-family label alone is insufficient. Inspect source terms and metadata for your copy. Local evaluation, video publication and model-file redistribution are distinct uses.

## What has and has not been checked

The pinned Twist, Seed-san and ChatVRM B copies contain required body and finger bones and were exercised in limited importer/adapter/contact/seek checks. See [availability and exact evidence](../availability.md) for implementation and test boundaries. Successful inspection or contact checks do not establish all poses, skin clearance or independent exported-file playback.

Creating a controlled short/tall pair in VRoid Studio remains a planned experiment. The [rig compatibility design](../../../specs/architecture/rig-compatibility.md) describes what to compare. Use [obtain a model](../../how-to/obtain-model.md) for editing/runtime formats and permissions, then [import it](../../how-to/import-character.md).

[Sources](README.md)
