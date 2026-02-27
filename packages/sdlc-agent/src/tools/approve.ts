import { z } from "zod";
import type { SdlcClient } from "../sdlc-client.js";
import type { ArtifactType, GateDefinition } from "../types.js";
import { runGates, allGatesPassed, formatGateResults } from "../gates.js";

export const approveArtifactSchema = {
  slug: z.string().describe("Feature slug"),
  artifact_type: z
    .enum(["spec", "design", "tasks", "qa_plan", "review", "audit", "qa_results"])
    .describe("Type of artifact to approve"),
  notes: z
    .string()
    .optional()
    .describe("Optional notes about the approval or what was verified"),
};

export type ApproveToolContext = {
  gates?: GateDefinition[];
  cwd: string;
};

export function makeApproveArtifactTool(client: SdlcClient, ctx: ApproveToolContext) {
  return async (args: {
    slug: string;
    artifact_type: ArtifactType;
    notes: string | undefined;
  }) => {
    // Run auto shell gates before approving
    const gates = ctx.gates ?? [];
    const relevantGates = gates.filter((g) => g.auto && g.type === "shell");

    if (relevantGates.length > 0) {
      const gateResults = await runGates(relevantGates, ctx.cwd);
      const passed = allGatesPassed(gateResults);
      const summary = formatGateResults(gateResults);

      if (!passed) {
        return {
          content: [
            {
              type: "text" as const,
              text: [
                `Gate checks FAILED for ${args.slug}/${args.artifact_type}.`,
                `Fix the failures before approving.`,
                ``,
                `Gate results:`,
                summary,
              ].join("\n"),
            },
          ],
        };
      }

      // All gates passed — proceed to approve
      try {
        await client.approveArtifact(args.slug, args.artifact_type);
        return {
          content: [
            {
              type: "text" as const,
              text: [
                `Approved: ${args.slug}/${args.artifact_type}`,
                args.notes ? `Notes: ${args.notes}` : "",
                ``,
                `Gate results (all passed):`,
                summary,
                ``,
                `Use sdlc_get_directive to get the next action.`,
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
              text: `Error approving artifact: ${err instanceof Error ? err.message : String(err)}`,
            },
          ],
        };
      }
    }

    // No gates — approve directly
    try {
      await client.approveArtifact(args.slug, args.artifact_type);
      return {
        content: [
          {
            type: "text" as const,
            text: [
              `Approved: ${args.slug}/${args.artifact_type}`,
              args.notes ? `Notes: ${args.notes}` : "",
              `Use sdlc_get_directive to get the next action.`,
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
            text: `Error approving artifact: ${err instanceof Error ? err.message : String(err)}`,
          },
        ],
      };
    }
  };
}
