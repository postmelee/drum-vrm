# Make your first video

**English** · [한국어](first-video.ko.md)

Use the included 30-second drum demo to learn the upload → preparation → playback → export flow. This tutorial uses the basic workflow available in the public app; see [feature availability](../reference/availability.md).

## Before you start

Open [Drum VRM](https://postmelee.github.io/drum-vrm/) in desktop Chromium. Video export needs a supported codec and HTTPS or localhost. You do not need a MIDI file or external character model for this tutorial.

## Prepare and play

1. Open **업로드 · 연주정보** (Upload / Performance), then choose **30초 데모** (30-second demo).
2. Wait for preparation. The scene shows a rig and drum kit, and the piano roll shows notes. Read the analysis messages; some hits may be unsupported or impossible for the chosen body and kit.
3. Press **재생** (Play). Watch the playhead and the sticks, then pause.
4. Seek to a different point on the timeline. Resume playback, or use the reset control to return to the beginning.
5. Try the front and top camera views. Open **드럼** (Drums) to inspect the kit from the front, then close the panel.

The demo uses synthesized drum sound. It is not a recording of a DAW instrument or the sound of an imported drum model.

## Export and check

1. Open **내보내기** (Export) and choose **영상 추출** (Export video).
2. Leave synthesized audio enabled. Wait for codec probing and frame rendering; you can cancel the operation.
3. When export completes, save the download and open it in your own video player.
4. Check that the picture and drum sound play together, including the beginning and end.

An exactly 30-second song produces 923 frames at 30 fps: about 30.7667 seconds including preparation, tail and frame rounding. The extension may be MP4 or WebM, depending on codec checks.

If export cannot start, follow [Export video](../how-to/export-video.md). Next, use your own MIDI and review [MIDI support and output limits](../reference/midi-and-export.md).

[Tutorials](README.md)
