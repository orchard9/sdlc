export type Phase =
  | 'draft'
  | 'specified'
  | 'planned'
  | 'ready'
  | 'implementation'
  | 'review'
  | 'audit'
  | 'qa'
  | 'merge'
  | 'released'

export type ActionType =
  | 'create_spec'
  | 'approve_spec'
  | 'create_design'
  | 'approve_design'
  | 'create_tasks'
  | 'approve_tasks'
  | 'create_qa_plan'
  | 'approve_qa_plan'
  | 'implement_task'
  | 'fix_review_issues'
  | 'create_review'
  | 'approve_review'
  | 'create_audit'
  | 'approve_audit'
  | 'run_qa'
  | 'approve_merge'
  | 'merge'
  | 'archive'
  | 'unblock_dependency'
  | 'wait_for_approval'
  | 'done'

export type ArtifactStatus = 'missing' | 'draft' | 'approved' | 'rejected' | 'needs_fix' | 'passed' | 'failed' | 'waived'
export type TaskStatus = 'pending' | 'in_progress' | 'completed' | 'blocked'
export type MilestoneStatus = 'active' | 'verifying' | 'released' | 'skipped'

export type UatVerdict = 'pass' | 'pass_with_tasks' | 'failed'

export interface UatRun {
  id: string
  milestone_slug: string
  started_at: string
  completed_at: string | null
  verdict: UatVerdict
  tests_total: number
  tests_passed: number
  tests_failed: number
  playwright_report_path: string | null
  tasks_created: string[]
  summary_path: string
  screenshots: string[]       // filenames; empty array if none
}

export interface PlatformArg {
  name: string
  required: boolean
  choices: string[]
}

export interface PlatformCommand {
  description: string
  script: string
  args: PlatformArg[]
  subcommands: Record<string, string>
}

export interface QualityConfig {
  min_score_to_advance: number
  min_score_to_release: number
  require_all_lenses: boolean
}

export interface PhaseConfig {
  enabled: Phase[]
  required_artifacts: Record<string, string[]>
}

export interface ProjectConfig {
  version: number
  project: { name: string; description: string | null }
  phases: PhaseConfig
  platform: { commands: Record<string, PlatformCommand> } | null
  quality: QualityConfig | null
  observability?: {
    daily_budget_usd?: number
  }
}

// ---------------------------------------------------------------------------
// Query types
// ---------------------------------------------------------------------------

export interface DiagnoseResult {
  title: string
  problem_statement: string
  root_cause: string
  files_affected: string[]
  /** "high" | "medium" | "low" | "none" — "none" means not a software issue */
  confidence: string
}

export interface QuerySearchResult {
  slug: string
  title: string
  phase: string
  score: number
}

export interface QueryPonderSearchResult {
  slug: string
  title: string
  status: PonderStatus
  score: number
}

export interface QuerySearchResponse {
  results: QuerySearchResult[]
  ponder_results: QueryPonderSearchResult[]
  parse_error: string | null
}

export interface QueryTaskSearchResult {
  feature_slug: string
  task_id: string
  title: string
  status: string
  score: number
}

export interface QueryTaskSearchResponse {
  results: QueryTaskSearchResult[]
  parse_error: string | null
}

export interface QueryBlockedItem {
  slug: string
  title: string
  blockers: string[]
}

export interface QueryReadyItem {
  slug: string
  phase: string
  action: string
  message: string
  next_command: string
}

export interface QueryNeedsApprovalItem {
  slug: string
  phase: string
  action: string
  message: string
  next_command: string
}

// ---------------------------------------------------------------------------
// Auth token types
// ---------------------------------------------------------------------------

/** Named tunnel-access token — stored in .sdlc/auth.yaml. */
export interface AuthToken {
  name: string
  created_at: string
}

/** Returned only on creation — the token value shown once. */
export interface CreatedAuthToken extends AuthToken {
  token: string
}

// ---------------------------------------------------------------------------
// Secrets types
// ---------------------------------------------------------------------------

export interface SecretsKey {
  name: string
  type: 'ssh' | 'age'
  short_id: string
  added_at: string
}

export interface SecretsEnvMeta {
  env: string
  key_names: string[]
  updated_at: string
}

export interface FeatureSummary {
  slug: string
  title: string
  description: string | null
  phase: Phase
  archived: boolean
  blocked: boolean
  next_action: ActionType
  next_message: string
  task_summary: string
  updated_at: string
}

export interface MilestoneSummary {
  slug: string
  title: string
  vision: string | null
  status: MilestoneStatus
  features: string[]
  created_at: string
}

export interface MilestoneDetail {
  slug: string
  title: string
  description: string | null
  vision: string | null
  status: MilestoneStatus
  features: string[]
  created_at: string
  updated_at: string
  skipped_at: string | null
}

export interface MilestoneFeatureReview {
  feature: string
  phase: Phase
  action: ActionType
  message: string
}

export interface MilestoneReview {
  milestone: string
  features: MilestoneFeatureReview[]
}

// ---------------------------------------------------------------------------
// Escalation types
// ---------------------------------------------------------------------------

export type EscalationKind = 'secret_request' | 'question' | 'vision' | 'manual_test'
export type EscalationStatus = 'open' | 'resolved'

export interface EscalationSummary {
  id: string
  kind: EscalationKind
  title: string
  context: string
  source_feature: string | null
  created_at: string
}

export interface EscalationDetail extends EscalationSummary {
  linked_comment_id: string | null
  status: EscalationStatus
  resolved_at: string | null
  resolution: string | null
}

export type ParallelWorkItemKind =
  | { type: 'feature'; slug: string; next_action: ActionType }
  | { type: 'uat' }

export type ParallelWorkItem = {
  milestone_slug: string
  milestone_title: string
  command: string
} & ParallelWorkItemKind

export interface ProjectState {
  project: string
  active_features: string[]
  active_directives: ActiveDirective[]
  blocked: BlockedItem[]
  features: FeatureSummary[]
  milestones: MilestoneSummary[]
  escalations: EscalationSummary[]
  parallel_work?: ParallelWorkItem[]
  last_updated: string
}

export interface ActiveDirective {
  feature: string
  action: ActionType
  started_at: string
  timeout_minutes: number
}

export interface BlockedItem {
  feature: string
  reason: string
  since: string
}

export interface Artifact {
  artifact_type: string
  status: ArtifactStatus
  path: string
  content: string | null
  approved_at: string | null
  approved_by: string | null
  rejected_at: string | null
  rejection_reason: string | null
  waived_at: string | null
  waive_reason: string | null
}

export interface Task {
  id: string
  title: string
  description: string | null
  status: TaskStatus
  created_at: string
  started_at: string | null
  completed_at: string | null
  blocker: string | null
}

export interface Comment {
  id: string
  author: string | null
  body: string
  flag: string | null
  target: Record<string, unknown>
  created_at: string
}

export interface FeatureDetail {
  slug: string
  title: string
  description: string | null
  phase: Phase
  archived: boolean
  blocked: boolean
  blockers: string[]
  artifacts: Artifact[]
  tasks: Task[]
  comments: Comment[]
  phase_history: PhaseTransition[]
  dependencies: string[]
  created_at: string
  updated_at: string
}

export interface PhaseTransition {
  phase: Phase
  entered: string
  exited: string | null
}

export interface Classification {
  feature: string
  title: string
  description: string | null
  current_phase: Phase
  action: ActionType
  message: string
  next_command: string
  output_path: string | null
  transition_to: Phase | null
  task_id: string | null
  is_heavy: boolean
  timeout_minutes: number
}

// ---------------------------------------------------------------------------
// Ponder / Roadmap types
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Prepare / Wave types
// ---------------------------------------------------------------------------

export type ProjectPhaseType = 'idle' | 'pondering' | 'planning' | 'executing' | 'verifying'

export interface ProjectPhase {
  phase: ProjectPhaseType
  milestone?: string
}

export type GapSeverity = 'blocker' | 'warning' | 'info'

export interface Gap {
  feature: string
  severity: GapSeverity
  message: string
}

export interface WaveItem {
  slug: string
  title: string
  phase: Phase
  action: string
  needs_worktree: boolean
  blocked_by: string[]
}

export interface Wave {
  number: number
  label: string
  items: WaveItem[]
  needs_worktrees: boolean
}

export interface BlockedFeatureItem {
  slug: string
  title: string
  reason: string
}

export interface MilestoneProgress {
  total: number
  released: number
  in_progress: number
  blocked: number
  pending: number
}

export interface PrepareResult {
  project_phase: ProjectPhase
  milestone?: string
  milestone_title?: string
  milestone_progress?: MilestoneProgress
  gaps: Gap[]
  waves: Wave[]
  blocked: BlockedFeatureItem[]
  next_commands: string[]
}

export interface Enrichment {
  source: string
  content: string
  added_at: string
}

export interface FeedbackNote {
  id: string
  content: string
  created_at: string
  updated_at: string | null
  enrichments: Enrichment[]
}

// ---------------------------------------------------------------------------
// Ponder / Roadmap types
// ---------------------------------------------------------------------------

export type PonderStatus = 'exploring' | 'converging' | 'committed' | 'parked'

// ---------------------------------------------------------------------------
// Advisory types
// ---------------------------------------------------------------------------

export type MaturityStage = 'health' | 'consistency' | 'refactor' | 'structure' | 'roadmap' | 'advanced'
export type FindingStatus = 'open' | 'acknowledged' | 'resolved' | 'dismissed'

export interface Finding {
  id: string
  stage: MaturityStage
  title: string
  description: string
  status: FindingStatus
  created_at: string
  resolved_at: string | null
}

export interface AdvisoryRun {
  run_at: string
  file_count: number | null
  stage_reached: MaturityStage
  summary: string
}

export interface AdvisoryHistory {
  runs: AdvisoryRun[]
  findings: Finding[]
}

export interface AdvisorySseEvent {
  type: 'advisory_run_completed' | 'advisory_run_stopped'
}

export interface MilestoneUatSseEvent {
  type: 'milestone_uat_completed' | 'milestone_uat_failed'
  slug: string
}

export interface PonderSummary {
  slug: string
  title: string
  status: PonderStatus
  tags: string[]
  artifact_count: number
  team_size: number
  sessions: number
  created_at: string
  updated_at: string
  committed_at: string | null
  committed_to: string[]
  merged_into: string | null
  merged_from: string[]
  last_session_preview?: string | null
}

export interface PonderTeamMember {
  name: string
  role: string
  context: string
  agent: string
  recruited_at: string
}

export interface PonderArtifact {
  filename: string
  size_bytes: number
  modified_at: string
  content: string | null
}

export interface PonderOrientation {
  current: string
  next: string
  commit: string
}

export interface SessionMeta {
  session: number
  timestamp: string | null
  orientation: PonderOrientation | null
}

export interface SessionContent extends SessionMeta {
  content: string
}

// Ponder run state (from SSE ponder events)
export type PonderRunState =
  | { status: 'idle' }
  | { status: 'running'; session: number; ownerName: string; ownerMessage: string | null }
  | { status: 'stopped'; session: number }

export interface PonderChatResponse {
  status: 'started' | 'conflict'
  session: number
  owner_name: string
}

// SSE ponder event payload (event: "ponder")
export interface PonderSseEvent {
  type: 'ponder_run_started' | 'ponder_run_completed' | 'ponder_run_stopped'
  slug: string
  session?: number
}

// ---------------------------------------------------------------------------
// Agent run tracking (panel)
// ---------------------------------------------------------------------------

export type RunStatus = 'running' | 'completed' | 'failed' | 'stopped'
export type RunType = 'feature' | 'milestone_uat' | 'milestone_prepare' | 'milestone_run_wave' | 'ponder' | 'investigation' | 'vision_align' | 'architecture_align'

export interface RunRecord {
  id: string
  key: string
  run_type: RunType
  target: string
  label: string
  status: RunStatus
  started_at: string
  completed_at?: string
  cost_usd?: number
  turns?: number
  error?: string
  prompt?: string | null
}

// ---------------------------------------------------------------------------
// Run telemetry types (GET /api/runs/:id/telemetry)
// ---------------------------------------------------------------------------

/** Raw event as stored in the events sidecar — matches message_to_event output */
export interface RawRunEvent {
  type: 'init' | 'assistant' | 'tool_progress' | 'tool_summary' | 'result' | 'error' | 'status' | 'system' | 'user' | 'stream_event' | 'auth_status' | 'subagent_started' | 'subagent_completed' | 'subagent_progress'
  // Wall-clock timestamp (ISO-8601). Canonical field name — must match message_to_event() in
  // crates/sdlc-server/src/routes/runs.rs which writes obj["timestamp"]. Do NOT rename to "ts".
  timestamp?: string
  // Subagent correlation id — present on subagent_started / subagent_completed / subagent_progress
  task_id?: string
  // init
  model?: string
  tools_count?: number
  mcp_servers?: string[]
  // assistant
  text?: string
  tools?: { name: string; input: unknown }[]
  // Correlated tool ids (for pairing tool calls with results)
  tool_use_id?: string
  tool_use_ids?: string[]
  // tool_progress
  tool?: string
  elapsed_seconds?: number
  // tool_summary
  summary?: string
  // result
  is_error?: boolean
  cost_usd?: number
  turns?: number
  total_tokens?: number
  duration_ms?: number
  // error / status
  message?: string
  status?: string
  // subagent fields
  description?: string
  last_tool_name?: string
}

export interface RunTelemetry {
  run_id: string
  prompt?: string | null
  events: RawRunEvent[]
}

// ---------------------------------------------------------------------------
// Paired/structured events for the activity feed
// ---------------------------------------------------------------------------

export interface PairedInitEvent {
  kind: 'init'
  event: RawRunEvent
  prompt?: string | null
}

export interface PairedToolExchange {
  kind: 'tool_exchange'
  toolName: string
  input?: unknown
  elapsed_seconds?: number
  summary?: string
  isError: boolean
  resultText?: string
}

export interface PairedAssistantText {
  kind: 'assistant_text'
  text: string
}

export interface PairedRunResult {
  kind: 'run_result'
  isError: boolean
  cost_usd?: number
  turns?: number
  text?: string
}

export type PairedEvent =
  | PairedInitEvent
  | PairedToolExchange
  | PairedAssistantText
  | PairedRunResult

export interface RunSseEvent {
  type: 'run_started' | 'run_finished'
  id: string
  key: string
  label?: string
  status?: string
}

export interface PonderDetail {
  slug: string
  title: string
  status: PonderStatus
  tags: string[]
  sessions: number
  orientation: PonderOrientation | null
  created_at: string
  updated_at: string
  committed_at: string | null
  committed_to: string[]
  merged_into: string | null
  merged_from: string[]
  redirect_banner: string | null
  team: PonderTeamMember[]
  artifacts: PonderArtifact[]
}

// ---------------------------------------------------------------------------
// Investigation types
// ---------------------------------------------------------------------------

export type InvestigationKind = 'root_cause' | 'evolve' | 'guideline'
export type InvestigationStatus = 'in_progress' | 'complete' | 'parked'

// Phase names vary by kind:
// root_cause: triage | investigate | synthesize | output | done
// evolve:     survey | analyze | paths | roadmap | output | done
// guideline:  problem | evidence | principles | draft | publish | done
export type InvestigationPhase = string

export interface InvestigationSummary {
  slug: string
  title: string
  kind: InvestigationKind
  phase: InvestigationPhase
  status: InvestigationStatus
  sessions: number
  artifact_count: number
  created_at: string
  updated_at: string
}

export interface LensScores {
  pit_of_success?: string
  coupling?: string
  growth_readiness?: string
  self_documenting?: string
  failure_modes?: string
}

export interface EvidenceCounts {
  anti_patterns: number
  good_examples: number
  prior_art: number
  adjacent: number
}

export interface InvestigationOrientation {
  current: string
  next: string
  commit: string
}

// Artifacts reuse PonderArtifact shape (same fields)
export type InvestigationArtifact = PonderArtifact

export interface InvestigationDetail {
  slug: string
  title: string
  kind: InvestigationKind
  phase: InvestigationPhase
  status: InvestigationStatus
  context: string | null
  sessions: number
  orientation: InvestigationOrientation | null
  created_at: string
  updated_at: string
  // root_cause specific
  confidence: number | null
  output_type: string | null      // "task" | "guideline"
  output_ref: string | null
  // evolve specific
  scope: string | null
  lens_scores: LensScores | null
  output_refs: string[]
  // guideline specific
  guideline_scope: string | null
  problem_statement: string | null
  evidence_counts: EvidenceCounts | null
  principles_count: number | null
  publish_path: string | null
  // always
  artifacts: InvestigationArtifact[]
}

export type InvestigationRunState =
  | { status: 'idle' }
  | { status: 'running'; session: number; ownerName: string; ownerMessage: string | null }
  | { status: 'stopped'; session: number }

export interface InvestigationChatResponse {
  status: 'started' | 'conflict'
  session: number
  owner_name: string
}

export interface InvestigationSseEvent {
  type: 'investigation_run_started' | 'investigation_run_completed' | 'investigation_run_stopped'
  slug: string
  session?: number
}

export interface DocsSseEvent {
  type: 'vision_align_completed' | 'architecture_align_completed' | 'team_recruit_completed'
}

// ---------------------------------------------------------------------------
// Tool Suite types
// ---------------------------------------------------------------------------

export interface SecretRef {
  env_var: string
  description: string
  required: boolean
}

export interface FormField {
  key: string
  field_type: string   // "text"|"textarea"|"code"|"select"|"checkbox"|"date_range"
  label?: string
  placeholder?: string
  options?: string[]
  language?: string
  default?: unknown
}

export interface ResultAction {
  label: string
  icon?: string
  condition?: string
  prompt_template: string
  confirm?: string
}

export interface ToolMeta {
  name: string
  display_name: string
  description: string
  version: string
  requires_setup: boolean
  setup_done?: boolean
  setup_description?: string
  input_schema: Record<string, unknown>
  output_schema: Record<string, unknown>
  built_in?: boolean   // true for tools managed by sdlc init (ama, quality-check)
  // Extended optional fields (Phase 1 delivery)
  secrets?: SecretRef[]
  form_layout?: FormField[]
  streaming?: boolean
  result_actions?: ResultAction[]
  timeout_seconds?: number
  tags?: string[]
  threaded?: boolean
  persist_interactions?: boolean
  // Injected by server on 422 when required env vars are missing
  missing_secrets?: string[]
}

// ---------------------------------------------------------------------------
// Tool interaction history
// ---------------------------------------------------------------------------

export interface ToolInteractionRecord {
  id: string
  tool_name: string
  created_at: string
  completed_at?: string
  input: unknown
  result?: unknown
  status: 'running' | 'streaming' | 'completed' | 'failed'
  tags: string[]
  notes?: string
  streaming_log: boolean
}

// ---------------------------------------------------------------------------
// Tool SSE events (streaming tool runs)
// ---------------------------------------------------------------------------

export interface ToolSseEvent {
  type: 'tool_run_started' | 'tool_run_progress' | 'tool_run_completed' | 'tool_run_failed'
  name: string
  interaction_id: string
  /** Present on tool_run_progress — the parsed NDJSON line emitted by the tool. */
  line?: unknown
  /** Present on tool_run_failed — the error message. */
  error?: string
}

// ---------------------------------------------------------------------------
// AMA thread types (server-backed)
// ---------------------------------------------------------------------------

export interface AmaThread {
  id: string
  title: string
  created_at: string
  updated_at: string
  turn_count: number
  tags: string[]
  committed_to?: string
}

export interface AmaTurnRecord {
  turn_index: number
  question: string
  sources: AmaSource[]
  synthesis?: string
  run_id?: string
  created_at: string
  completed_at?: string
}

export interface AmaThreadDetail extends AmaThread {
  turns: AmaTurnRecord[]
}

export interface ToolResult<T = unknown> {
  ok: boolean
  data?: T
  error?: string
  duration_ms?: number
}

export interface CheckResult {
  name: string
  command: string
  status: 'passed' | 'failed'
  output: string
  duration_ms: number
}

export interface QualityCheckData {
  passed: number
  failed: number
  checks: CheckResult[]
}

export interface AmaSource {
  path: string
  lines: [number, number]
  excerpt: string
  score: number
  stale?: boolean
}

export interface AmaData {
  sources: AmaSource[]
}

export interface AmaThreadTurn {
  question: string
  sources: AmaSource[]
  synthesisRunKey: string
  synthesisText: string | null  // null until streaming completes
  timestamp: string
}

// ---------------------------------------------------------------------------
// Tunnel types
// ---------------------------------------------------------------------------

export interface TunnelStatus {
  active: boolean
  url: string | null
  /** Present whenever a tunnel is active; null otherwise. */
  token: string | null
  port: number
}

export interface AppTunnelStatus {
  active: boolean
  url: string | null
  /** The port of the user's project dev server being tunneled. */
  configured_port: number | null
}

// ---------------------------------------------------------------------------
// Agent types
// ---------------------------------------------------------------------------

export interface AgentDefinition {
  name: string
  description: string
  model: string
  tools: string[]
  content: string
}

// Area artifact frontmatter — parsed from area-N-*.md files by parseInvestigation.ts
export interface AreaArtifactMeta {
  area: string       // "code_paths" | "bottlenecks" | "data_flow" | "auth_chain" | "environment"
  status: 'pending' | 'investigating' | 'finding' | 'hypothesis'
  confidence?: number
  finding?: string   // first non-empty line after frontmatter
}

// ---------------------------------------------------------------------------
// Knowledge base types
// ---------------------------------------------------------------------------

export type KnowledgeStatus = 'draft' | 'published' | 'archived'

export interface KnowledgeCatalogDivision {
  code: string
  name: string
  description: string | null
}

export interface KnowledgeCatalogClass {
  code: string
  name: string
  description: string | null
  divisions: KnowledgeCatalogDivision[]
}

export interface KnowledgeCatalog {
  classes: KnowledgeCatalogClass[]
  updated_at: string | null
}

export interface KnowledgeSource {
  type: string          // "url" | "file" | "workspace" | "manual"
  url: string | null
  path: string | null
  workspace: string | null
  captured_at: string
}

export interface KnowledgeArtifact {
  filename: string
  size_bytes: number
  modified_at: string
}

export interface KnowledgeEntrySummary {
  slug: string
  title: string
  code: string
  status: KnowledgeStatus
  summary: string | null
  tags: string[]
  created_at: string
  updated_at: string
}

export interface KnowledgeEntryDetail extends KnowledgeEntrySummary {
  sources: KnowledgeSource[]
  related: string[]
  origin: string          // "manual" | "harvest" | "research" | "import"
  harvested_from: string | null
  last_verified_at: string | null
  staleness_flags: string[]
  content: string
  artifacts: KnowledgeArtifact[]
}

export interface KnowledgeSseEvent {
  type: 'knowledge_query_started' | 'knowledge_query_completed'
  slug?: string
}

// ---------------------------------------------------------------------------
// FeedbackThread types
// ---------------------------------------------------------------------------

export type ThreadStatus = 'open' | 'synthesized' | 'promoted'

export interface ThreadSummary {
  slug: string
  title: string
  author: string
  status: ThreadStatus
  comment_count: number
  created_at: string
  updated_at: string
  promoted_to: string | null
}

export interface ThreadComment {
  id: string
  author: string
  body: string
  incorporated: boolean
  created_at: string
}

export interface ThreadDetail extends ThreadSummary {
  body: string | null
  body_version: number
  comments: ThreadComment[]
}

// ---------------------------------------------------------------------------
// Orchestrator / Actions types
// ---------------------------------------------------------------------------

export interface OrchestratorActionStatus {
  type: 'pending' | 'running' | 'completed' | 'failed'
  result?: unknown
  reason?: string
}

export interface OrchestratorActionTrigger {
  type: 'scheduled' | 'webhook'
  next_tick_at?: string
}

export interface OrchestratorAction {
  id: string
  label: string
  tool_name: string
  tool_input: unknown
  recurrence_secs: number | null
  status: OrchestratorActionStatus
  trigger: OrchestratorActionTrigger
  created_at: string
  updated_at: string
}

export interface OrchestratorWebhookEvent {
  id: string
  seq: number
  route_path: string
  received_at: string
  outcome: {
    kind: 'received' | 'no_route' | 'routed' | 'dispatch_error'
    route_id?: string
    tool_name?: string
    reason?: string
  }
}

export interface OrchestratorWebhookRoute {
  id: string
  path: string
  tool_name: string
  input_template: string
  created_at: string
}

export interface ActionSseEvent {
  type: 'action_state_changed'
}

// ---------------------------------------------------------------------------
// Spike types
// ---------------------------------------------------------------------------

export type SpikeVerdict = 'ADOPT' | 'ADAPT' | 'REJECT'

export interface SpikeSummary {
  slug: string
  title: string
  verdict: SpikeVerdict
  date: string
  the_question: string
  ponder_slug?: string
  knowledge_slug?: string
}

export type SpikeDetail = SpikeSummary

// ---------------------------------------------------------------------------
// Hub types (hub mode — multi-project registry)
// ---------------------------------------------------------------------------

export type HubProjectStatus = 'online' | 'stale' | 'offline'

export interface HubProjectEntry {
  name: string
  url: string
  active_milestone: string | null
  feature_count: number | null
  agent_running: boolean | null
  last_seen: string  // ISO-8601
  status: HubProjectStatus
}

export interface HubSseEvent {
  type: 'project_updated' | 'project_removed' | 'fleet_updated' | 'fleet_provisioned' | 'fleet_agent_status'
  project?: HubProjectEntry
  instance?: FleetInstance
  url?: string
  agent_summary?: FleetAgentSummary
}

// ---------------------------------------------------------------------------
// Fleet control plane types (hub mode — fleet management)
// ---------------------------------------------------------------------------

export interface FleetInstance {
  slug: string
  namespace: string
  url: string
  deployment_status: 'running' | 'pending' | 'failed' | 'unknown'
  pod_healthy: boolean
  active_milestone: string | null
  feature_count: number | null
  agent_running: boolean | null
  created_at: string | null
}

export interface AvailableRepo {
  slug: string
  full_name: string
  description: string | null
  clone_url: string
  created_at: string | null
  archived: boolean
  can_provision: boolean
}

export interface FleetAgentSummary {
  total_active_runs: number
  projects_with_agents: number
}

export interface CreateRepoResponse {
  repo_slug: string
  push_url: string
  gitea_url: string
  provision_triggered: boolean
}

// ---------------------------------------------------------------------------
// Webhook payload inspector types
// ---------------------------------------------------------------------------

export interface WebhookPayloadItem {
  id: string
  route_id: string
  received_at: string
  headers: Record<string, string>
  body: string
  content_type: string | null
  status: 'pending' | 'delivered' | 'failed'
}

// ---------------------------------------------------------------------------
// Subagent exchange types
// ---------------------------------------------------------------------------

export interface PairedSubagentExchange {
  id: string
  description: string | null
  status: string
  isComplete: boolean
  summary: string | null
  lastToolName: string | null
  totalTokens: number | null
  durationMs: number | null
}
