import { z } from "zod";
import type { SdlcClient } from "../sdlc-client.js";

export const getDirectiveSchema = {
  slug: z.string().describe("Feature slug to get the next directive for"),
};

export function makeGetDirectiveTool(client: SdlcClient) {
  return async (args: { slug: string }) => {
    try {
      const directive = await client.getDirective(args.slug);
      return {
        content: [
          {
            type: "text" as const,
            text: JSON.stringify(directive, null, 2),
          },
        ],
      };
    } catch (err) {
      return {
        content: [
          {
            type: "text" as const,
            text: `Error getting directive: ${err instanceof Error ? err.message : String(err)}`,
          },
        ],
      };
    }
  };
}
