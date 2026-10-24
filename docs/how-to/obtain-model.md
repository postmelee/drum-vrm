# Obtain a character model

**English** · [한국어](obtain-model.ko.md)

Choose a source that permits the use you intend. [Model source records](../reference/sources/models.md) distinguish specific samples and their inspected versions.

| Source | Useful for |
|---|---|
| [VRoid Studio](https://vroid.com/en/studio) | Make an editable character and export VRM 1.0. |
| [VRoid Hub](https://hub.vroid.com/en/) | Download models when the creator permits it; follow the site's consent flow. |
| [Official VRM samples](https://github.com/vrm-c/vrm-specification/tree/master/samples) | Inspect documented standard features under each sample's own conditions. |
| [BOOTH](https://booth.pm/) | Find creator packages; check the included file type, version and terms. |

## Prepare the runtime file

1. Check the package includes VRM 1.0, or use a permitted editable source that can be exported as VRM 1.0.
2. Keep `.vroid` as the editable Studio source and `.vrm` as the runtime export. A downloaded VRM does not replace the source body parameters. Studio has some VRM dress-up support; this does not make the two formats interchangeable.
3. In Studio, choose VRM export and select **VRM 1.0**. Follow the [official export guide](https://vroid.pixiv.help/hc/en-us/articles/15760756822297-I-want-to-learn-more-about-the-VRM-export-feature). Retain finger bones where possible and inspect the result after bone reduction.
4. For an older model, use a permitted source export or the [UniVRM migration guide](https://vrm.dev/en/univrm1/migrate_vrm0/). Changing the extension is insufficient.
5. [Import the resulting file](import-character.md) and test it with the intended MIDI.

See the [Studio format guide](https://vroid.pixiv.help/hc/en-us/articles/23731198070809-About-file-formats-used-by-VRoid-Studio) and [VRM dress-up description](https://vroid.pixiv.help/hc/en-us/articles/39513464329881-What-is-VRM) for the editing/export distinction.

## Check conditions for each use

| Intended use | Check |
|---|---|
| Local import and evaluation | Download/use terms, allowed users and additional conditions. |
| Publishing a performance video | Avatar/video use, commercial use where relevant, content restrictions and required credit. |
| Sharing the model or textures | Separate modification and redistribution conditions. Video permission does not imply model-file redistribution permission. |

Read embedded metadata and the source page together. The [VRM Public License](https://vrm.dev/licenses/1.0/) uses per-model settings. Clothing, textures and special presets can add terms; see [VRoid's usage guidance](https://vroid.pixiv.help/hc/en-us/articles/4405813333657-Can-I-use-the-models-created-with-VRoid-Studio-Stable-Ver-for-commercial-purposes).

Free price does not imply CC0 or redistribution permission. The exporter does not automatically insert model credits; include required attribution when publishing. Keep third-party character downloads outside the repository and public assets.

[Task guides](README.md)
