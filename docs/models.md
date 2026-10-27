# Preparing character models

**English** · [한국어](models.ko.md) · [Documentation](README.md) · [Open the app](https://postmelee.github.io/drum-vrm/?lang=en)

Choose a bundled VRM 1.0 sample or load your own character, inspect its terms and fit it to the drum kit. This guide describes the repository build; see [version availability](../README.md#which-version-am-i-using) before using the public app.

## Bundled character presets

Open **Character** and choose a sample from the dropdown. Its file downloads only
when selected; the initial skeleton needs no model download. The app uses the same
import checks, measured-body solver and automatic fitting as a local VRM. Use
**Cancel download** to keep the current model while stopping a preset download.

| Preset | Creator and terms | Purpose |
| --- | --- | --- |
| [AvatarSample_B](../public/characters/avatar-sample-b-provenance.json) | VRoid Project; VRM Public License and additional VRoid sample conditions | General character test; the pinned ChatVRM file is VRM 1.0. |
| [Seed-san](../public/characters/seed-san-provenance.json) | VirtualCast, Inc.; VRM Public License; **credit required** | A different body and bone-constraint configuration. |
| [Twist sample](../public/characters/twist-provenance.json) | pixiv Inc.; VRM Public License | Constraint test; **automatic fitting remains limited**. |

The linked provenance records identify the exact unchanged files, original sources
and conditions. These models are not MIT or CC0. Selection does not certify natural
contact or collision-free motion. Check **Analysis** for fitting and motion warnings.
For Seed-san videos, include the required creator credit; choosing a preset does not
automatically insert credit into exported video. The app does not export avatar files.

## Supported files

The table below describes VRM. [Rigged FBX](#import-a-rigged-fbx) has a separate setup and resource boundary.

| Property | Application limit |
| --- | --- |
| Character format | Self-contained VRM 0.0 or 1.0 `.vrm`, at most 50 MiB |
| Humanoid | 15 required semantic bones; optional finger bones are reported |
| Scene | At most 10,000 nodes; valid bone references and hierarchy; finite transforms |
| Images | Embedded PNG/JPEG; at most 64 images, 4096 pixels on either side and 64 million pixels in total |
| Resources | Embedded buffers and textures; no external file or URL references |

Bone references must be valid and non-duplicated; the hierarchy cannot contain cycles or multiple parents. These are acceptance limits, not a promise that every device can comfortably render the largest accepted model. See the [rig input contract](../specs/architecture/rig-contract.md) for technical details.

VRM 0.0 imports directly, with facing and legacy thumb names normalized before body fitting. Unknown versions and files declaring both VRM extensions are rejected. `.vroid` is an editing source, not the runtime import format. Generic character GLB, OBJ, BLEND and GLTF mapping is not supported; binary FBX uses the explicit workflow below. Static drum GLB replacement is a [separate workflow](calibration.md#replace-drum-bodies-while-keeping-hardware) with a 25 MiB limit.

## Import and inspect

1. Choose a **Character** preset, or use **Character → Import VRM / FBX…** to select your own `.vrm`. Local files are processed in the browser without uploading them.
2. Open **Model details / terms** for the name, author, version, required-bone and finger-bone counts and terms. VRM 1.0 optional rights fields show their specification defaults when omitted. VRM 0.0 shows its original usage and redistribution/modification fields; missing values stay unspecified and are never replaced with VRM 1.0 defaults. Metadata links are not opened or fetched automatically.
3. Load a demo or MIDI and choose **Model**, **Skeleton**, **Model + skeleton** or **Hidden** in the display selector. The adapter measures limb lengths and uses the same semantic-bone solver as the built-in skeleton. Available finger bones receive a grip pose; missing optional fingers do not prevent import.
4. The app automatically attempts a body-fit layout. Inspect its result and, if needed, [adjust the seat and targets](calibration.md), then compare playback and arbitrary forward/backward seeks. Check the visible grip, attached stick tips, feet and pedals. Contact measurements use tool/foot sockets, not all skin, glove or shoe vertices.
5. Choose **Default skeleton** in the preset selector to restore the skeleton. Import and reset replan the current MIDI for the selected body without changing original hit times.

Spring motion is reset for each evaluated pose so seeking and output are repeatable. Dynamic hair/clothing simulation, collision-free seating, precise skin/sole calibration and natural wrists at every target are not established by a successful import. Use a [local project](projects.md) to retain model/calibration choices across sessions; the same model file must be relinked.

## A legacy model looks washed out

MToon uses separate lit and shade colors/textures. A VRM 0.0 material with a
`_MainTex`, white `_ShadeColor` and no `_ShadeTexture` can turn white in shaded
regions even when its main texture loads correctly. The importer preserves that
material setting. Inspect the original material before changing global lighting.
If the intended shade uses the base artwork, assign the same texture to the lit
and shade slots in the source editor and export a separate copy, as described in
the [official MToon guide](https://vrm.dev/en/univrm/shaders/shader_mtoon/).
This appearance correction does not resolve body-fitting or contact warnings.

## Import a rigged FBX

1. Use **Character → Import folder…** to select one extracted character package. The app finds VRM/FBX files and PNG/JPEG images in subfolders; choose the model if several are present. Other files are ignored. ZIP archives are not imported. Alternatively, select one binary FBX 7.4/7.5 with **Import VRM / FBX…**, then choose companion images before **Inspect file**. Existing file/image limits still apply to the selected package.
2. Only unresolved required bones appear first. Use the dropdowns; already assigned bones and candidates incompatible with mapped ancestry are excluded. Matching names appear first; **Show candidates with other names** reveals other eligible bones. A current assignment remains visible so it can be cleared. **Review assigned bones** and **Optional bones / fingers** allow corrections. There is no arbitrary selection between ambiguous aliases.
3. Selecting a bone marks its position in pink in the 3D preview. Drag to rotate or scroll to zoom. This shows the source rest pose, not a deformation or performance guarantee. If no suitable candidate remains, check the assigned/optional ancestors and descendants and clear incorrect assignments.
4. Review **Colors and textures**. Unique filename matches suggest base-color images; ties and common shadow/normal/roughness maps are not guessed. Original texture bindings take precedence. Choose another image if necessary, or restore the original material by clearing the selection. **Material appearance** defaults to **Original material**, including explicit FBX specular intensity. **Matte lighting** removes highlights and environment reflections; **Texture only (Unlit)** displays base colors without lighting or tone mapping. Choose Unlit to match a flat, texture-only reference. Lit preview modes use different lighting from the performance scene. No mode recreates authoring-tool toon shaders.
5. **Scale and file details** contains the metre conversion (`0.01` for centimetres, `1` for metres). Use a standing rest pose. **Use character** validates the full mapping and runs automatic fitting without stretching bones. Cancel/failure preserves the active character.
6. Successful imports remember settings for the same FBX bytes in this browser, for up to 10 files. This stores mappings, units, image names and material appearance, not model/image contents; images must be selected again. Storage availability is optional. Save a project and keep original FBX/images for portable, hash-verified reopening.

Images are local: up to 64 selected PNG/JPEG files totaling 50 MiB, each at most 4096 pixels per side, with a combined 64M pixel budget including used embedded images. External resources are never fetched. The [FBX import contract](../specs/architecture/rig-contract.md#fbx-import) owns further parser and scene limits.

FBX clips, Blender/MMD constraints, spring physics and toon node shaders are not playback inputs. Unmapped auxiliary bones retain their local rest transforms and follow their ancestors; complex twist/control rigs may therefore deform differently. First-person masking follows mapped head descendants. A successful import does not certify clothing clearance or natural motion. FBX has no VRM rights panel: consult the original source and model terms. No downloaded FBX is bundled by this feature.

## Find a model

| Source | What to use it for |
| --- | --- |
| [VRoid Studio](https://vroid.com/en/studio) | Create a character and export VRM 1.0; useful for controlled body-proportion comparisons. |
| [VRoid Hub](https://hub.vroid.com/en/) | Find downloadable models and follow the creator's download/consent flow. Check each file's version and conditions. |
| [Official VRM samples](https://github.com/vrm-c/vrm-specification/tree/master/samples) | Test documented standard features. Each model has its own notice and embedded settings. |
| [BOOTH](https://booth.pm/) | Find creator assets. Check package contents and terms; a free price does not establish redistribution permission or compatibility. |

The inspected Hub entries for [AvatarSample_A](https://hub.vroid.com/en/characters/2843975675147313744/models/5644550979324015604) and [AvatarSample_B](https://hub.vroid.com/en/characters/7939147878897061040/models/2292219474373673889) were VRM 0.0, with [custom sample conditions](https://vroid.pixiv.help/hc/en-us/articles/4402394424089-VRoidPreset-A-Z), not CC0. Verify the actual downloaded version.

For developers, the pinned [ChatVRM AvatarSample_B](https://github.com/pixiv/ChatVRM/blob/b542aa00e19dccf9fc48ba340cf7eee011d2329a/public/AvatarSample_B.vrm), [Twist sample](https://github.com/vrm-c/vrm-specification/tree/master/samples/VRM1_Constraint_Twist_Sample) and [Seed-san](https://github.com/vrm-c/vrm-specification/tree/master/samples/Seed-san) are the sources of the bundled VRM 1.0 presets. The inspected files contain the required bones and both hands' finger bones. Twist has separate VRM Public License settings; it does not inherit three-vrm's MIT license. Seed-san requires credit. Exact file identities, terms and test outcomes are preserved in the [model research record](research/vrm-models.md).

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

Keep other acquired models outside the repository's tracked/public files unless their exact distribution rights have been reviewed and recorded. `.local/vrm-test-models/` is an ignored local storage option; placing a file there does not authorize redistribution. The bundled presets are separately reviewed exceptions.

## What has been verified?

Pinned Twist, Seed-san and ChatVRM AvatarSample_B files have been exercised with the actual importer, VRM adapter, WebGL renderer and Rust/WASM evaluator. The records include assigned and unreachable hits, stick/foot-socket contact measurements, and repeatability under forward/backward seeking. They do not certify every strike, visible skin/clothing clearance or other files with the same model name.

- [Original model inspection and browser measurements](research/vrm-models.md)
- [Application import, playback and output checks](verification/vrm-studio-unreleased.md)
- [Seat/contact/shared-hardware checks](verification/calibration-unreleased.md)
- [Palm orientation checks and wrist limitations](verification/palm-orientation-unreleased.md)

The recorded VRM output runs completed encoding/muxing, but the browser provider could not retrieve those blob downloads for independent player/ffprobe verification. The controlled short/tall VRoid pair remains untested. Check your own model with the intended MIDI; successful format inspection alone is not a performance or video-compatibility guarantee.

[Body and kit fitting](calibration.md) · [VRM compatibility design](../specs/architecture/rig-compatibility.md)
