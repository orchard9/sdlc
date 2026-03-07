# QA Plan: AgentsPage Two-Tier Display

## Q1: Both sections render with agents
- Navigate to /agents with both project and workstation agents present
- Verify "Project Team" section appears first with correct count
- Verify "Workstation" section appears second with correct count and "not shared" warning
- Verify AgentCard expand/collapse works in both sections

## Q2: Project agents empty, workstation populated
- Navigate to /agents with no `.claude/agents/` in the project but agents in `~/.claude/agents/`
- Project Team section shows empty state with guidance text
- Workstation section shows agent cards normally

## Q3: Both sections empty
- Navigate to /agents with no agents in either location
- Full-page empty state is displayed (not two empty sections)

## Q4: Independent error handling
- If `/api/project/agents` fails, workstation section still renders
- If `/api/agents` fails, project section still renders
- Error message appears only in the failed section

## Q5: Build verification
- `npm run build` completes without errors
- No TypeScript or lint errors
