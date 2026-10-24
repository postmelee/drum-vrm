# Import a VRM character

**English** · [한국어](import-character.ko.md)

This workflow requires the VRM importer listed in [feature availability](../reference/availability.md). Prepare a permitted, self-contained VRM 1.0 file; see [obtain a model](obtain-model.md) and [format limits](../reference/model-formats.md).

1. Open **캐릭터** (Character) and choose **VRM 불러오기…** (Import VRM). Select the file.
2. Wait for inspection and loading. If the importer reports a version, bone or image error, correct the source and export again. Renaming `.vroid`, `.glb` or VRM 0.x to `.vrm` does not convert it.
3. Read the model name, author, required/finger-bone counts and **모델 이용 조건** (Model usage conditions). Displayed conditions come from the file. Omitted optional fields are labeled with the VRM specification default; that display is not independent permission to use the asset. Links are not fetched automatically.
4. Load the demo or your MIDI. Use **모델 표시** (Model) and **뼈대 표시** (Skeleton) separately to inspect the seated body, hands and feet.
5. Follow [seat and contact fitting](fit-character.md). Pause at contact frames and seek backward/forward to inspect the actual stick tips and pedal contacts.
6. Choose **기본 리그** (Default rig) to restore the built-in skeleton if needed. Changing the body replans the current MIDI while preserving its original hit times.

Files are processed in the browser rather than uploaded by the app. No third-party avatar is included with the application.

A successful import establishes the input boundary, not natural motion for every pose. Missing optional finger bones reduce grip detail. Hair/clothing spring motion is reset for deterministic seeking and video output. Skin, clothes and shoe geometry may still intersect the kit.

[Task guides](README.md)
