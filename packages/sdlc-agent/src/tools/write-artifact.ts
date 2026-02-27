import { z } from "zod";
import { join, resolve } from "node:path";
import type { SdlcClient } from "../sdlc-client.js";
import type { ArtifactType } from "../types.js";

export const writeArtifactSchema = {
  slug: z.string().describe("Feature slug"),
  artifact_type: z
    .enum(["spec", "design", "tasks", "qa_plan", "review", "audit", "qa_results"])
    .describe("Type of artifact to write"),
  content: z.string().describe("Full markdown content to write to the artifact file"),
};

export function makeWriteArtifactTool(client: SdlcClient) {
  return async (args: {
    slug: string;
    artifact_type: ArtifactType;
    content: string;
  }) => {
    try {
      const artifactTypeToFile: Record<ArtifactType, string> = {
        spec: "spec.md",
        design: "design.md",
        tasks: "tasks.md",
        qa_plan: "qa-plan.md",
        review: "review.md",
        audit: "audit.md",
        qa_results: "qa-results.md",
      };

      const filename = artifactTypeToFile[args.artifact_type];
      const outputPath = `.sdlc/features/${args.slug}/${filename}`;
      const absolutePath = resolve(client.cwd, outputPath);

      // Verify path stays within project root (guard against path traversal)
      const projectRoot = resolve(client.cwd);
      if (!absolutePath.startsWith(projectRoot)) {
        throw new Error(`Path traversal detected: ${outputPath}`);
      }

      client.writeArtifactFile(outputPath, args.content);

      // Mark artifact as draft so sdlc tracks it
      await client.draftArtifact(args.slug, args.artifact_type);

      return {
        content: [
          {
            type: "text" as const,
            text: `Written and marked as draft: ${outputPath}`,
          },
        ],
      };
    } catch (err) {
      return {
        content: [
          {
            type: "text" as const,
            text: `Error writing artifact: ${err instanceof Error ? err.message : String(err)}`,
          },
        ],
      };
    }
  };
}
