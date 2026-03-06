# Acceptance Test: v41-artifact-share-capture

## Scenario 1: Copy markdown from artifact

1. Navigate to any feature's artifact panel (e.g. a spec or design)
2. Open an artifact (e.g. spec.md)
3. Observe: [MD] [IMG] [↓] buttons are visible in the artifact header
4. Click [MD]
5. Expect: button flashes green with "Copied" for 2 seconds
6. Paste into a text editor
7. Expect: full raw markdown content is pasted

## Scenario 2: Copy artifact as image

1. Navigate to any feature's artifact panel
2. Open a markdown artifact
3. Click [IMG]
4. Expect: button shows "Capturing…" briefly, then flashes green with "Copied"
5. Open Slack or Discord compose box
6. Paste (Cmd+V / Ctrl+V)
7. Expect: rendered artifact appears as an inline image — no file, no attachment

## Scenario 3: Download artifact as PNG

1. With an artifact open, click [↓]
2. Expect: PNG file downloads to Downloads folder
3. Expect: file renders the artifact as it appeared in the UI

## Scenario 4: HTML artifact — image buttons absent

1. Open a `.html` artifact (e.g. a design mockup)
2. Expect: only [MD] button is shown — [IMG] and [↓] are not present

## Scenario 5: Copy a ponder dialogue message

1. Navigate to any ponder entry with session history
2. Hover over any message bubble
3. Expect: copy icon appears in the top-right corner of the bubble
4. Click the icon
5. Expect: icon flashes green check for 2 seconds
6. Paste into another tool
7. Expect: message text is pasted

## Scenario 6: Mobile dialogue copy (touch)

1. Open a ponder session on a mobile device (or narrow window / touch simulation)
2. Observe: copy icons are always visible on message bubbles (no hover required)
3. Tap a copy icon
4. Expect: message text copies to clipboard

## Scenario 7: Clipboard permission denied fallback

1. In a browser with clipboard-write permission blocked
2. Click [IMG] on any artifact
3. Expect: PNG file downloads automatically (no silent failure, no error modal)
