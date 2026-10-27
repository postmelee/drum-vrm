# Notification design review evidence

Date: 2026-10-27 (Asia/Seoul). Scope: [#62](https://github.com/postmelee/drum-vrm/issues/62).
Design branch starts from merged PR #61, `6038e8e138cbe6ddd30fcf1ce99de0c7750df9fa`.
The [proposed contract](../../specs/architecture/notifications.md) and
[standalone HTML prototype](../design/notifications.html) do not change production
UI, interpretation, audio or motion behavior. No private files are used by the prototype.

## Inspection

Current `src/main.ts` has a one-message `notify()` with a six-second timeout,
plus separate bottom-strip messages and many inline hints in the studio template.
The existing toast is centered above the piano roll. Mapping comparison prose
is permanently shown in the source inspector after an upload.
`src/timeline.ts` already visually hides its long accessible description with
`.midi-roll-sr`; this is not visible menu clutter and must be retained.

## Executed checks

- `npm run docs:check`: bilingual pairs, structure and links passed.
- `git diff --check`: passed. Extracted prototype JavaScript passed `node --check`.
- Actual local in-app browser at a 1280 × 720 viewport: checked light desktop,
  dark theme with a simulated 390 px preview, and reduced-motion toggle.
- Progress changed to success in the same card; success auto-dismissed with
  fixed-preview mode off; the result remained in session history.
- Conflicting-choice action opened settings, selected General MIDI and focused
  the mapping control. Three-card preview and close/history controls were checked.
- Browser error log was empty after these interactions. A full-page screenshot
  remains local at `/private/tmp/drum-vrm-toast-design.png`, not committed.

These checks cover the design preview, not a complete notification implementation.
Queue priority/overflow, all stale-action paths, hover/focus/visibility timing,
keyboard/screen-reader coverage, real 390 px device layout and 200% zoom still
require the implementation checks listed in the contract. Rust/WASM and full
production tests were not rerun because no production source changed.

## Visual refinement

Owner follow-up on 2026-10-27: reduce lower padding, align with the app's design
tokens, distinguish warning/failure icons, and use selection blue for actions.
The prototype now uses the system font and neutral/blue token values from
`src/style.css`, 8 px lower padding, and outlined triangle-exclamation/circle-X
icons. Dark action text derives from focus blue and the existing text token.

At the actual 745 × 833 browser viewport, light and dark three-card previews were
visually checked. DOM-computed light values were background `rgb(245,245,245)`,
action `rgb(50,116,200)`, lower padding `8px`, and radius `8px`. Documentation,
diff and prototype JavaScript syntax checks passed. The updated local screenshot
is `/private/tmp/drum-vrm-toast-design-refined.png`. No production source changed.

## Bilingual prototype

Owner follow-up on 2026-10-27 adds an English / 한국어 switch and `lang=en|ko`
URL selection to the prototype. Visible copy, history and accessible labels update
in place, preserving card DOM, timers, theme and layout. Existing announcements
are not replayed; each retains its original language attribute.

Browser checks covered both switch directions, English three-card text, a dark
390 px simulated preview, history translation, and switching during progress then
receiving completion in the newly selected language. Reloading the English URL
retained English. Syntax, documentation and diff checks passed. The English review
screenshot is local at `/private/tmp/drum-vrm-toast-design-en.png`.

## Compact content-driven height

Owner follow-up on 2026-10-27 removes fixed card height assumptions and reduces
bottom padding to 4 px desktop / 2 px narrow. The card and its text column now use
flex layout; descriptions have no paragraph margins and text rows use a 2 px gap.
Missing descriptions/actions reserve no space, while 32/44 px controls stay intact.

Browser-computed success-card heights were 89.59 px desktop and 97.59 px in the
390 px narrow preview. Narrow action and close controls both remained 44 px high.
The no-action result card measured 51.59 px, confirming content-dependent height.
English and Korean narrow previews were inspected; the local screenshot is
`/private/tmp/drum-vrm-toast-compact.png`. Syntax, diff and docs checks passed.

## Compact action highlight

The subsequent mobile refinement separates the action label's visible highlight
from its hit target. Browser inspection in the narrow preview measured a 24 px
highlight inside the unchanged 44 px button. Keyboard focus visibly outlines the
label; the outer button remains transparent. Hover feedback is gated by pointer
capability, while pressed feedback also works on touch. Syntax, diff and docs
checks passed. Screenshot: `/private/tmp/drum-vrm-toast-highlight.png` (local only).

## Production integration

The owner subsequently approved applying this study to the actual app. The
implementation increment follows design commit
`6e43c6fda029df6c1e7baeadaa693f86a94d4951` on `codex/toast-design`, in PR #63.
The earlier sections above remain evidence of their respective prototype stages.
No Rust, WASM interpretation, uploaded assets, dependency versions or media encoding
algorithms changed. No public deployment is established by these local checks.

### Implemented boundary

- `NotificationStore`: delayed progress, ID coalescing, source invalidation,
  stable priority ordering, 3/1 visible cards, 8 pending, 20 terminal history,
  separately retained current/unresolved conditions, remaining-time pause/resume.
- `NotificationFeedback`: complete operation boundaries and nested setup steps,
  one final result, required-choice precedence and cancellation suppression.
- Keyed DOM cards/history, existing i18n bindings, separate polite/error announcers,
  scoped actions, focus routing, reduced-motion CSS and app design tokens.
- MIDI/demo/reprepare, character/layout/equipment, WAV, projects, note edits,
  review and export feed the shared center. Numeric progress and cancellation
  controls remain at the task. Background initialization emits no success toast.
- Mapping coverage is an Analysis table. Reference guidance is disclosed. Current
  mapping, provisional/unapplied state, missing original files, WAV no-overlap,
  analysis badges, required attribution and piano-roll descriptions remain.
  Old result live regions were replaced by the shared announcers; the explicitly
  requested pose measurement retains its own status semantics.

### Executed verification

- Full frontend suite: 39 files, 267 tests, including 10 controlled-clock lifecycle
  cases for fast results, progress dismissal, overlapping pause reasons, queue
  pressure, capacity changes, persistent errors, scope invalidation, nested work,
  cancellation/replacement and required-choice precedence.
- `npm run build:web` (including typecheck), `npm run docs:check` and
  `git diff --check`. Vite retains its existing large-bundle advisory.
- Actual local app in the in-app browser, at 745 × 833: imported the independently
  authored seven-note Songsterr study and prepared the built-in posture demo.
  Each import produced one final result; analysis/motion warnings stayed distinct
  from mapping success. No initial automatic-fit success was announced.
- Invalid authored MIDI: persistent failure, badge after dismissal, detail recovery
  through history, and obsolete history actions removed on replacement.
- Authored GM/Logic conflict study: provisional GM label and required-choice toast;
  dismissing or changing the selector did not remove the provisional label.
  Applying Logic succeeded and removed it. Saving and reopening that project
  preserved Logic and its rimshot interpretation without automatic remapping.
- Authored one-second WAV: load, offset outside the song (persistent warning),
  and correction to zero (warning resolved). A 3.80-second WebM preparation
  completed with the persistent download link and one export result. This checks
  notification routing; it is not a new encoded-media quality audit.
- Demo preparation cancellation left one cancellation history entry and no later
  success. Escape on a toast preserved the inspector and moved focus to the bell.
  A history action opened Analysis and focused the specific error detail.
- English/Korean switching changed card text without replaying the existing live
  regions; the previous announcement retained its original language. Light/dark
  layouts and the focused blue action were inspected. A focused completion card
  remained present across subsequent checks, beyond its normal timeout.
- Real production app embedded in a **390 CSS px iframe viewport**, not the synthetic
  toast study: Korean/dark and English/light error cards measured 95.59 px high,
  with 44 px action targets and 24 px visible action labels. Viewport and document
  scroll width were both 390 px. Desktop completion without a description measured
  69.59 px, with a 32 px action and 24 px label. Heights were content-driven.
- Browser error logs were empty for the actual app checks. The local screenshot
  `/private/tmp/drum-vrm-notifications-app.png` shows the integrated Korean result
  and concise source inspector using authored fixture data. Temporary viewport
  tooling is ignored under `.local/`; fixtures/screenshots are not committed.

### Remaining manual coverage

A physical touch device, browser-level 200% zoom, OS reduced-motion preference and
spoken screen-reader output were not available in this browser harness. The CSS
reduced-motion/coarse-pointer branches and live-region/focus DOM were reviewed;
these are not claims of those device/assistive checks. Existing rig/project/audio
regression tests passed, but every model-import failure and hardware combination
was not repeated interactively. The center changes UI feedback only; it does not
remove the engine's existing motion diagnostics or improve contact accuracy.
