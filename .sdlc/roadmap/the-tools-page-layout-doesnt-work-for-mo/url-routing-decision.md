# Architecture Decision: URL Routing for ToolsPage

## Decision
ToolsPage should adopt URL-based routing (/tools/:name) to align with PonderPage's established pattern.

## Why not the minimal fix (back button prop)?
The back button prop with local state is a simulation of navigation — it mimics what the browser already provides when pages use the router. It solves one symptom (can't go back on mobile) without addressing the root cause.

## Root cause
ToolsPage uses `useState<string|null>` for tool selection. Every other master-detail page in this app (PonderPage, InvestigationPage, EvolvePage, GuidelinePage, KnowledgePage) uses URL routing. ToolsPage was implemented inconsistently.

## What URL routing gives us
- Browser back button works naturally
- Deep links to specific tools (/tools/quality-check)
- ToolCard becomes a real link (right-click → Open in new tab)
- Mobile back is `navigate('/tools')` — identical to PonderPage
- showMobileDetail = \!\!name — identical pattern

## Why not extract a shared MasterDetailLayout component?
Convention alignment (same URL pattern, same widths, same back button pattern) is sufficient. A shared component adds coupling between five pages. The cost of enforced consistency is paid every time you debug or evolve any of those pages.

## Changes required
1. App.tsx: Add routes `/tools` and `/tools/:name`
2. ToolsPage: Replace useState with useParams() + useNavigate()
3. ToolsPage: Left panel width w-64 → w-72 (match PonderPage)
4. ToolRunPanel: Add onBack prop with md:hidden ArrowLeft button
5. ToolsPage: Remove auto-select-first-tool behavior on load
6. showMobileDetail = \!\!name (identical to PonderPage's \!\!slug)

## Scope
~50-70 lines changed. Two files: ToolsPage.tsx + App.tsx router config.
