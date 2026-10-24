# Audio, transport and export contract

**English** · [한국어](audio-export-contract.ko.md)

## Shared time axis

Source MIDI time remains unchanged. Output time adds a 0.5-second preparation window
and a 0.25-second tail. At output frame `i`, evaluate song time `i / 30 - 0.5`.
The output frame count is `ceil((0.5 + sourceDuration + 0.25) * 30)`; a 30-second
source therefore produces 923 frames. Apply the same shift to synthesized audio.

[src/types.ts](../../src/types.ts) owns browser timing constants, mirrored in the
[Rust model](../../crates/drum-engine/src/model.rs). A future timing change must update
preview, synthesis, export and boundary tests together; expressive movement must not
silently extend or clip these windows.

## Sound and playback

[synthesis.ts](../../src/synthesis.ts) generates deterministic 48 kHz stereo PCM from
supported hits; this is not reproduction of a user's DAW samples or WAV mix.
The synthesis worker prepares audio off the UI thread. Unassigned supported hits
remain audible. Web Audio provides the running playback clock, with paused/seeking
states evaluated from the same absolute timeline.

## Encoding

[export.ts](../../src/export.ts) renders 1280×720 at 30 fps with backpressure rather
than recording frames at the wall-clock rate. Candidate profiles are MP4 H.264/AAC,
WebM VP9/Opus, then WebM VP8/Opus, subject to actual support.

Before offering an audio profile, run its real configuration and a short A/V encoding,
then demux/decode the sample. Validate decoded onset and audio duration against the
expected time axis within one frame. Check final audio duration as well. API presence
alone is not codec compatibility, and an unsupported codec must not silently produce
a silent file. Explicit silent output follows the user's separate choice.

Cancellation applies during probing and encoding. Close frames, encoders and buffers
on completion/failure/cancellation. Browser flush may require cleanup time; do not
claim immediate forced termination or present an aborted result as a completed file.

## Resource and compatibility limits

The renderer does not accumulate an array of all raw video frames. PCM and the encoded
file remain in memory; a single five-minute stereo Float32 PCM allocation is roughly
110 MiB before copies and encoded data. Input limits do not certify peak memory or
successful maximum-duration output on all devices.

WebCodecs requires a secure context and a browser/OS-supported codec combination.
A local encoding result, an independently decoded downloaded file and a public-site
check are separate evidence levels. Record exactly which checks ran, the tested commit, environment and remaining limits
in the linked issue or verification report.

Historical codec latency measurements remain in the
[v0.1 evidence](../../docs/verification/mvp-0.1.md); they must not be generalized
to every AAC encoder or used to shift source MIDI timing.
