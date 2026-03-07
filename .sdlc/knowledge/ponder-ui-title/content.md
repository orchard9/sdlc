## Design Decisions

**⚑ Decided: Separator is `·` (middle dot), not `$`**
The `$` reads as shell variable or currency. Middle dot is the industry standard (GitHub, Google, Slack). Clean and unambiguous at small tab widths.

**⚑ Decided: Format is `PROJECT · FOCUS · Ponder`**
Project name is leftmost because it is the highest-value differentiator when a user has multiple Ponder instances open (one per project). Focus content is second because it changes most frequently and helps distinguish tabs within the same project. "Ponder" is rightmost as branding — least valuable for identification since every tab has it.

**⚑ Decided: Start with Option A (slug-based, AppShell only)**
Zero new abstractions. A single `useEffect` in AppShell. Detail views show the URL slug which is human-readable by convention. Can evolve to context-based approach later if display names are needed.

**⚑ Decided: Project name updates at runtime**
`projectName` state already comes from the config API and is set on mount. If the project name changes (e.g., via settings), the title updates on next navigation. No polling needed — users rarely rename projects mid-session.

**? Open: Should Hub page use a different format?**
Hub page (`/hub`) manages multiple projects. Title could be `Ponder Hub` instead of `PROJECT · Hub · Ponder`. Low priority — hub runs in its own tab context.
