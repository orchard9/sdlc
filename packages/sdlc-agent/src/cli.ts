#!/usr/bin/env node
import { Command } from "commander";
import { runFeature } from "./runner.js";
import { SdlcClient } from "./sdlc-client.js";

const program = new Command();

program
  .name("sdlc-agent")
  .description("Autonomous SDLC directive consumer powered by Claude Agent SDK")
  .version("0.1.0");

program
  .command("run <slug>")
  .description("Run a feature to completion or the next human gate")
  .option("--model <model>", "Claude model to use", "claude-sonnet-4-6")
  .option("--max-turns <n>", "Max agent turns per action", "30")
  .option("--cwd <path>", "Project root directory", process.cwd())
  .option("--sdlc-bin <path>", "Path to sdlc binary", "sdlc")
  .action(
    async (
      slug: string,
      opts: { model: string; maxTurns: string; cwd: string; sdlcBin: string }
    ) => {
      const result = await runFeature(slug, {
        model: opts.model,
        maxTurns: parseInt(opts.maxTurns, 10),
        cwd: opts.cwd,
        sdlcBin: opts.sdlcBin,
        onDirective: (d) => {
          console.log(`\n→ ${d.action}: ${d.message.split("\n")[0]}`);
        },
      });

      console.log(`\n[sdlc-agent] Done.`);
      console.log(`  Feature:   ${result.feature}`);
      console.log(`  Phase:     ${result.finalPhase}`);
      console.log(`  Actions:   ${result.actionsCompleted}`);
      console.log(`  Stopped:   ${result.stoppedAt}`);

      if (result.stoppedAt === "human_gate") {
        console.log(`\n  Resume after approval with:`);
        console.log(`  sdlc-agent run ${result.feature}`);
      }

      if (result.error) {
        console.error(`\n  Error: ${result.error.message}`);
        process.exit(1);
      }
    }
  );

program
  .command("run-all")
  .description("Run all features that have pending autonomous actions")
  .option("--model <model>", "Claude model to use", "claude-sonnet-4-6")
  .option("--max-turns <n>", "Max agent turns per action", "30")
  .option("--cwd <path>", "Project root directory", process.cwd())
  .option("--sdlc-bin <path>", "Path to sdlc binary", "sdlc")
  .action(
    async (opts: {
      model: string;
      maxTurns: string;
      cwd: string;
      sdlcBin: string;
    }) => {
      const client = new SdlcClient({ cwd: opts.cwd, bin: opts.sdlcBin });

      let features: string[];
      try {
        const all = await client.listFeatures();
        features = all
          .filter((f) => f.phase !== "released" && f.phase !== "merge")
          .map((f) => f.slug);
      } catch (err) {
        console.error("Failed to list features:", err);
        process.exit(1);
        return;
      }

      if (features.length === 0) {
        console.log("No features to run.");
        return;
      }

      console.log(`Running ${features.length} feature(s): ${features.join(", ")}`);

      const results = [];
      for (const slug of features) {
        console.log(`\n=== Feature: ${slug} ===`);
        const result = await runFeature(slug, {
          model: opts.model,
          maxTurns: parseInt(opts.maxTurns, 10),
          cwd: opts.cwd,
          sdlcBin: opts.sdlcBin,
        });
        results.push(result);
      }

      // Summary
      console.log("\n=== Summary ===");
      for (const r of results) {
        const icon =
          r.stoppedAt === "done" ? "✓" : r.stoppedAt === "human_gate" ? "⏸" : "✗";
        console.log(
          `  ${icon} ${r.feature}: ${r.stoppedAt} (${r.actionsCompleted} actions)`
        );
      }
    }
  );

program
  .command("plan <slug>")
  .description("Show what the agent would do next (dry run)")
  .option("--cwd <path>", "Project root directory", process.cwd())
  .option("--sdlc-bin <path>", "Path to sdlc binary", "sdlc")
  .action(async (slug: string, opts: { cwd: string; sdlcBin: string }) => {
    const { agentForAction, isHumanGateAction } = await import("./agents/index.js");
    const client = new SdlcClient({ cwd: opts.cwd, bin: opts.sdlcBin });

    const directive = await client.getDirective(slug);
    const agent = agentForAction(directive.action);
    const isHuman = isHumanGateAction(directive.action);

    console.log(`Feature:  ${directive.feature} (${directive.current_phase})`);
    console.log(`Action:   ${directive.action}`);
    console.log(`Message:  ${directive.message}`);
    console.log(`Heavy:    ${directive.is_heavy ? "yes" : "no"}`);
    console.log(
      `Agent:    ${agent ? agent.model : isHuman ? "HUMAN GATE" : "none (unhandled)"}`
    );
    if (directive.output_path) {
      console.log(`Output:   ${directive.output_path}`);
    }
    if (agent) {
      console.log(`Tools:    ${agent.tools.join(", ")}`);
    }
    if (directive.gates && directive.gates.length > 0) {
      console.log(`Gates:    ${directive.gates.map((g) => `${g.name}(${g.type})`).join(", ")}`);
    }
  });

program.parseAsync(process.argv).catch((err: unknown) => {
  console.error(err);
  process.exit(1);
});
