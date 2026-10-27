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
