# Maschine Mikro MK3 manual notes

Source: `Maschine_Mikro_MK3_Manual_English_06_12_23.pdf` (Native Instruments,
revision 06/12/23). These notes record the Mikro-specific controller behavior that
matters to DECKARD's built-in surface. The USB report details remain in
`docs/mk3-usb.md`.

## Confirmed controller behavior

- The magnifying-glass `BROWSER` button toggles the software Browser. It is not
  only a controller page selector (manual p. 27).
- `PROJECT` opens the recent-project list; the encoder selects and loads a recent
  project. DECKARD maps this to the existing File page's Recent/Load workflow.
- `FAVORITES` opens the Browser with its Favorites filter active; the encoder
  selects and loads a filtered result. `SHIFT + PROJECT` is the controller's
  separate fixed MIDI Mode and is not a project/browser action (manual pp. 9, 64).
- The encoder is both rotary and push-button. The arrow buttons change the
  context or page around the encoder (manual pp. 9–10, 36–37).
- `VOLUME`, `SWING`, and `TEMPO` select quick-edit modes for the encoder. Holding
  a Group button or pad applies the edit to that Group or Sound; `SHIFT` gives
  finer increments (manual pp. 30–31).
- `PITCH`, `MOD`, `PERFORM`, and `NOTES` select Smart Strip modes. The strip is
  inactive when none is selected; `SHIFT + PERFORM` selects a Perform FX (manual
  pp. 11–12).
- `GROUP` held with pads 9–16 selects Groups. `SHIFT + GROUP` enters Step mode;
  `SHIFT` with Group selection accesses additional banks (manual p. 11).
- `REC` starts recording, `SHIFT + REC` starts count-in recording when stopped,
  and holding `REC` opens Pattern Preset mode (manual p. 12).
- `SHIFT + FOLLOW` opens Grid mode. `SHIFT + TAP` toggles the metronome.
- `SHIFT + NOTE REPEAT` switches to Keyboard mode and activates the Arp; Note
  Repeat is normally held, but Arp mode can be pinned (manual p. 12).
- `MUTE`, `SOLO`, and `SELECT` are temporary pad/Group modes. `DUPLICATE`,
  `VARIATION`, `EVENTS`, `PATTERN`, and `SCENE` are editing or navigation modes
  (manual pp. 13–14).
- `PAD MODE`, `KEYBOARD`, `CHORDS`, and `STEP` are the four pad input modes.
  `FIXED VEL` applies across them; `SHIFT + FIXED VEL` enables 16 Velocities in
  Pad mode (manual pp. 14–15).

## Pattern mode

- `PATTERN` enters Pattern mode. Existing Patterns are dimly lit, the selected
  Pattern is fully lit, and empty slots are dark (manual p. 299).
- `PATTERN +` an existing pad selects that Pattern; `PATTERN +` an empty pad
  creates a new Pattern in that slot (manual pp. 300–302).
- Official Mikro behavior uses the right arrow to enter Pattern-bank selection,
  then the encoder chooses the bank (manual pp. 300–301).
- DECKARD follows the manual's Group-first workflow: select the desired Group,
  enter Pattern mode, then use the 4x4 pads to select or create that Group's
  Pattern slots. LEFT/RIGHT remain Pattern-page navigation controls.

## Navigation mode

- `SHIFT + VARIATION` enters Navigate mode.
- In Pattern Editor navigation: pad 1 scrolls left, pad 3 scrolls right, pad 6
  zooms in, and pad 2 zooms out.
- In Song view navigation: pad 9 scrolls left, pad 11 scrolls right, pad 14
  zooms in, and pad 10 zooms out (manual p. 37).

## DECKARD implementation priorities

1. Keep Browser button tied to the GUI Browser toggle.
2. Keep the encoder push routed as `BUTTON_WHEEL`; the Mikro decoder exposes it
   through the canonical event path and the existing pages handle it.
3. Add Group-held pad selection and Group-bank navigation.
4. Add Navigate mode shortcuts for Pattern Editor and Song view once viewport
   actions exist at the application boundary.
5. Add Pattern-bank selection without changing the Group-first pad workflow.
6. Complete Smart Strip mode behavior and mode-specific LED feedback.
