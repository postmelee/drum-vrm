# Save and reopen a local project

**English** · [한국어](projects.ko.md) · [Documentation](README.md)

The repository build can save a prepared performance and its setup as a local
`.drumvrm.json` file. This is separate from the [measurement report](calibration.md#inspect-a-problem-passage).

1. Prepare MIDI, choose its mapping/interpretation, fit the character and kit, and
   align the [WAV](export.md) if used.
2. In **업로드 · 연주정보 → 프로젝트 저장 · 열기** (Upload / Performance → Project
   save/open), press **프로젝트 저장** (Save project). Save again after changing settings.
3. To return, choose **프로젝트 열기…** (Open project). The current session stays
   available while required files are missing or a file fails validation.
4. Use **원본 파일 연결…** (Relink source files) to select the same MIDI and any
   WAV, VRM or custom drum GLB. Names may change, but file contents must match the
   saved SHA-256 and size. Bundled drum presets do not need relinking.
5. Once all references match, the app restores the prepared plan, selected tracks,
   interpretation and explicit note edits, seat, kit, appearance, camera, hit response, WAV offset, visibility
   and playhead. Play and seek to check the result.

The project **contains MIDI note data and the performance plan**. Keep it private
when the source performance is private. It does not include raw MIDI/WAV/VRM/GLB
files, decoded audio or model geometry. Keep the original files alongside your
project; saving it does not grant permission to redistribute those assets. Nothing
is uploaded automatically, and projects are not autosaved in browser storage.

Exact reopening requires the same WASM engine binary and compatible measured rig.
If the app reports a version or rig mismatch, use the matching app build or prepare
the original MIDI as a new performance. A newer engine does not silently reinterpret
the stored project. The current schema does not migrate projects across engine
revisions. The [project contract](../specs/architecture/project-format.md) describes
the format and validation boundaries.
