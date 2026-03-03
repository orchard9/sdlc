# Design Artifact Protocol

## When to produce HTML (not Markdown)

Produce a self-contained HTML mockup when the design is about a **user interface** —
a screen, panel, modal, widget, layout, or interaction flow.

Stay with Markdown for: data model shapes, CLI syntax, API contracts, algorithm sketches.

## HTML format spec

```html
<!DOCTYPE html>
<html lang="en" class="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>[Feature Name] — Design Mockup</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-950 text-gray-100 p-8 font-mono">
  <!-- Prototype banner -->
  <div class="text-xs text-yellow-400 border border-yellow-900 rounded px-3 py-1 inline-block mb-6">
    ⚠ Design Prototype — not production code
  </div>

  <!-- State navigation (when showing multiple states) -->
  <div class="flex gap-2 mb-6">
    <button class="px-3 py-1 rounded bg-gray-800 text-gray-200 text-sm">Empty State</button>
    <button class="px-3 py-1 rounded bg-gray-700 text-white text-sm">Populated</button>
    <button class="px-3 py-1 rounded bg-gray-800 text-gray-200 text-sm">Loading</button>
  </div>

  <!-- Mockup content here -->
  <!-- Use placeholder data only — no real data, no complex animations -->
</body>
</html>
```

## Filename convention

`<descriptive-name>-mockup.html`

Examples:
- `dashboard-layout-mockup.html`
- `thread-detail-mockup.html`
- `quota-panel-mockup.html`

## Capture command

```bash
# Write HTML to a temp file first
# Write tool → /tmp/<name>-mockup.html

# Then capture into the ponder scrapbook
sdlc ponder capture <slug> --file /tmp/<name>-mockup.html --as <name>-mockup.html
```

## Checklist for a good mockup

- [ ] Dark theme (bg-gray-950 body)
- [ ] Prototype banner present
- [ ] Shows 2–3 key states (empty, populated; before/after; states A/B)
- [ ] Tailwind CDN only — no other external deps
- [ ] Self-contained — no imports, no build step required
- [ ] Placeholder data throughout — no real content
- [ ] Readable without running — static or minimal JS only
