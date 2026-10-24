# Export video

**English** · [한국어](export-video.ko.md)

Prepare MIDI first. Export uses the same time-based performance and synthesized audio as the preview. It does not record your screen or the DAW's sound. [Output reference](../reference/midi-and-export.md) lists dimensions, timing and codec behavior.

1. Finish your camera, character and kit adjustments, then open **내보내기** (Export).
2. Keep audio enabled, or explicitly choose silent output if that is what you want.
3. Press **영상 추출** (Export video). The app checks codec support and performs a short encode/decode trial before rendering the full video.
4. Wait for completion, or cancel if necessary. A canceled or failed operation is not a completed download. Final encoder cleanup may take time.
5. Save the completed download and check it in a separate player. Verify the first hit, later hits, picture, sound and end of the video.

The development **연주자** (Player) view locks orbit, pan and zoom. Opening the drum editor switches to its front view; closing it restores the previous camera. Check the view again before exporting.

## If export fails

| Symptom | Action |
|---|---|
| Export APIs unavailable | Use desktop Chromium on HTTPS or localhost; browser/OS codec availability still varies. |
| MP4 unavailable but WebM works | Save the offered WebM. The app rejects codec combinations that fail its duration/onset checks. |
| Audio combination rejected | Read the reason. Silent output is an explicit choice, not an automatic successful fallback. |
| Long preparation or memory pressure | Try a shorter MIDI selection/file. The accepted five-minute limit is not a performance guarantee. |
| Pose looks wrong | Return to preview and inspect contacts at the same song time. Export does not repair reach, grip or collisions. |

Export includes preparation and tail intervals. Imported model conditions can require video credit; add it when publishing because the exporter does not insert attribution automatically.

[Task guides](README.md)
