import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { writeFileSync, mkdirSync } from "node:fs";
import { dirname } from "node:path";
import type { SdlcDirective, ArtifactType } from "./types.js";

const execFileAsync = promisify(execFile);

export type SdlcClientOptions = {
  cwd?: string;
  bin?: string;
};

export type AddTaskResult = {
  slug: string;
  task_id: string;
  title: string;
};

export class SdlcClient {
  readonly cwd: string;
  readonly bin: string;

  constructor(opts: SdlcClientOptions = {}) {
    this.cwd = opts.cwd ?? process.cwd();
    this.bin = opts.bin ?? "sdlc";
  }

  async getDirective(slug: string): Promise<SdlcDirective> {
    const { stdout } = await this.exec(["next", "--for", slug, "--json"]);
    return JSON.parse(stdout.trim()) as SdlcDirective;
  }

  async approveArtifact(slug: string, artifactType: ArtifactType): Promise<void> {
    await this.exec(["artifact", "approve", slug, artifactType]);
  }

  async rejectArtifact(
    slug: string,
    artifactType: ArtifactType,
    reason: string
  ): Promise<void> {
    await this.exec(["artifact", "reject", slug, artifactType, "--reason", reason]);
  }

  async draftArtifact(slug: string, artifactType: ArtifactType): Promise<void> {
    await this.exec(["artifact", "draft", slug, artifactType]);
  }

  // title is passed as a single positional arg: sdlc task add <slug> <title>
  async addTask(slug: string, title: string): Promise<AddTaskResult> {
    const { stdout } = await this.exec(["task", "add", "--json", slug, title]);
    return JSON.parse(stdout.trim()) as AddTaskResult;
  }

  async completeTask(slug: string, taskId: string): Promise<void> {
    await this.exec(["task", "complete", slug, taskId]);
  }

  // body is a positional arg: sdlc comment create <slug> <body> [--flag <flag>]
  async addComment(slug: string, body: string, flag?: string): Promise<void> {
    const args: string[] = ["comment", "create", slug, body];
    if (flag) args.push("--flag", flag);
    await this.exec(args);
  }

  async transitionPhase(slug: string, targetPhase: string): Promise<void> {
    await this.exec(["feature", "transition", slug, targetPhase]);
  }

  async listFeatures(): Promise<Array<{ slug: string; phase: string }>> {
    const { stdout } = await this.exec(["feature", "list", "--json"]);
    return JSON.parse(stdout.trim()) as Array<{ slug: string; phase: string }>;
  }

  writeArtifactFile(outputPath: string, content: string): void {
    const absolutePath = outputPath.startsWith("/")
      ? outputPath
      : `${this.cwd}/${outputPath}`;
    mkdirSync(dirname(absolutePath), { recursive: true });
    writeFileSync(absolutePath, content, "utf8");
  }

  private async exec(args: string[]): Promise<{ stdout: string; stderr: string }> {
    return execFileAsync(this.bin, args, { cwd: this.cwd });
  }
}
