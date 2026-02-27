import { z } from "zod";
import type { SdlcClient } from "../sdlc-client.js";

export const addCommentSchema = {
  slug: z.string().describe("Feature slug"),
  body: z.string().describe("Comment body — the question, note, or blocker description"),
  flag_type: z
    .enum(["blocker", "question", "decision", "fyi"])
    .optional()
    .describe("Flag type: blocker (blocks progress), question (needs answer), decision (pending decision), fyi (informational)"),
};

export function makeAddCommentTool(client: SdlcClient) {
  return async (args: { slug: string; body: string; flag_type: string | undefined }) => {
    try {
      await client.addComment(args.slug, args.body, args.flag_type);
      return {
        content: [
          {
            type: "text" as const,
            text: `Comment added to ${args.slug}${args.flag_type ? ` [${args.flag_type}]` : ""}: ${args.body}`,
          },
        ],
      };
    } catch (err) {
      return {
        content: [
          {
            type: "text" as const,
            text: `Error adding comment: ${err instanceof Error ? err.message : String(err)}`,
          },
        ],
      };
    }
  };
}
