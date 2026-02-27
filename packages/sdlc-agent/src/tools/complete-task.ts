import { z } from "zod";
import type { SdlcClient } from "../sdlc-client.js";

export const completeTaskSchema = {
  slug: z.string().describe("Feature slug"),
  task_id: z.string().describe("Task ID to mark as complete"),
  notes: z.string().optional().describe("Optional completion notes"),
};

export function makeCompleteTaskTool(client: SdlcClient) {
  return async (args: { slug: string; task_id: string; notes: string | undefined }) => {
    try {
      await client.completeTask(args.slug, args.task_id);

      return {
        content: [
          {
            type: "text" as const,
            text: [
              `Completed task: ${args.task_id}`,
              args.notes ? `Notes: ${args.notes}` : "",
            ]
              .filter(Boolean)
              .join("\n"),
          },
        ],
      };
    } catch (err) {
      return {
        content: [
          {
            type: "text" as const,
            text: `Error completing task: ${err instanceof Error ? err.message : String(err)}`,
          },
        ],
      };
    }
  };
}
