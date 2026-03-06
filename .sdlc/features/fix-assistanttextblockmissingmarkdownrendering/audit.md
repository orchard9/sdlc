# Audit: Fix AssistantTextBlock Missing Markdown Rendering

## Security

| # | Check | Result |
|---|-------|--------|
| 1 | XSS via markdown injection | Safe — `react-markdown` renders to React elements, not `dangerouslySetInnerHTML`. User-controlled text cannot inject scripts. |
| 2 | Link targets | `CompactMarkdown` already sets `target="_blank" rel="noopener noreferrer"` on links — no opener vulnerability. |

## Performance

| # | Check | Result |
|---|-------|--------|
| 1 | Bundle size impact | None — `CompactMarkdown` (and its `react-markdown`/`remark-gfm` deps) are already in the bundle, used elsewhere. |
| 2 | Render cost | `react-markdown` parses on each render. For activity feed items this is negligible — text blocks are small and the feed is virtualized/paginated. |

## Correctness

| # | Check | Result |
|---|-------|--------|
| 1 | Empty guard preserved | Yes — outer `if (!event.text.trim()) return null` still prevents empty wrapper divs. |
| 2 | Color consistency | `className="text-foreground/90"` passed through `cn()` — matches original `<p>` color. |
| 3 | Typography consistency | `CompactMarkdown` uses `text-xs` and `leading-relaxed` internally — matches original sizing. |

## Findings

No issues found. The change is minimal and introduces no new risks.
