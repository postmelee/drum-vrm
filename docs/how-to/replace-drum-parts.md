# Replace drum parts

**English** · [한국어](replace-drum-parts.ko.md)

Check [feature availability](../reference/availability.md): basic appearance replacement exists in the public app; the development workflow separates playable bodies from shared stands and pedals.

1. Open **드럼** (Drums). The camera moves to the front and the character is temporarily hidden for editing.
2. Select a kit preset, or click a part and change that part's appearance. See [preset sources](../reference/presets.md).
3. To import your own part, prepare a self-contained, static, **body-only GLB**. Remove stands, pedals and unrelated objects in a 3D editor first. Use [the format reference](../reference/model-formats.md) for input limits.
4. Select the target part and press **이 부품의 GLB 불러오기…** (Import GLB for this part).
5. Inspect the automatically fitted result. Adjust appearance scale, rotation or finish if needed. Fitting is approximate; compare the visible playing surface with the contact target.
6. If placement must change, use [contact calibration](fit-character.md). Use **부품을 프리셋으로 복원** (Restore part from preset) to discard the local replacement.

The development workflow retains application-owned stands and pedals while changing shells, heads and cymbals. The importer cannot reliably separate arbitrary combined geometry; a file containing its own supports can display duplicate hardware. Exact clamps, telescoping stands and photorealistic connections are not guaranteed.

Appearance scale/rotation/finish and optional hit response do not change the musical target or synthesized sound. Moving a contact target deliberately changes placement and related hardware. Replacing a model does not load a sound sample. Settings are session-only.

Enable **타격 반응** (Hit response) and adjust **반응 강도** (Amount) to preview instrument movement. It is suppressed while editing drums or when the operating system requests reduced motion. Close the drum panel to see the response.

Imported assets retain their source conditions. [Model-use guidance](obtain-model.md#check-conditions-for-each-use) also explains the distinction between rendered video and asset redistribution.

[Task guides](README.md)
