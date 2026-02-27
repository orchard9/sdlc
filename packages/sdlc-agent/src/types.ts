export type ActionType =
  | "create_spec"
  | "approve_spec"
  | "create_design"
  | "approve_design"
  | "create_tasks"
  | "create_qa_plan"
  | "implement_task"
  | "fix_review_issues"
  | "create_review"
  | "approve_review"
  | "create_audit"
  | "run_qa"
  | "approve_merge"
  | "merge"
  | "archive"
  | "unblock_dependency"
  | "wait_for_approval"
  | "done";

export type GateType = "shell" | "human" | "step_back";

export type GateDefinition = {
  name: string;
  type: GateType;
  command?: string;
  auto: boolean;
  max_retries?: number;
};

export type SdlcDirective = {
  feature: string;
  title: string;
  current_phase: string;
  action: ActionType;
  message: string;
  next_command: string | null;
  output_path: string | null;
  transition_to: string | null;
  task_id: string | null;
  is_heavy: boolean;
  timeout_minutes: number;
  gates?: GateDefinition[];
};

export type ArtifactType =
  | "spec"
  | "design"
  | "tasks"
  | "qa_plan"
  | "review"
  | "audit"
  | "qa_results";

export type GateResult = {
  name: string;
  type: GateType;
  passed: boolean;
  output?: string;
  error?: string;
};

export type AgentConfig = {
  description: string;
  prompt: string;
  tools: string[];
  model: string;
};

export type AgentOptions = {
  cwd?: string;
  model?: string;
  maxTurns?: number;
  onDirective?: (d: SdlcDirective) => void;
  onMessage?: (m: unknown) => void;
  sdlcBin?: string;
};

export type RunResult = {
  feature: string;
  finalPhase: string;
  actionsCompleted: number;
  stoppedAt: "done" | "human_gate" | "error";
  error?: Error;
};
