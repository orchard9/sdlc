# Green State: Weekly Quote System

## Concept
When git status is fully clean (no uncommitted changes, no unpushed commits, in sync with origin), the status area displays a subtle, slowly-animating quote from a famous physicist, mathematician, or philosopher. The quote rotates weekly based on ISO week number.

## Behavior

- **Trigger**: `severity === "green"` (all clean)
- **Selection**: `quotes[weekOfYear % quotes.length]` — deterministic, same quote all week
- **Animation**: Fade-in on mount, gentle opacity pulse (0.6 → 1.0, 4s cycle) — subtle enough to not distract from sidebar navigation
- **Layout**: Small text below the status chip area, max 2 lines with ellipsis, tooltip shows full quote + attribution
- **Typography**: `text-xs italic text-gray-500` — deliberately understated

## Quote Corpus (52 entries, one per week)

Curated from physicists, mathematicians, and philosophers. Short quotes only (< 120 chars) to fit the space. Examples:

| Week | Quote | Attribution |
|------|-------|------------|
| 1 | "The only way to do great work is to love what you do." | Euler (attributed) |
| 2 | "Imagination is more important than knowledge." | Einstein |
| 3 | "The book of nature is written in mathematics." | Galileo |
| 4 | "I think, therefore I am." | Descartes |
| 5 | "We are what we repeatedly do." | Aristotle |
| 6 | "Simplicity is the ultimate sophistication." | da Vinci |
| 7 | "The universe is under no obligation to make sense to you." | Tyson |
| 8 | "Do not worry about your difficulties in mathematics." | Einstein |
| 9 | "Nature uses only the longest threads to weave her patterns." | Feynman |
| 10 | "The important thing is not to stop questioning." | Einstein |
| ... | (full 52-entry list built at implementation time) | |

## Implementation Notes

- Store quotes as a static array in a `quotes.ts` module — no API call needed
- Week number via `getISOWeek(new Date())` (date-fns or manual calc)
- CSS animation: `@keyframes gentle-pulse { 0%, 100% { opacity: 0.7 } 50% { opacity: 1 } }`
- Animation duration: 4s, `animation-timing-function: ease-in-out`
- Respects `prefers-reduced-motion` — static display, no pulse

⚑ Decided: Week-of-year deterministic quote rotation
⚑ Decided: Physicist/mathematician/philosopher quotes only, < 120 chars
⚑ Decided: Subtle pulse animation, respects prefers-reduced-motion
⚑ Decided: text-xs italic gray — deliberately understated, not a feature