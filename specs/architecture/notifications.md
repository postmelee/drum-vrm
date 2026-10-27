# Notifications and concise inspectors

**English** · [한국어](notifications.ko.md)

This contract describes the shared notifications in the checked-out app.
[Issue #62](https://github.com/postmelee/drum-vrm/issues/62) owns scope and acceptance;
[PR #63](https://github.com/postmelee/drum-vrm/pull/63) contains the implementation.
The [interactive prototype](../../docs/design/notifications.html) remains a separate
synthetic design study. Source implementation does not establish public deployment.

English / 한국어 switches translate current cards, history and accessible labels
in place. `?lang=en` / `?lang=ko` selects the initial language. Switching preserves
remaining time and does not replay announcements; announcers retain the language
of the event that was actually announced.

## Information belongs in one place

| Information | Destination | Example |
| --- | --- | --- |
| Operation result | One shared toast | MIDI loaded / Songsterr mapping applied |
| Long-running work | Update the same toast; keep Cancel at the operation | Preparing performance… |
| Current selection/state | Control value or compact status | Mapping: Songsterr; Audio: Synthesized |
| Unresolved condition | Persistent badge/field state plus one toast | Mapping needs review; Analysis · 2 |
| Evidence and measurements | Analysis, disclosed on demand | Candidate coverage, raw notes, unassigned hits |
| Reference instructions | Collapsed contextual help or documentation | Offset sign, mapping/version limitations |
| Input validation | One concise inline correction at the field | Enter a value from 5 to 100% |
| Attribution/terms | Discoverable model/license section | Creator and usage terms |
| Screen-reader description | Existing accessible association | Piano-roll instrument counts and shortcuts |

Ordinary inspectors contain headings, labels, current values and actions. Keep at
most one short helper sentence per group, only when needed to make a decision.
Do not paste analysis output, caveats, implementation explanations or entire
operation histories into `.hint` paragraphs. Do not simply move those paragraphs
into long toasts. Colors never substitute for labels.

## Placement and appearance

- Desktop: anchor to the upper-right of the **stage content**, 16 px inside its
  top/right edges, below the camera strip. Width 344 px, capped by available width.
  Opening an inspector moves the anchor with the stage, keeping menu controls free.
  The closed-inspector position is the app's upper-right content area.
- At 700 px and below: one card below the toolbar/camera strip, 12 px side margins,
  including safe-area insets. Do not place cards over transport or the piano roll.
  A narrow inspector may be temporarily overlaid; dismissal must remain reachable.
- Cards: content-driven flex layout with auto height and no fixed/minimum card
  height. Desktop padding is 10 px top, 12 px sides, 4 px bottom; narrow/touch
  layouts use 8 px top, 10 px sides, 2 px bottom. Text rows have a 2 px gap and
  no paragraph margins; absent descriptions/actions leave no empty rows.
  Keep 8 px radius, 1 px border, restrained shadow, 10 px internal
  gap; use existing light/dark tokens. Opaque surface, readable 14 px title and
  12–13 px optional description. No blurred or translucent text background.
  Narrow layouts use an 8 px icon/content gap. Content flexes with `min-width: 0`;
  title, detail and action text wrap naturally, never using clipping or line clamps
  to achieve a smaller height. Button hit targets still impose their own minimum
  height; do not shrink the 32/44 px interaction areas to flatten a card.
- Reuse the system font and `--bg`, `--surface`, `--control`, `--border`, `--text`,
  `--muted`, `--active`, `--focus` from `src/style.css`; do not introduce a separate
  green-tinted palette. Actions use the selection blue (`--focus: #3274c8`), with
  `--active` for hover/focus backgrounds. In dark mode derive readable action text
  by mixing 65% focus blue with 35% text, retaining the same hue family.
- Use consistent 18 px outlined SVG icons: check in a circle for success,
  exclamation in a triangle for review required, X in a circle for failure, and
  information in a circle. Warning stays amber; failure uses the app's error red
  (`#cf4335`), lightened against the dark surface. Icons distinguish shape as well
  as color. Clickable text stays blue regardless of severity.
- Structure: 18 px severity icon, short title, optional short supporting sentence,
  at most one labeled action, always a close control. Color only the icon/accent.
  Action/close hit targets are at least 32 px on desktop and 44 px on touch layouts.
  Separate the action hit area from its visual highlight: a single-line label has
  an 18 px line height and 3 px vertical padding (24 px highlight). Hover, pressed
  and keyboard-focus styling wraps that label, not the entire 44 px button. Long
  labels can wrap and grow. Hover styling applies only to hover-capable pointers;
  touch uses pressed feedback. Keep a visible keyboard outline around the label.
- Target one title line and up to two supporting lines at desktop width. Translate
  naturally and wrap at narrow widths/200% zoom; never clip essential recovery copy.
  Dense details belong behind the action. Avoid seconds/countdown badges.

## Copy and concrete outcomes

| Event | Title / optional detail | Action |
| --- | --- | --- |
| Successful MIDI preparation | MIDI loaded / Songsterr mapping · 1,513 hits | MIDI settings |
| Ready with unassigned motion | Performance ready · 2 motions need review / Songsterr mapping applied | View analysis |
| Ambiguous mapping | Review drum mapping / General MIDI selected provisionally | Choose mapping |
| Unsupported notes remain | Some notes could not be mapped | View analysis |
| No usable mapping | Could not find a drum mapping | MIDI settings |
| Preparation failure | Could not prepare the performance / Check the details and try again | View details |
| WAV ready | WAV loaded | Files / Audio |
| Project saved | Project file ready | Files / Audio |
| Video ready | Video ready to download | View export |
| Fit ready | Layout fitted to character | View details; Undo remains in Character |

Combine the result and its most important warning into one completion toast.
Do not show separate “mapping selected,” “audio synthesized,” “ready,” and “two
warnings” cards for one import. Stage status stays terse: Idle / Preparing / Ready
/ Needs attention. Selecting a mapping is not proof of full preparation success.
Playback, pause, seek, camera changes and ordinary slider edits produce no toast.
First-load background setup should not produce an unsolicited success notification.

## Lifecycle and concurrency

| Kind | Automatic dismissal | Persistent recovery |
| --- | --- | --- |
| Success | 4 seconds | Result remains in the destination |
| Information, including cancellation | 5 seconds | Current operation state |
| Warning with a recoverable result | 8 seconds | Badge/Analysis entry remains |
| Error or choice required | None | Field/operation state remains until resolved |
| Progress | None; appears after 400 ms | Real Cancel control; no invented percentage |

One operation owns one stable ID: progress becomes success/warning/error in place.
Fast operations skip progress. Stage changes do not restart the animation or add
cards. Start the terminal timeout only after the result becomes visible.
Pause its remaining time on hover, keyboard focus, or a hidden document; resume
the remaining time afterward. Closing a progress card does not cancel the work.
If it was closed, show the final result once; cancellation must suppress stale
success callbacks. New-file scope invalidates prior-file actions and late results.

Show at most three desktop cards or one narrow card, with 8 px between cards.
Keep existing visible order stable. Coalesce by operation ID before queuing;
prioritize error/required choice, then warning, progress, success/information.
Queue up to eight unique notifications. Overflow success/info can be collapsed
into recent activity; errors must remain discoverable through unresolved badges
and the notification list even when no overlay slot is available.

A small toolbar notification button opens the last 20 terminal results from the
current session. It is not a new permanent side panel and is not stored in the
project or localStorage. Pending/unresolved current-operation states are retained
separately from that limit. Dismissing a toast acknowledges display, not resolution.
Historical entries from an obsolete file have no stale Retry/Undo/Download action.

## Motion and accessibility

- Intent: calm acknowledgement with a clear destination. No bouncing, shaking,
  confetti, pulsing backgrounds or continuous decorative animation.
- Enter: 220 ms, `cubic-bezier(.2,0,0,1)`, translate Y −8 px to 0 with opacity
  0 to 1. Shadow settles with the card. Exit: 150 ms, `cubic-bezier(.3,0,1,1)`,
  opacity to 0 with translate Y −4 px. Stack movement: 180 ms using transforms.
- Reduced motion: no translation or stack movement; instant or ≤100 ms fade.
  Progress uses static text instead of a spinning indicator. Do not animate layout
  measurements or start independent ambient effects while a performance plays.
- Use one polite announcer for results and a separate assertive announcer only for
  blocking failures. Visible cards are not additional live regions. Deduplicate
  announcements and never narrate each progress tick or announce history again.
- Toasts never steal focus. Actions are normal buttons. Escape dismisses only a
  toast whose controls have focus, leaving the existing inspector Escape behavior.
  When a focused card closes, move focus to the next notification control or the
  notification-history button. An action focuses its destination control/heading.
- Keep field labels, units, field errors, active mapping and unresolved badges
  outside temporary notices. Preserve `.midi-roll-sr` and `aria-describedby`; the
  long piano-roll text in copied accessibility output is already visually hidden.

## Implementation boundary and migration

`NotificationStore` owns IDs, scoped invalidation, timers, priority and the bounded
queue/history. `NotificationFeedback` coalesces nested work into its owning operation,
retaining required choices ahead of recoverable warnings. The keyed DOM view in
`notifications.ts` owns cards, history, focus and announcements; `notifications.css`
uses existing app tokens. Typed content carries localized strings and optional
labelled action callbacks. Obsolete actions are removed on scope invalidation.
No raw HTML or exception dump enters a card; full errors open Task details in Analysis.

Callers begin feedback around complete commit/rollback boundaries. Progress updates
the same ID without re-announcing every stage; nested fitting/synthesis does not
create its own completion. The existing source-generation checks reject obsolete
async work, and the store rejects stale terminal updates. `renderDiagnostics()`
updates persistent evidence only. Ordinary playback, camera and seek controls do
not create result toasts. Numeric export progress and Cancel remain in Export.

| Surface | Implemented destination |
| --- | --- |
| Previous `#toast` | Shared store/view and stage-anchored viewport |
| `#message` bottom strip | Remove duplicate result paragraphs; retain compact stage status |
| `#automatic-mapping`, `#mapping-evidence` | Result toast; coverage table in Analysis; selected control value remains |
| `#source-summary` | Analysis badge/link only when attention is needed |
| `#project-message`, `#character-message`, `#calibration-message`, `#model-message`, `#arrangement-status` | Audit each event: result to toast, current state to compact value, field correction inline |
| `#wav-info`, `#fit-state` | Retain short current state; move completion/error prose to the center |
| Persistent file/model caveats | One collapsed help destination; keep attribution intact |
| Export status | Real progress/cancel at Export; update one toast and retain download link |

Do not remove every `.hint` or `role="status"` with a bulk replacement. Missing
project assets, unsaved/applied settings and failed fitting remain findable after
the toast disappears. A mapping warning must not be relabeled as a motion failure.
Rust interpretation and planning remain unchanged by this UI migration.

## Verification boundary

Verify normal/partial/ambiguous/failed import; cancellation and rapid replacement;
one result per operation; queue overflow; persistent recovery after dismissal;
stale action prevention; hover/focus/visibility timer pause; keyboard routing;
no duplicate announcements; English/Korean; light/dark; reduced motion; 200% zoom;
desktop with inspector open/closed and a 390 px narrow layout.
Capture actual UI evidence and test lifecycle behavior with controlled timers.
Document checks alone do not verify these production behaviors.

See [implementation evidence](../../docs/verification/notification-design.md#production-integration) for executed checks and remaining manual device/accessibility coverage.
