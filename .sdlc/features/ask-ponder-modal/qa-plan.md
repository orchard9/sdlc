# QA Plan: AskPonderModal

## Approach

Manual verification via the running frontend. No new backend, so focus is entirely on frontend behavior.

## Checklist

### Sidebar button
- [ ] "Ask Ponder" button visible in bottom-left utility strip, below Search
- [ ] Collapsed sidebar: only HelpCircle icon shown (no label, no kbd hint)
- [ ] Expanded sidebar: icon + "Ask Ponder" label + ⌘/ hint visible
- [ ] Button click opens the modal

### Keyboard shortcut
- [ ] ⌘/ (Mac) or Ctrl+/ (Windows/Linux) opens the modal from any page
- [ ] ⌘/ again while modal is open closes it (toggle behavior)

### Modal — input state
- [ ] Textarea is autofocused when modal opens
- [ ] Ask button is disabled when textarea is empty
- [ ] Ask button enables when text is entered
- [ ] ⌘↵ submits the question
- [ ] Escape closes the modal

### Modal — answering state
- [ ] Modal transitions from input to answering on submit
- [ ] Header shows pulsing "Reading codebase…" indicator
- [ ] Header subtitle shows the question text (truncated)
- [ ] Source file chips appear (at least one)
- [ ] Answer text streams in progressively

### Modal — answered state
- [ ] Modal transitions from answering to answered when streaming completes
- [ ] Source chips display file path and line range
- [ ] Answer is rendered as markdown (bold, code, lists)
- [ ] "Ask another" button is visible (left footer)
- [ ] "Open as Thread" button is visible (right footer)

### Ask another action
- [ ] Clicking "Ask another" resets to input state
- [ ] Question textarea is repopulated with previous question for easy editing
- [ ] Sources and answer are cleared

### Open as Thread action
- [ ] Clicking "Open as Thread" navigates to `/threads/:id`
- [ ] Modal closes after navigation
- [ ] The thread exists and contains the question + answer

### Reset behavior
- [ ] Reopening the modal after closing shows fresh input state
- [ ] Previous answer does not bleed into a new session

## Regression checks
- [ ] Fix Right Away still opens with ⌘⇧F
- [ ] Search still opens with ⌘K
- [ ] No layout shifts in the sidebar bottom strip
