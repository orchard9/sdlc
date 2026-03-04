# Tasks: ToolsPage mobile back navigation fix

## T1 — Add `ArrowLeft` to lucide-react import in ToolsPage.tsx

Add `ArrowLeft` to the existing `lucide-react` import line.

## T2 — Add `onBack` prop to `ToolRunPanelProps` and `ToolRunPanel`

Extend the interface with `onBack: () => void` and destructure it in the function signature.

## T3 — Insert mobile back button in ToolRunPanel header

Add the `<button ... md:hidden ... onClick={onBack}>` element as the first child of the
header flex row, before the tool-name `<div>`.

## T4 — Pass `onBack` from `ToolsPage` to `ToolRunPanel`

In the `ToolsPage` render, update the `<ToolRunPanel>` usage to include
`onBack={() => setSelectedName(null)}`.

## T5 — Verify: build passes with no TypeScript errors

Run `cd frontend && npm run build` (or `SDLC_NO_NPM=1 cargo test --all` for the Rust
side). Confirm no TS errors introduced.
</content>
