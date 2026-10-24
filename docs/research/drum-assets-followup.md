> Research record preserved from [baseline 66951cd](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/research/drum-assets-followup.md). Dated findings and statements of implementation status below belong to that record; use the [current specifications](../../specs/README.md) and [model guide](../models.md) for current behavior.

# Additional drum preset candidates

Research date: **2026-10-24**. Status: **assessment only; candidates for implementation PRs**.

This follow-up evaluates distinct model assets beyond the existing Innerscene kit and the alternatives in [the initial research](drum-assets.md). It adds no model to the public asset directory, changes no application code, and performs no release or deployment. The files downloaded for inspection remain in the ignored `.local/drum-assets/research-2/` directory. Their topology and metadata were inspected; their rendered quality, contact calibration, and performance in the application have **not** been verified.

An implementation PR can proceed within the user's requested development scope without another implementation approval. **A release or deployment requires explicit user approval.** This research is not release approval.

## Recommended order

| Priority | Candidate | Readiness | Reason and remaining work |
|---|---|---|---|
| 1 | Batoski, **Full Drum Kit** | Candidate for the next implementation PR | Verified downloadable CC BY 3.0 GLB with 17 named mesh parts, one material, and a small embedded texture. Requires scale/contact calibration, pedal inspection, and an explicit decision about its second crash cymbal. |
| 2 | smoj, **Drum Kit** | License and file verified; grouping work needed | Small, texture-free low-poly alternative. Its 111 meshes have numeric names, so instrument grouping and contact anchors must be authored before independent part replacement. |
| 3 | Blendkit, **Electronic drum kit (low poly)** | Promising category; original file not inspected | The original listing declares Free and CC0. Its official download flow requires an account and Blender add-on. Verify the acquired file, export format, part boundaries, and materials before selecting it for implementation. |

These are asset-readiness judgments, not claims that a model is popular, an industry standard, or already visually validated. Changing only color, material, or depth on shared geometry is a **finish or shape variant**, not another independently modeled drum kit. The useful diversity target is a distinct acoustic kit, a stylized low-poly kit, and an electronic kit with a different physical construction. Hand percussion should be evaluated as a separate instrument feature.

## 1. Batoski — Full Drum Kit

- Original model page: [Full Drum Kit](https://poly.pizza/m/1HvDgRF7fX).
- Creator: **Batoski**. Publication date in the page metadata: **2025-11-08**.
- Original downloadable file: [GLB](https://static.poly.pizza/75833e82-6e6b-4ca1-9caa-82b3d9c6b73e.glb).
- License: **CC BY 3.0**, confirmed in the model page's embedded `model.Licence` field and its [Creative Commons license link](https://creativecommons.org/licenses/by/3.0/).
- Inspected file: **788,112 bytes**, SHA-256 `1ced36a850608f146407ad9387208209c59dd4784377a41c53727632bf00f895`.

The actual GLB contains **9,969 triangles, 17 nodes/meshes, one material, and one embedded 512 × 512 PNG**. It was generated with `obj2gltf`; no external buffer or image URI was found. These are measurements of the downloaded file, not the listing's triangle counter.

The named parts include `Bigsnare`, `BigKick`, `BigTOM`, `BigFL`, `BigFL_1`, separate upper/lower hi-hat meshes `HHT`/`HHB`, `Crash`, `Crash_1`, `Ride`, their stands, and the stool. This is a practical starting point for independent instrument selection and replacement. The author's configuration has one rack tom, two floor toms, two crashes, and one ride, giving it a different arrangement from the current kit.

The measured overall bounds are approximately **5.096 × 3.199 × 4.318 source units**. Normalize and verify the physical scale rather than assuming these dimensions are appropriate metres for the performer. Pedal geometry and footboard contact were not visually verified. The current canonical instrument set has one `crash` role; two visually distinct crashes need an explicit mapping/profile decision. Duplicating a visible cymbal must not imply that MIDI hits are independently routed to it.

## 2. smoj — Drum Kit

- Original model page: [Drum Kit](https://poly.pizza/m/7FCPjy-3xmz).
- Creator: **smoj**. Publication date in the page metadata: **2018-01-29**.
- Original downloadable file: [GLB](https://static.poly.pizza/56bea38d-cb46-43e2-976b-2ed5d0827ffe.glb).
- License: **CC BY 3.0**, confirmed in the page metadata and [license link](https://creativecommons.org/licenses/by/3.0/).
- Inspected file: **416,052 bytes**, SHA-256 `4893bf0cd89542862298456be893bce24742947d31c8293230b817e91629d5de`.

The actual GLB contains **3,776 triangles, 111 nodes/meshes, 11 flat materials, and no images**. It uses `obj2gltf` and has no external buffer/image URI. The measured bounds are approximately **1.600 × 0.872 × 1.250 source units**.

The low polygon count is attractive for a stylized preset, but names such as `group887072338` and `mesh887072338` carry no instrument semantics. A developer must identify each drum, cymbal, stand, and pedal; create stable groups; and record their contact anchors. Merge static subparts by material **within each instrument** to reduce draw calls without losing independent replacement. Low triangle count alone does not establish good rendering performance or correct part boundaries.

## 3. Blendkit — Electronic drum kit (low poly)

- Original listing: [Electronic drum kit (low poly)](https://www.blendkit.com/asset-gallery-detail/ec73293c-30da-46e7-bda6-fc12a597ab6b/).
- Asset record: `ec73293c-30da-46e7-bda6-fc12a597ab6b`.
- Creator profile: [Flareworks Studio, author ID 272522](https://www.blendkit.com/?query=author_id:272522). The cached asset page labels the same linked profile **Daniel Lee**. Preserve the stable author ID and recheck the displayed name when acquiring the original file.
- License evidence: the original listing labels the asset **Free** and **Creative commons zero**; [Blendkit's license documentation](https://www.blendkit.com/docs/licenses/) distinguishes CC0 from its separate Royalty Free license.
- Official acquisition instructions: [Get this model](https://www.blendkit.com/get-blendkit/ec73293c-30da-46e7-bda6-fc12a597ab6b/), which directs users to a free account and the Blender add-on.

The page reports **31,303 polygons** and a **60.3 MiB** file. These are publisher-reported values, not inspected triangle counts or a predicted GLB download size. No original file was acquired in this follow-up, so there is **no verified file hash, mesh inventory, texture inventory, or GLB export** to record. Do not describe this candidate as ready to ship or assume it is a single self-contained browser asset.

An electronic kit would add more useful structural diversity than another acoustic color variant. Before choosing this asset, inspect the lawful original download, confirm its model-specific CC0 declaration, convert and inspect the exported GLB, identify pad/cymbal/rack/pedal boundaries, and calibrate actual contact surfaces. The potentially large source file makes texture and export-size reduction worth measuring; it is not evidence that the final browser asset must remain large.

## Other evaluated candidates

| Candidate | Evidence | Assessment |
|---|---|---|
| [J-Toastie — Drumkit](https://poly.pizza/m/4l5jRtpTec) | CC BY 3.0 in the original page metadata. Actual [GLB](https://static.poly.pizza/144c6d27-9e58-4102-8790-83d0d21140e9.glb): **368,512 bytes, 5,414 triangles, 5 nodes/meshes, 6 materials, no images or external URIs**. SHA-256 `00395c7e32f2bd1563e38c8c0481e877768771ca2d869e0a5bb65127cb45508a`. | Legitimate low-poly comparison candidate, but only five mesh groups are exposed: combined base, floor tom, snare, and two cymbals. Separation is needed for independent kick/tom replacement. The page reports 5,222 triangles; use the inspected file's 5,414 for this pinned download. |
| [MNDV.ecb — drum kit with optional drummer](https://opengameart.org/content/drum-kit-with-optional-drummer) | Original listing declares **OGA-BY 3.0**, credits `mandervellt records`, and provides a [`.blend` download](https://opengameart.org/sites/default/files/drummer.blend). Actual file: **7,206,320 bytes**; SHA-256 `87cb330830f34edd2d4742469d393a4d7a58882bc21fbf22f5276fcae09c3d40`. | A distinct cartoon/rigged option, but not an immediate preset. Binary block inspection found a Blender 2.78 file with 6 Object, 3 Mesh, 2 Armature, 2 Action, and 11 Image blocks. No Blender render or export was performed. The author requests changing the company logo and says most textures are packed and all are CC0; verify textures, remove/change the logo, and separate/export the kit before adoption. [OGA-BY explanation and license-text link](https://opengameart.org/content/oga-by-30-faq). |
| [1DesignTool — Drum Kit](https://1designtool.com/3d/models/drum-kit) | Its [custom license](https://1designtool.com/3d/license) permits product embedding but restricts redistribution of model files on their own or in asset kits/libraries, including modified models. | Exclude from the publicly distributed preset asset library under the currently verified terms. It is not CC0. A different decision needs written clarification or permission covering the actual repository distribution. |
| [Innerscene — Djembe](https://www.innerscene.com/tools/library/3d-parts/djembe-f083adc3) | Original creator's listing declares CC0. No file was downloaded in this follow-up. | A single hand-percussion instrument, not a complete drum-set preset. It needs different note mapping, hand/palm contacts, playing motion, and audio before being presented as playable percussion. |

## Implementation boundaries

Keep source-mesh naming and material handling in an asset adapter. `PerformancePlan` describes musical roles, event times, targets, and limb assignments; it must not depend on a vendor's mesh names. A kit profile must explicitly connect those roles to visible parts. Replacing a visual model does not add audio samples, WAV playback, character compatibility, or new percussion techniques.

For each adopted kit, record its units, forward/up axes, pivot conventions, head/bow/footboard contact anchors, and floor constraints. Separate shell, cymbal, stand, and pedal roles where the editor or animation requires independent control. A mesh boundary is not automatically a meaningful musical part. Apply any batching inside those boundaries, then verify picking, replacement, floor placement, and direct/reverse/seek contact consistency. Do not move MIDI hit times to hide a geometry mismatch.

Before acceptance, render the actual imported materials, inspect missing textures and visible marks, compare contact points with the performer, and measure loading time, draw calls, decoded texture memory, and export behavior. Source-file bytes and triangle counts are useful screening values, not complete performance measurements.

## Notices and provenance required on adoption

For a CC BY candidate, preserve the model title, creator, original page, exact license/version and link, downloaded-file hash, and any supplied notices. Describe changes such as separation, scale normalization, texture conversion, optimization, and coordinate adaptation. Add those records and the license text to the distribution's asset notices; do not label the model itself as MIT. The application code and each third-party asset retain their respective license records. [CC BY 3.0 terms](https://creativecommons.org/licenses/by/3.0/)

For the electronic CC0 candidate, retain the model-specific license evidence and original-file hash even though attribution is not required by CC0. Do not substitute the site's general license documentation for the acquired asset's actual declaration. Recheck that the original download and any included textures match the recorded permission.

No immutable upstream release tag or commit was identified for the inspected Poly Pizza files. Their exact download URLs, inspection date, byte sizes, and SHA-256 values above pin the examined versions. Local HTML snapshots, binaries, and `assessment.json` remain research evidence rather than public distribution assets. No additional model in this document has been shipped or visually validated in the application.
