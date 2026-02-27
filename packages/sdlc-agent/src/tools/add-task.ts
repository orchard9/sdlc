import { z } from "zod";
import type { SdlcClient } from "../sdlc-client.js";

export const addTaskSchema = {
  slug: z.string().describe("Feature slug"),
  title: z.string().describe("Task title — short imperative phrase (e.g. 'Implement auth middleware')"),
};

export function makeAddTaskTool(client: SdlcClient) {
  return async (args: { slug: string; title: string }) => {
    try {
      const result = await client.addTask(args.slug, args.title);
      return {
        content: [
          {
            type: "text" as const,
            text: `Task added: ${result.task_id} — ${result.title}`,
          },
        ],
      };
    } catch (err) {
      return {
        content: [
          {
            type: "text" as const,
            text: `Error adding task: ${err instanceof Error ? err.message : String(err)}`,
          },
        ],
      };
    }
  };
}
