# Documentation

**English** · [한국어](README.ko.md) · [Drum VRM](../README.md)

Start with the [app workflow and local setup](../README.md#use-the-app). These guides describe the repository build; the [version table](../README.md#which-version-am-i-using) identifies differences from the public app.

[Open the English app](https://postmelee.github.io/drum-vrm/?lang=en). Use **EN / 한국어** in the toolbar to change the interface language during a session.

## Use and customize

| Goal | Guide |
| --- | --- |
| Find a model, check its terms and load VRM | [Character models](models.md) |
| Fit the seat, hands and feet; replace drum parts | [Calibration and drum parts](calibration.md) |
| Save a setup and relink its local files | [Local projects](projects.md) |
| Align your WAV and export video | [WAV and video export](export.md) |
| Understand the instructional basis and source credits | [Performance foundations](performance-foundations.md) |

## Explore the design

The [technical design index](../specs/README.md) explains requirements and implementation boundaries. Useful starting points are [architecture](../specs/architecture/overview.md), [body/kit calibration](../specs/architecture/calibration.md), [VRM compatibility](../specs/architecture/rig-compatibility.md), [performance expression](../specs/architecture/performance-expression.md), and [dependency choices](../specs/architecture/dependencies.md).

[Research and reuse](research/references.md) records animation papers, tools and data. [VRM models](research/vrm-models.md), [drum assets](research/drum-assets.md) and the [follow-up asset survey](research/drum-assets-followup.md) retain source-specific findings and permissions. [Design decisions](decisions/README.md) preserve accepted tradeoffs.

## Verification

A verification record describes one environment and tested revision, not universal support or a new release.

| Scope | Evidence |
| --- | --- |
| Detailed tom holders, cymbal stands and multi-view layout | [Hardware checks](verification/detailed-hardware.md) |
| Automatic fit, measured body clearance and temporal coordination | [Body-clearance checks](verification/body-clearance-measurements.md) |
| Published browser batches | [Pages deployment evidence](verification/pages-browser-performance.md) |
| Public MIDI-to-video workflow | [v0.2 studio checks](verification/studio-0.2.md) |
| VRM application and output path | [VRM checks](verification/vrm-studio-unreleased.md) |
| Seat, contact points and shared hardware | [Calibration checks](verification/calibration-unreleased.md) |
| Palm direction and remaining wrist limits | [Palm checks](verification/palm-orientation-unreleased.md) |

For asset terms, see [bundled model credits](../public/models/README.md) and [distribution notices](../public/licenses.txt). Original application code uses the [MIT license](../LICENSE).
