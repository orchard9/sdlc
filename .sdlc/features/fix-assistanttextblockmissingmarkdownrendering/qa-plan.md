# QA Plan: Fix AssistantTextBlock Missing Markdown Rendering

## Test Cases

### TC-1: Code fences render as styled code blocks
1. Trigger an agent run that produces output containing triple-backtick code fences.
2. Verify the activity feed renders the code in a `<pre>` block with monospace font and background styling, not as raw triple-backtick text.

### TC-2: Inline code renders with code styling
1. Trigger an agent run that produces output with backtick-wrapped inline code.
2. Verify inline code appears with monospace font, background highlight, and border — not as raw backtick characters.

### TC-3: Bold and italic text renders correctly
1. Verify `**bold**` renders as `<strong>` and `*italic*` renders as `<em>` in agent text blocks.

### TC-4: Lists render as HTML lists
1. Verify markdown lists (`- item`, `1. item`) render as `<ul>`/`<ol>` with proper indentation, not as raw dash/number text.

### TC-5: Plain text without markdown renders cleanly
1. Agent output that contains no markdown syntax should render as a simple paragraph with no visual regression from the previous `<p>` rendering.

### TC-6: Empty text blocks still suppressed
1. Verify that `event.text` that is empty or whitespace-only still returns `null` (no empty DOM element rendered).
