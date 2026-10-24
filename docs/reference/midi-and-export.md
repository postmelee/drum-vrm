# MIDI and output limits

**English** · [한국어](midi-and-export.ko.md)

## MIDI input

| Property | Supported boundary |
|---|---|
| File | `.mid` / `.midi`, SMF format 0 or 1, PPQN timing |
| File size | At most 10 MiB |
| Selected duration | At most 300 seconds through the selected tracks' valid End of Track |
| Tracks | 1–512 |
| Total events | At most 200,000 across the file |
| Selected supported hits | At most 50,000 |

SMF format 2, SMPTE timing, zero PPQ and malformed framing/tracks are rejected. Tempo changes are read across the whole file, including unselected tracks. MIDI channel 10 is the default drum candidate; inspect track/channel selection and prepare again after changes.

A velocity-zero NoteOn is not a hit. Unmapped notes produce a diagnostic and are excluded from synthesis/rig hits. Supported but unplayable hits retain their original event time, sound and target highlight. These are different cases.

## Video output

| Property | Behavior |
|---|---|
| Picture | 1280 × 720 at 30 fps |
| Sound | Synthesized stereo drum audio at 48 kHz; no WAV/DAW rendering |
| Preparation / tail | 0.5 s before the song; 0.25 s after it |
| Frame count | Ceiling of `(0.5 + song duration + 0.25) × 30` |
| Codec candidates | MP4 H.264/AAC, WebM VP9/Opus, then WebM VP8/Opus |

The app probes the actual configuration and a short encode/decode trial. A failed audio combination is not silently changed to successful silent output. Codec support depends on the browser and OS; secure context (HTTPS/localhost) is required.

The desktop Chromium verification does not establish Safari, Firefox, mobile or maximum-length performance. PCM and compressed output remain in memory, so longer files need more memory. The five-minute input limit is not a per-device performance guarantee.

[Export procedure](../how-to/export-video.md) · [Engine contract](../../specs/architecture/engine-contract.md) · [Audio/export contract](../../specs/architecture/audio-export-contract.md)

[Reference](README.md)
