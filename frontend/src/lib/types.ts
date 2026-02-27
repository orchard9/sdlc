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
  | 'create_qa_plan'
  | 'implement_task'
  | 'fix_review_issues'
  | 'create_review'
  | 'approve_review'
  | 'create_audit'
  | 'run_qa'
  | 'approve_merge'
  | 'merge'
  | 'archive'
  | 'unblock_dependency'
  | 'wait_for_approval'
  | 'done'

export type ArtifactStatus = 'missing' | 'draft' | 'approved' | 'rejected' | 'needs_fix' | 'passed' | 'failed' | 'waived'
export type TaskStatus = 'pending' | 'in_progress' | 'completed' | 'blocked'
export type MilestoneStatus = 'active' | 'released' | 'skipped'

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

export interface QuerySearchResult {
  slug: string
  title: string
  phase: string
  score: number
}

export interface QuerySearchResponse {
  results: QuerySearchResult[]
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

export interface ProjectState {
  project: string
  active_features: string[]
  active_directives: ActiveDirective[]
  blocked: BlockedItem[]
  features: FeatureSummary[]
  milestones: MilestoneSummary[]
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
