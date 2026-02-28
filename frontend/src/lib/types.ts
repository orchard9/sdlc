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

export interface ProjectState {
  project: string
  active_features: string[]
  active_directives: ActiveDirective[]
  blocked: BlockedItem[]
  features: FeatureSummary[]
  milestones: MilestoneSummary[]
  escalations: EscalationSummary[]
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
// Agent run events (SSE from /api/run/:slug/events)
// ---------------------------------------------------------------------------

export interface AgentEvent {
  type: 'init' | 'assistant' | 'tool_progress' | 'tool_summary' | 'result' | 'error' | 'status' | 'system' | 'user' | 'stream_event' | 'auth_status' | 'not_running'
  // init
  model?: string
  tools_count?: number
  mcp_servers?: string[]
  // assistant
  text?: string
  tools?: { name: string; input: unknown }[]
  // tool_progress
  tool?: string
  elapsed_seconds?: number
  // tool_summary
  summary?: string
  // result
  is_error?: boolean
  cost_usd?: number
  turns?: number
  // error / status
  message?: string
  status?: string
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

export interface FeedbackNote {
  id: string
  content: string
  created_at: string
}

// ---------------------------------------------------------------------------
// Ponder / Roadmap types
// ---------------------------------------------------------------------------

export type PonderStatus = 'exploring' | 'converging' | 'committed' | 'parked'

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
}

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
  type: 'vision_align_completed' | 'architecture_align_completed'
}

// ---------------------------------------------------------------------------
// Tool Suite types
// ---------------------------------------------------------------------------

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
  /** Only present in the POST (start) response; null on GET. */
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
