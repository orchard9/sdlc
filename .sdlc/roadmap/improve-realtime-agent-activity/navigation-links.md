# Navigation Links from Agent Activity Tiles

## Requirement (from Xist)
Add a link to the milestone or feature the agent is working on so users can quickly navigate from an agent activity tile to the detailed information about that agent run in the UI.

## Data Available
`RunRecord` already has everything needed:
- `run_type`: feature, milestone_uat, milestone_prepare, milestone_run_wave, ponder, investigation, vision_align, architecture_align
- `target`: the slug of the entity being worked on

## Route Mapping
| run_type | Route |
|---|---|
| feature | `/features/{target}` |
| milestone_uat | `/milestones/{target}` |
| milestone_prepare | `/milestones/{target}` |
| milestone_run_wave | `/milestones/{target}` |
| ponder | `/ponder/{target}` |
| investigation | `/investigations/{target}` |
| vision_align | `/` (project-level) |
| architecture_align | `/` (project-level) |

## Implementation
Add a small link icon or clickable target name in the RunCard header row (next to the label, before the chevron). Use react-router `Link` component. For vision_align/architecture_align, either omit the link or point to project root.

This is zero backend work — purely a frontend addition to `RunCard.tsx`.