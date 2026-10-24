# What Drum VRM represents

**English** · [한국어](product-scope.ko.md)

Drum VRM turns note events into a reviewable 3D performance. MIDI supplies note numbers, timing and velocities; it does not fully describe the performer's hands, grip, posture, instrument sound or intent.

The application therefore makes explicit motion choices. It prepares the selected events, assigns limbs where feasible, evaluates a pose at a requested song time, and renders video with synthesized audio. Diagnostics preserve the distinction between an unsupported note and a supported hit that the current body/kit cannot perform.

A visual preset does not determine the sound. A character model does not change the source notes. Playback and export use a common time-based evaluation so seeking is meaningful rather than dependent on the frames previously played.

## What a successful result means

A usable performance should preserve musical timing, make the chosen motion readable and expose failures. A close stick-tip contact alone does not prove a comfortable wrist, collision-free body or natural gesture. A successful model import does not prove those outcomes either.

The project begins with an independently implemented rule-based solver. Learned motion models, WAV synchronization, arrangement editing and project persistence are separate work. See [availability](../reference/availability.md) for the implementation boundary and [requirements](../../specs/product/requirements.md) for the adopted product criteria.

[Explanations](README.md)
