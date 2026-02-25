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

export type ArtifactStatus = 'missing' | 'draft' | 'approved' | 'rejected' | 'needs_fix' | 'passed' | 'failed'
export type TaskStatus = 'pending' | 'in_progress' | 'completed' | 'blocked'
export type MilestoneStatus = 'active' | 'complete' | 'cancelled'

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
  status: MilestoneStatus
  features: string[]
  created_at: string
}

export interface MilestoneDetail {
  slug: string
  title: string
  description: string | null
  status: MilestoneStatus
  features: string[]
  created_at: string
  updated_at: string
  completed_at: string | null
  cancelled_at: string | null
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
  active_work: ActiveWork[]
  blocked: BlockedItem[]
  features: FeatureSummary[]
  milestones: MilestoneSummary[]
  last_updated: string
}

export interface ActiveWork {
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
