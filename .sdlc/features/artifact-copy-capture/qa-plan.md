# QA Plan: Artifact and Dialogue Copy & Screenshot

## Scope

Validate copy and screenshot functionality across all specified surfaces. Confirm backward compatibility of `CopyButton` changes. Verify graceful degradation for HTML artifacts and clipboard permission denial.

## Test Cases

### TC-1: CopyButton label prop — backward compatibility
- **Setup:** Open any page using `CommandBlock` (e.g. a feature detail page with CLI command shown)
- **Steps:** Verify command copy button renders as icon-only (no label text visible)
- **Expected:** Existing icon-only CopyButton unaffected — no regression

### TC-2: CopyButton with label
- **Setup:** Open WorkspacePanel with a markdown artifact active
- **Steps:** Observe the artifact header
- **Expected:** [MD] button shows copy icon + "MD" text label

### TC-3: Copy markdown from WorkspacePanel
- **Setup:** Open any ponder or feature artifact panel; select a `.md` artifact
- **Steps:** Click [MD] button
- **Expected:** Button shows green "Copied" flash for 2 seconds; clipboard contains the raw markdown text of the artifact

### TC-4: Copy as image from WorkspacePanel
- **Setup:** Open any `.md` artifact
- **Steps:** Click [IMG] button
- **Expected:** Button shows "Capturing…" briefly then "Copied"; can paste as inline image in Slack/Discord/email

### TC-5: Download PNG from WorkspacePanel
- **Setup:** Open any `.md` artifact
- **Steps:** Click [↓] (download) button
- **Expected:** PNG file downloads; file renders the artifact content visually

### TC-6: HTML artifact — image buttons absent
- **Setup:** Open a WorkspacePanel with an `.html` artifact (e.g. a design mockup)
- **Steps:** Observe the artifact header
- **Expected:** Only [MD] button is visible; [IMG] and [↓] are not rendered

### TC-7: HTML artifact markdown copy still works
- **Setup:** Same as TC-6
- **Steps:** Click [MD]
- **Expected:** Raw HTML source code is copied to clipboard

### TC-8: Dialogue message hover copy (desktop)
- **Setup:** Open any ponder with session history on desktop browser
- **Steps:** Hover over a `PartnerMessage` bubble
- **Expected:** Copy icon appears in top-right corner of the bubble on hover

### TC-9: Dialogue message copy content
- **Setup:** Same as TC-8
- **Steps:** Click the copy icon on a message with known text
- **Expected:** Icon flashes green check; clipboard contains the message's plain text

### TC-10: Mobile dialogue copy (always visible)
- **Setup:** Open a ponder session on mobile viewport (or use DevTools to simulate touch)
- **Steps:** Observe message bubbles without hovering
- **Expected:** Copy icon is always visible on message bubbles (no hover required)

### TC-11: Clipboard permission denied fallback
- **Setup:** Block clipboard-write in browser settings (or use a browser context with permission denied)
- **Steps:** Click [IMG]
- **Expected:** PNG file downloads automatically; no error UI shown; button shows "Saved" state

### TC-12: CaptureButton bundle impact
- **Setup:** Check network tab on initial page load
- **Steps:** Load any page without clicking [IMG]; check loaded scripts
- **Expected:** html2canvas is NOT in the initial bundle (only loaded on first [IMG] click)

## Regression Checks

- [ ] `CommandBlock` copy buttons still work (icon-only, no label)
- [ ] All existing artifact panels load without errors
- [ ] No TypeScript errors in modified files
- [ ] `SDLC_NO_NPM=1 cargo test --all` passes (backend unaffected)
