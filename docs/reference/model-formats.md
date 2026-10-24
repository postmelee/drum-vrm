# Supported model files

**English** · [한국어](model-formats.ko.md)

See [feature availability](availability.md) before using these input paths. A file extension alone does not establish the underlying format.

| Purpose | Accepted input | Application limits |
|---|---|---|
| Character | Self-contained VRM 1.0 `.vrm` | 50 MiB; 15 required humanoid bones; valid, finite hierarchy/transforms; at most 10,000 nodes |
| Character images | Embedded PNG/JPEG | At most 64 images, each side at most 4096 pixels, 64 million pixels across images |
| Drum appearance | Self-contained static GLB 2.0 `.glb` | 25 MiB; no skinned models; visible geometry required |

Resources must be embedded; external file/URL references are rejected. Character bone references must be valid and non-duplicated, and the hierarchy must not contain cycles or multiple parents. Optional finger bones are reported but their absence does not reject a model.

These are application acceptance limits, not a guarantee that the largest accepted file renders comfortably on every device. Character metadata and license fields are displayed for review; they do not replace source conditions.

## Not directly accepted

- VRM 0.x: convert or re-export as VRM 1.0 where permitted.
- `.vroid`: VRoid Studio editing source, not the runtime model input.
- Generic character GLB, FBX, OBJ, BLEND or GLTF: no general humanoid mapping/import path.
- Drum assets in GLTF, FBX, OBJ or BLEND: export a self-contained static GLB first.
- Combined local drum bodies and supports: no automatic semantic segmentation; prepare a body-only file for shared-hardware replacement.

Use [character import](../how-to/import-character.md), [model acquisition](../how-to/obtain-model.md), or [part replacement](../how-to/replace-drum-parts.md). The authoritative implementation contract is [rig and model input](../../specs/architecture/rig-contract.md).

[Reference](README.md)
