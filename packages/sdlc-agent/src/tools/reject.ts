import { z } from "zod";
import type { SdlcClient } from "../sdlc-client.js";
import type { ArtifactType } from "../types.js";

export const rejectArtifactSchema = {
  slug: z.string().describe("Feature slug"),
  artifact_type: z
    .enum(["spec", "design", "tasks", "qa_plan", "review", "audit", "qa_results"])
    .describe("Type of artifact to reject"),
  reason: z.string().describe("Reason for rejection — what needs to be fixed"),
};

export function makeRejectArtifactTool(client: SdlcClient) {
  return async (args: {
    slug: string;
    artifact_type: ArtifactType;
    reason: string;
  }) => {
    try {
      await client.rejectArtifact(args.slug, args.artifact_type, args.reason);

      return {
        content: [
          {
            type: "text" as const,
            text: `Rejected: ${args.slug}/${args.artifact_type}\nReason: ${args.reason}`,
          },
        ],
      };
    } catch (err) {
      return {
        content: [
          {
            type: "text" as const,
            text: `Error rejecting artifact: ${err instanceof Error ? err.message : String(err)}`,
          },
        ],
      };
    }
  };
}
