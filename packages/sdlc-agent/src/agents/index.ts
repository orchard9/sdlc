import type { ActionType, AgentConfig } from "../types.js";
import { specWriterAgent } from "./spec-writer.js";
import { designerAgent } from "./designer.js";
import { taskPlannerAgent } from "./task-planner.js";
import { implementerAgent } from "./implementer.js";
import { reviewerAgent } from "./reviewer.js";
import { auditorAgent } from "./auditor.js";
import { qaRunnerAgent } from "./qa-runner.js";

const AGENT_MAP: Partial<Record<ActionType, AgentConfig>> = {
  create_spec: specWriterAgent,
  create_design: designerAgent,
  create_tasks: taskPlannerAgent,
  create_qa_plan: taskPlannerAgent,
  implement_task: implementerAgent,
  fix_review_issues: implementerAgent,
  create_review: reviewerAgent,
  create_audit: auditorAgent,
  run_qa: qaRunnerAgent,
};

const HUMAN_GATE_ACTIONS: ActionType[] = [
  "approve_spec",
  "approve_design",
  "approve_review",
  "approve_merge",
  "wait_for_approval",
];

export function agentForAction(action: ActionType): AgentConfig | null {
  return AGENT_MAP[action] ?? null;
}

export function isHumanGateAction(action: ActionType): boolean {
  return HUMAN_GATE_ACTIONS.includes(action);
}

export function isTerminalAction(action: ActionType): boolean {
  return action === "done";
}

export {
  specWriterAgent,
  designerAgent,
  taskPlannerAgent,
  implementerAgent,
  reviewerAgent,
  auditorAgent,
  qaRunnerAgent,
};
