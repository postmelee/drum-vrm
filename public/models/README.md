# Bundled drum model

**Five-piece drum kit** by [Innerscene](https://www.innerscene.com/tools/library/3d-parts/five-piece-drum-kit-daff10f4), dedicated under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). Attribution is provided for provenance; CC0 does not require it.

- `innerscene-drum-kit.glb`: unmodified upstream binary, downloaded 2026-10-24 from the original library's public download endpoint.
- `innerscene-LICENSE.txt`: complete CC0 legal text from Creative Commons.
- `innerscene-provenance.json`: original URL, retrieval date, exact SHA-256 checksums, and inspected model structure.
- `NOTICE.txt`: short notice that the distribution license generator can append.

The application adapts coordinates, groups individual instruments, merges static meshes by material, resamples textures to at most 512 pixels, and moves surface contacts to the planned targets at runtime. The binary remains unchanged. The source throne is omitted to keep the skeleton view uncluttered. These adaptations neither change the source MIDI timing nor replace audio: the model is visual only.

Research, alternatives, and current limitations: [drum asset research](../../docs/research/drum-assets.md).
