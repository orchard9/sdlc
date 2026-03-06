# Design: AskPonderModal

Third utility button in the sidebar bottom-left — "Ask Ponder" — opens a modal for asking how features, code, and systems work. Reuses the existing AMA infrastructure with no backend changes.

## Layout

The modal follows the same fixed overlay + centered card pattern as `FixRightAwayModal`:
- `fixed inset-0 z-50 flex items-start justify-center pt-[12vh] bg-black/60`
- Card: `max-w-xl`, rounded-xl, shadow-2xl

## States

1. **input** — textarea (autofocused), ⌘↵ hint, Ask button
2. **answering** — pulsing indicator, source file chips appear, answer streams via AmaAnswerPanel
3. **answered** — full answer rendered, source chips, "Ask another" + "Open as Thread" footer

## Sidebar addition

New button after Search in the bottom utility section. Same visual style as Fix Right Away and Search. `HelpCircle` icon, label "Ask Ponder", kbd hint `⌘/`.

[Mockup](mockup.html)
