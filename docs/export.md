# Align your WAV for playback and video

**English** · [한국어](export.ko.md)

This guide describes the repository build. Check the [version table](../README.md#which-version-am-i-using)
before assuming that a local change is on the public site.

1. Load MIDI and prepare the performance in **업로드 · 연주정보** (Upload / Performance).
2. In the same panel, choose **WAV 불러오기…** (Load WAV) under **연주 오디오** (Performance audio). The WAV
   replaces synthesized drum sound for both preview and video; it is not mixed with it.
3. Set **WAV 시작 위치** (WAV start position) relative to MIDI song time, then press
   **오프셋 적용** (Apply offset). Start at zero if both files were exported from the
   same timeline origin. Positive values delay the WAV; negative values advance it.
4. Review the displayed trimming/padding information, play a recognizable hit and
   seek near the end to check alignment. The app does not automatically match beats
   or remove silence from your WAV. Keep count-in and leading silence when comparing origins.
5. If a video is needed, open **내보내기** (Export), choose an available format and press **영상 추출**
   (Export video). Keep **오디오 없이 내보내기** (Export without audio) off to include
   the selected WAV. Download the completed file and check its sound and picture.

**합성음으로 복원** (Restore synthesized sound) removes the WAV choice. A failed WAV
load preserves the previous audio. Audio changes invalidate a previously prepared
video, so export again after changing the file or offset. WAV and offset settings
can be preserved through [local project save/open](projects.md); files are processed locally in the browser.

The 0.5-second preparation window is added automatically. Do not add it again to
the offset. With an offset of +0.25 seconds, WAV sample zero starts at output
0.75 seconds. A negative offset can use the preparation window; only audio before
output zero is cut. Audio beyond the video end is cut, and missing portions are silent.
The video duration remains determined by the selected MIDI tracks.

Supported input is one RIFF/WAVE file, at most 100 MiB and five minutes, mono/stereo,
8–192 kHz, uncompressed PCM 8/16/24/32-bit or Float 32-bit. The browser prepares
48 kHz stereo audio. Compressed WAV and other formats need conversion first.
For decoder, time and memory details, see the [audio/export contract](../specs/architecture/audio-export-contract.md).

Long files require memory for the decoded source, aligned audio and encoded video.
Available MP4/WebM combinations depend on browser codecs. Silence is an explicit
export choice, not an automatic fallback when an audio encoder fails.
