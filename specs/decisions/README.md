# Architecture decisions

Decision identifiers and original bodies are preserved. The migration notice identifies
the audited source; the record's own status and date describe its historical scope.
Some records include implementation notes from that time. They are not current status tables.

| ID | Decision | Scope |
| --- | --- | --- |
| [0001](0001-engine-and-rig-boundary.md) | Performance intent and rig separation | Adopted boundary; fixed-body implementation extended by 0004 |
| [0002](0002-mvp-scope-and-export.md) | MVP scope and fixed-frame output | Historical MVP scope and export rationale |
| [0003](0003-minimal-studio-and-models.md) | Minimal studio and replaceable appearance | UI and appearance boundary |
| [0004](0004-vrm-humanoid-and-grip.md) | Semantic humanoid and numeric profiles | Extends the fixed-body assumption in 0001 |
| [0005](0005-calibration-pedals-hardware.md) | Calibration, planted pedals and hardware | Extends 0004; ankle-only approximation |
| [0006](0006-technique-and-expression-foundation.md) | Technique and constrained expression | Research/design foundation; proposed runtime profiles |

Current executable contracts are in [architecture](../architecture/README.md).
A change of decision is recorded in a new ADR with an explicit supersedes/extends
relationship. Do not rewrite earlier decisions to make them appear contemporaneous.
