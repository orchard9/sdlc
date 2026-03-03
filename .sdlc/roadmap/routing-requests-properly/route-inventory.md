## Route Inventory — All Agent-Spawning Routes

Every call to `spawn_agent_run` in `crates/sdlc-server/src/routes/runs.rs`:

| Skill Name (proposed) | Handler | Current max_turns | Key Purpose |
|---|---|---|---|
| `feature_run` | `start_run` | 200 | Drive a feature through the state machine |
| `milestone_uat` | `start_milestone_uat` | 200 | Run acceptance tests (has Playwright MCP) |
| `milestone_prepare` | `start_milestone_prepare` | 100 | Pre-flight milestone alignment |
| `milestone_run_wave` | `start_milestone_run_wave` | 200 | Execute parallel feature wave |
| `ponder_chat` | `start_ponder_chat` | 100 | Ideation session with thought partners |
| `ponder_commit` | `commit_ponder` | 100 | Crystallize ponder into milestones |
| `investigation_chat` | `start_investigation_chat` | varies | Root-cause / evolve / guideline workspaces |
| `vision_align` | `start_vision_align` | 40 | Align project to vision |
| `architecture_align` | `start_architecture_align` | 40 | Align code to architecture |
| `team_recruit` | `start_team_recruit` | 40 | Recruit agent thought partners |
| `ama_answer` | `answer_ama` | 5 | Quick Q&A answers |
| `quality_reconfigure` | `reconfigure_quality_gates` | 10 | Plan quality gate reconfiguration |
| `quality_fix` | `fix_quality_issues` | 20 | Fix quality issues |
| `tool_plan` | `plan_tool` | 15 | Plan a new SDLC tool |
| `tool_build` | `build_tool` | 25 | Build a planned tool |
| `tool_evolve` | `evolve_tool` | 20 | Evolve an existing tool |
| `tool_act` | `act_tool` | 20 | General tool action |

All routes currently use `sdlc_query_options()` which:
- Sets `path_to_executable: None` → defaults to `claude`
- Sets `model: None` → Claude CLI picks its own default
- Sets `permission_mode: BypassPermissions`

**Single change point:** `sdlc_query_options(root, max_turns)` is the chokepoint. Adding a third parameter for skill name and reading routing config there would touch 15+ callsites but only change the factory function.