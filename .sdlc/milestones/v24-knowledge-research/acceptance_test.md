# Acceptance Test: v24-knowledge-research

## Prerequisites
- sdlc server running at http://localhost:7777
- At least one knowledge entry exists in the knowledge base

## Checklist

### 1. Knowledge page loads with entry list
- [ ] Navigate to http://localhost:7777/knowledge
- [ ] Verify the knowledge page renders with a list of entries

### 2. Research button visible on list entries
- [ ] Hover over an entry in the knowledge list
- [ ] Verify a Research icon button is visible on the entry row

### 3. NewResearchModal opens from list
- [ ] Click the Research button on an entry row
- [ ] Verify a modal opens with the entry title in the header
- [ ] Verify the modal contains a "Topic hint" text field
- [ ] Verify Cancel and "Start Research" buttons are present

### 4. Modal closes on Escape
- [ ] Press Escape while the modal is open
- [ ] Verify the modal closes without triggering a research run

### 5. Research modal submits successfully
- [ ] Re-open the Research modal on an entry
- [ ] Leave the topic field empty and click "Start Research"
- [ ] Verify the modal closes (API call accepted — 200 or run started)

### 6. Research with custom topic
- [ ] Re-open the Research modal on an entry
- [ ] Type a custom topic (e.g. "error handling patterns")
- [ ] Click "Start Research"
- [ ] Verify the modal closes successfully

### 7. Server research endpoint has web tools
- [ ] Call GET /api/knowledge to verify entries exist
- [ ] Confirm the research endpoint code includes WebSearch and WebFetch in allowed_tools (verified via code inspection)
