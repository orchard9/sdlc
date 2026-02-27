import { tool, createSdkMcpServer } from "@anthropic-ai/claude-agent-sdk";
import type { SdlcClient } from "../sdlc-client.js";
import type { GateDefinition } from "../types.js";
import { getDirectiveSchema, makeGetDirectiveTool } from "./get-directive.js";
import { writeArtifactSchema, makeWriteArtifactTool } from "./write-artifact.js";
import { approveArtifactSchema, makeApproveArtifactTool } from "./approve.js";
import { rejectArtifactSchema, makeRejectArtifactTool } from "./reject.js";
import { addTaskSchema, makeAddTaskTool } from "./add-task.js";
import { completeTaskSchema, makeCompleteTaskTool } from "./complete-task.js";
import { addCommentSchema, makeAddCommentTool } from "./add-comment.js";

export const SDLC_SERVER_NAME = "sdlc";

export const SDLC_TOOLS = [
  "mcp__sdlc__sdlc_get_directive",
  "mcp__sdlc__sdlc_write_artifact",
  "mcp__sdlc__sdlc_approve_artifact",
  "mcp__sdlc__sdlc_reject_artifact",
  "mcp__sdlc__sdlc_add_task",
  "mcp__sdlc__sdlc_complete_task",
  "mcp__sdlc__sdlc_add_comment",
] as const;

export type SdlcMcpServerOptions = {
  /** Gates from the current directive to run before approving artifacts. */
  gates?: GateDefinition[];
  /** Working directory for running gate commands. */
  cwd: string;
};

export function createSdlcMcpServer(client: SdlcClient, opts: SdlcMcpServerOptions) {
  const approveCtx = { gates: opts.gates ?? [], cwd: opts.cwd };

  return createSdkMcpServer({
    name: SDLC_SERVER_NAME,
    version: "0.1.0",
    tools: [
      tool(
        "sdlc_get_directive",
        "Get the current SDLC directive (next action) for a feature. Call this to understand what to do next.",
        getDirectiveSchema,
        makeGetDirectiveTool(client)
      ),
      tool(
        "sdlc_write_artifact",
        "Write content to an SDLC artifact file (spec, design, tasks, qa_plan, review, audit, qa_results). Always write the complete artifact content.",
        writeArtifactSchema,
        makeWriteArtifactTool(client)
      ),
      tool(
        "sdlc_approve_artifact",
        "Approve an SDLC artifact to advance the feature phase. Runs any configured auto shell gates first — if they fail, fix the issues and try again.",
        approveArtifactSchema,
        makeApproveArtifactTool(client, approveCtx)
      ),
      tool(
        "sdlc_reject_artifact",
        "Reject an SDLC artifact with a reason explaining what needs to be fixed.",
        rejectArtifactSchema,
        makeRejectArtifactTool(client)
      ),
      tool(
        "sdlc_add_task",
        "Add an implementation task to the feature. Use when creating the tasks artifact to register individual work items.",
        addTaskSchema,
        makeAddTaskTool(client)
      ),
      tool(
        "sdlc_complete_task",
        "Mark an implementation task as complete. Use after implementing a task and verifying it works.",
        completeTaskSchema,
        makeCompleteTaskTool(client)
      ),
      tool(
        "sdlc_add_comment",
        "Add a comment, question, or blocker to the feature. Use to flag issues, questions, or blockers for human review.",
        addCommentSchema,
        makeAddCommentTool(client)
      ),
    ],
  });
}
