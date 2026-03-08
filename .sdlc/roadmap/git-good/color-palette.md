# Colorblind-Safe Diff Palette

## Principle
No red/green combinations. Use blue/orange as the primary axis — distinguishable across all common forms of color vision deficiency (protanopia, deuteranopia, tritanopia).

## Color Tokens (CSS Custom Properties)

```css
:root {
  /* Content additions — blue family */
  --diff-add-bg: #dbeafe;           /* blue-100 */
  --diff-add-bg-highlight: #bfdbfe; /* blue-200 — inline word highlight */
  --diff-add-border: #3b82f6;       /* blue-500 — gutter accent */

  /* Content deletions — orange/amber family */
  --diff-del-bg: #fef3c7;           /* amber-100 */
  --diff-del-bg-highlight: #fde68a; /* amber-200 — inline word highlight */
  --diff-del-border: #f59e0b;       /* amber-500 — gutter accent */

  /* Whitespace changes — same families but very muted */
  --diff-ws-add-bg: #eff6ff;        /* blue-50 — barely there */
  --diff-ws-del-bg: #fffbeb;        /* amber-50 — barely there */

  /* Unchanged context */
  --diff-context-bg: #ffffff;
  --diff-context-text: #374151;     /* gray-700 */

  /* Dark mode variants */
  --diff-add-bg-dark: #1e3a5f;
  --diff-del-bg-dark: #4a3520;
  --diff-ws-add-bg-dark: #172340;
  --diff-ws-del-bg-dark: #3a2a18;
}
```

## Design Rationale

1. **Blue for additions, amber/orange for deletions** — maximally distinguishable for all color vision types
2. **Whitespace uses the same hue family** at much lower saturation (50-level vs 100/200-level) — visible on close inspection but does not compete with content changes
3. **Content changes use two tiers**: background (line-level) and highlight (word-level inline diff) — the highlight tier is one step darker in the same family
4. **Gutter accents** use the 500-level of each family — strong enough to scan vertically
5. **WCAG AA contrast** maintained for all text-on-background combinations

## Accessibility Testing Checklist

- [ ] Sim test with Coblis (colorblind simulator)
- [ ] Verify 4.5:1 contrast ratio for all text
- [ ] Test with forced-colors media query (Windows High Contrast)
- [ ] Verify whitespace changes are distinguishable from unchanged context

⚑ Decided: Blue/amber palette — no red/green anywhere in diffs
⚑ Decided: Whitespace uses same color family at lower saturation, not a different color
⚑ Decided: Two-tier highlighting — line background + inline word highlight