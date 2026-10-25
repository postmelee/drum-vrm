# Audio, transport and export contract

**English** · [한국어](audio-export-contract.ko.md)

## Shared time axis

Source MIDI time remains unchanged. Output time adds a 0.5-second preparation window
and a 0.25-second tail. At output frame `i`, evaluate song time `i / 30 - 0.5`.
The output frame count is `ceil((0.5 + sourceDuration + 0.25) * 30)`; a 30-second
source therefore produces 923 frames. Apply the same shift to synthesized audio
and to the song-time placement of imported WAV audio.

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

An open-hi-hat voice is truncated at the next closed-hat/pedal hit or explicit close
control while retaining its original onset. This is approximate choke behavior,
not velocity-layer or partial-open sample playback. The
[MIDI interpretation contract](midi-interpretation.md) owns control meaning.

## Local WAV replacement

[wav.ts](../../src/wav.ts) validates a local RIFF/WAVE file before browser decoding.
It accepts uncompressed PCM 8/16/24/32-bit or IEEE Float 32-bit, including their
WAVE_FORMAT_EXTENSIBLE subtypes. Input is mono/stereo, 8–192 kHz, at most 100 MiB
and 300 seconds. Compressed WAV, RF64, multiple data chunks, nonfinite float samples,
invalid size/rate/frame metadata and other channel layouts are rejected.

`loadLocalWav(file)` decodes once through a 48 kHz `AudioContext`, validates the
decoded length/channels and finite samples, then closes that decoder context.
The unshifted source buffer remains local and unchanged. Mono is copied equally to
stereo. There is no loudness normalization, limiter, stem extraction or mixing with
the synthesized drums.

`alignWavToPlan(source, plan, offsetSeconds)` returns the aligned stereo buffer and
trim/padding information. Offset is finite and bounded to −300…300 seconds. Its sign
refers to song time: positive places WAV sample zero later; it never moves MIDI.
The integer output-sample shift is:

`round((PRE_ROLL + offsetSeconds) * SAMPLE_RATE)`

At +0.25 seconds, source sample zero occurs at output 0.75 seconds. At −0.25 seconds
it occurs at output 0.25 seconds, inside the preparation window. Only samples before
output zero are trimmed from the beginning. Samples after the fixed video end are
trimmed; uncovered output samples are zero. The buffer length always matches the
frame-rounded video duration. Alignment reports copied duration, start/end trimming,
leading/trailing padding and the effective sample-rounded offset, including zero
overlap. The UI displays trimming and lack of overlap rather than hiding them.

Preview and export use the same aligned buffer. Loading WAV replaces drum synthesis;
removing it restores synthesis. Reapplying an offset starts from the unchanged source,
so edits do not accumulate resampling or trimming. A changed plan realigns that source
to the new duration. Settings remain session-only; a silent export is still an
explicit choice. See the [export guide](../../docs/export.md) for controls.

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
110 MiB before copies and encoded data. WAV retains its decoded source and an aligned
output buffer, in addition to decoder temporaries and encoded data. Alignment writes
directly into the output buffer rather than retaining a third full PCM copy.
Input limits do not certify peak memory or
successful maximum-duration output on all devices.

WebCodecs requires a secure context and a browser/OS-supported codec combination.
A local encoding result, an independently decoded downloaded file and a public-site
check are separate evidence levels. Record exactly which checks ran, the tested commit, environment and remaining limits
in the linked issue or verification report.

Historical codec latency measurements remain in the
[v0.1 evidence](../../docs/verification/mvp-0.1.md); they must not be generalized
to every AAC encoder or used to shift source MIDI timing.
