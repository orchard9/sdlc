//! `sdlc tool` subcommands — the SDLC Tool Suite CLI.
//!
//! Tools are TypeScript scripts installed in `.sdlc/tools/<name>/tool.ts`.
//! They speak a JSON stdin/stdout protocol and can be run with any of:
//!   bun run tool.ts --run
//!   deno run --allow-read tool.ts --run
//!   npx tsx tool.ts --run

use anyhow::Context;
use clap::Subcommand;
use serde::Deserialize;
use std::path::Path;

use sdlc_core::paths;

#[derive(Subcommand)]
pub enum ToolCommand {
    /// List all installed SDLC tools
    List,

    /// Run a tool (reads JSON from stdin or --input flag)
    Run {
        /// Tool name (e.g. ama, quality-check)
        name: String,
        /// Raw JSON input as a string (alternative to piping stdin)
        #[arg(long, value_name = "JSON")]
        input: Option<String>,
        /// Convenience: question for the 'ama' tool
        #[arg(long)]
        question: Option<String>,
        /// Convenience: scope filter for the 'quality-check' tool
        #[arg(long)]
        scope: Option<String>,
        /// Run setup mode instead of run mode
        #[arg(long)]
        setup: bool,
    },

    /// Regenerate .sdlc/tools/tools.md from installed tool metadata
    Sync,

    /// Show detailed metadata for a tool
    Info {
        /// Tool name
        name: String,
    },

    /// Scaffold a new custom tool skeleton
    Scaffold {
        /// Tool name (slug, e.g. git-activity)
        name: String,
        /// One-sentence description of what this tool does
        description: String,
    },
}

pub fn run(cmd: ToolCommand, root: &Path) -> anyhow::Result<()> {
    match cmd {
        ToolCommand::List => list_tools(root),
        ToolCommand::Run {
            name,
            input,
            question,
            scope,
            setup,
        } => {
            let mode = if setup { "--setup" } else { "--run" };
            let json_input = build_input_json(input, question, scope);
            run_tool_cmd(root, &name, mode, json_input.as_deref())
        }
        ToolCommand::Sync => sync_manifest(root),
        ToolCommand::Info { name } => tool_info(root, &name),
        ToolCommand::Scaffold { name, description } => scaffold_tool(root, &name, &description),
    }
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ToolConfigMin {
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    version: Option<String>,
}

fn list_tools(root: &Path) -> anyhow::Result<()> {
    let tools_dir = paths::tools_dir(root);

    if !tools_dir.exists() {
        println!("No tools installed.");
        println!("Run 'sdlc init' or 'sdlc update' to install core tools.");
        return Ok(());
    }

    let mut tools: Vec<(String, String, String)> = Vec::new();

    for entry in std::fs::read_dir(&tools_dir).context("failed to read tools dir")? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();

        // Skip _shared and hidden dirs
        if name.starts_with('_') || name.starts_with('.') {
            continue;
        }

        if !entry.file_type()?.is_dir() {
            continue;
        }

        let script = entry.path().join("tool.ts");
        if !script.exists() {
            continue;
        }

        // Read description from config.yaml if present
        let (desc, version) = read_tool_config_min(&entry.path());
        tools.push((name, desc, version));
    }

    if tools.is_empty() {
        println!("No tools installed.");
        println!("Run 'sdlc init' or 'sdlc update' to install core tools.");
        return Ok(());
    }

    println!("Installed SDLC tools:\n");
    for (name, desc, version) in &tools {
        let ver = if version.is_empty() {
            String::new()
        } else {
            format!(" (v{version})")
        };
        println!("  {name}{ver}");
        if !desc.is_empty() {
            println!("    {desc}");
        }
    }
    println!();
    println!("Run: sdlc tool run <name> [--question \"...\"]");
    println!("Docs: .sdlc/tools/tools.md");

    Ok(())
}

fn read_tool_config_min(tool_dir: &Path) -> (String, String) {
    let config_path = tool_dir.join("config.yaml");
    if !config_path.exists() {
        return (String::new(), String::new());
    }
    let Ok(raw) = std::fs::read_to_string(&config_path) else {
        return (String::new(), String::new());
    };
    let Ok(cfg) = serde_yaml::from_str::<ToolConfigMin>(&raw) else {
        return (String::new(), String::new());
    };
    (
        cfg.description.unwrap_or_default(),
        cfg.version.unwrap_or_default(),
    )
}

// ---------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------

fn build_input_json(
    input: Option<String>,
    question: Option<String>,
    scope: Option<String>,
) -> Option<String> {
    if let Some(j) = input {
        return Some(j);
    }
    // Build a JSON object from convenience flags
    let mut fields: Vec<String> = Vec::new();
    if let Some(q) = question {
        fields.push(format!(r#""question": {}"#, serde_json::json!(q)));
    }
    if let Some(s) = scope {
        fields.push(format!(r#""scope": {}"#, serde_json::json!(s)));
    }
    if fields.is_empty() {
        // Empty object — tool handles missing optional fields gracefully
        Some("{}".into())
    } else {
        Some(format!("{{{}}}", fields.join(", ")))
    }
}

fn run_tool_cmd(
    root: &Path,
    name: &str,
    mode: &str,
    input_json: Option<&str>,
) -> anyhow::Result<()> {
    let script = paths::tool_script(root, name);

    if !script.exists() {
        anyhow::bail!(
            "Tool '{}' not found at {}\nRun 'sdlc tool list' to see installed tools.",
            name,
            script.display()
        );
    }

    let output = sdlc_core::tool_runner::run_tool(&script, mode, input_json, root)
        .with_context(|| format!("failed to run tool '{name}'"))?;

    // Pretty-print if JSON, otherwise raw
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&output) {
        println!("{}", serde_json::to_string_pretty(&val)?);
    } else {
        print!("{output}");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Sync
// ---------------------------------------------------------------------------

fn sync_manifest(root: &Path) -> anyhow::Result<()> {
    let tools_dir = paths::tools_dir(root);
    if !tools_dir.exists() {
        println!("No tools directory found. Run 'sdlc init' first.");
        return Ok(());
    }

    let mut entries: Vec<ToolEntry> = Vec::new();

    for entry in std::fs::read_dir(&tools_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('_') || name.starts_with('.') {
            continue;
        }
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let script = entry.path().join("tool.ts");
        if !script.exists() {
            continue;
        }

        match sdlc_core::tool_runner::run_tool(&script, "--meta", None, root) {
            Ok(json) => {
                if let Ok(meta) = serde_json::from_str::<ToolMeta>(&json) {
                    entries.push(ToolEntry {
                        name: meta.name,
                        display_name: meta.display_name,
                        description: meta.description,
                        version: meta.version,
                        requires_setup: meta.requires_setup,
                        setup_description: meta.setup_description,
                        input_schema: meta.input_schema,
                        output_schema: meta.output_schema,
                    });
                } else {
                    eprintln!("warning: could not parse metadata for tool '{name}'");
                }
            }
            Err(e) => {
                eprintln!("warning: could not get metadata for tool '{name}': {e}");
            }
        }
    }

    let manifest = format_tools_manifest(&entries);
    let manifest_path = paths::tools_manifest_path(root);
    sdlc_core::io::atomic_write(&manifest_path, manifest.as_bytes())
        .context("failed to write tools.md")?;

    println!("Updated .sdlc/tools/tools.md ({} tools)", entries.len());
    Ok(())
}

// ---------------------------------------------------------------------------
// Info
// ---------------------------------------------------------------------------

fn tool_info(root: &Path, name: &str) -> anyhow::Result<()> {
    let script = paths::tool_script(root, name);
    if !script.exists() {
        anyhow::bail!(
            "Tool '{}' not found. Run 'sdlc tool list' to see installed tools.",
            name
        );
    }

    let output = sdlc_core::tool_runner::run_tool(&script, "--meta", None, root)
        .with_context(|| format!("failed to get metadata for tool '{name}'"))?;

    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&output) {
        println!("{}", serde_json::to_string_pretty(&val)?);
    } else {
        println!("{output}");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Scaffold
// ---------------------------------------------------------------------------

fn scaffold_tool(root: &Path, name: &str, description: &str) -> anyhow::Result<()> {
    sdlc_core::paths::validate_slug(name).map_err(|e| anyhow::anyhow!("invalid tool name: {e}"))?;

    let tool_dir = paths::tool_dir(root, name);
    if tool_dir.exists() {
        anyhow::bail!("Tool '{}' already exists at {}", name, tool_dir.display());
    }

    sdlc_core::io::ensure_dir(&tool_dir).context("failed to create tool directory")?;

    // Scaffold tool.ts
    let script_content = build_scaffold_ts(name, description);
    let script_path = paths::tool_script(root, name);
    sdlc_core::io::atomic_write(&script_path, script_content.as_bytes())
        .context("failed to write tool.ts")?;

    // Scaffold config.yaml
    let config_content = format!(
        "name: {name}\nversion: 0.1.0\ndescription: {description:?}\n# Add tool-specific config here\n"
    );
    let config_path = paths::tool_config(root, name);
    sdlc_core::io::atomic_write(&config_path, config_content.as_bytes())
        .context("failed to write config.yaml")?;

    // Scaffold README.md
    let readme_content = build_scaffold_readme(name, description);
    let readme_path = paths::tool_readme(root, name);
    sdlc_core::io::atomic_write(&readme_path, readme_content.as_bytes())
        .context("failed to write README.md")?;

    println!("Scaffolded tool '{name}':");
    println!("  .sdlc/tools/{name}/tool.ts");
    println!("  .sdlc/tools/{name}/config.yaml");
    println!("  .sdlc/tools/{name}/README.md");
    println!();
    println!("Next steps:");
    println!("  1. Fill in the run() function in .sdlc/tools/{name}/tool.ts");
    println!("  2. Test: bun run .sdlc/tools/{name}/tool.ts --meta | jq .");
    println!("  3. Test: echo '{{}}' | bun run .sdlc/tools/{name}/tool.ts --run | jq .");
    println!("  4. Register: sdlc tool sync");

    Ok(())
}

fn build_scaffold_ts(name: &str, description: &str) -> String {
    let display_name = to_display_name(name);
    let underline = "=".repeat(display_name.len());
    format!(
        r#"/**
 * {display_name}
 * {underline}
 * {description}
 *
 * WHAT IT DOES
 * ------------
 * --run:   Reads JSON from stdin: {{ ... }}
 *          Returns JSON to stdout: {{ ... }}
 * --meta:  Writes ToolMeta JSON to stdout. Used by `sdlc tool sync`.
 *
 * WHAT IT READS
 * -------------
 * - .sdlc/tools/{name}/config.yaml
 *
 * WHAT IT WRITES
 * --------------
 * - STDERR only (structured log lines via _shared/log.ts)
 * - STDOUT: JSON only (ToolResult shape)
 *
 * EXTENDING
 * ---------
 * TODO: describe the main extension point
 */

import type {{ ToolMeta, ToolResult }} from '../_shared/types.ts'
import {{ makeLogger }} from '../_shared/log.ts'
import {{ loadToolConfig }} from '../_shared/config.ts'
import {{ getArgs, readStdin, exit }} from '../_shared/runtime.ts'

const log = makeLogger('{name}')

export const meta: ToolMeta = {{
  name: '{name}',
  display_name: '{display_name}',
  description: '{description}',
  version: '0.1.0',
  requires_setup: false,
  input_schema: {{
    type: 'object',
    required: [],        // TODO: add required fields
    properties: {{}}      // TODO: define input fields
  }},
  output_schema: {{
    type: 'object',
    properties: {{}}      // TODO: define output fields
  }}
}}

interface Input {{
  // TODO: define input fields
}}

interface Output {{
  // TODO: define output fields
}}

export async function run(input: Input): Promise<ToolResult<Output>> {{
  log.info('starting')
  try {{
    const root = process.env.SDLC_ROOT ?? process.cwd()
    const config = loadToolConfig(root, '{name}', {{}})

    // TODO: implement tool logic here

    const result: Output = {{}}
    log.info('done')
    return {{ ok: true, data: result }}
  }} catch (e) {{
    log.error(String(e))
    return {{ ok: false, error: String(e) }}
  }}
}}

// ---------------------------------------------------------------------------
// CLI entrypoint — handles --meta, --run
// ---------------------------------------------------------------------------

const mode = getArgs()[0] ?? '--run'

if (mode === '--meta') {{
  console.log(JSON.stringify(meta))
  exit(0)
}} else if (mode === '--run') {{
  readStdin()
    .then(raw => run(JSON.parse(raw || '{{}}')))
    .then(result => {{
      console.log(JSON.stringify(result))
      exit(result.ok ? 0 : 1)
    }})
    .catch(e => {{
      console.log(JSON.stringify({{ ok: false, error: String(e) }}))
      exit(1)
    }})
}}
"#,
        display_name = display_name,
        underline = underline,
        name = name,
        description = description,
    )
}

fn build_scaffold_readme(name: &str, description: &str) -> String {
    let display = to_display_name(name);
    format!(
        "# {display}\n\n{description}\n\n## Usage\n```bash\nsdlc tool run {name} --json '{{}}'\n```\n\n## Configuration\nEdit `.sdlc/tools/{name}/config.yaml`\n\n## How it works\nTODO: describe what the tool does\n"
    )
}

fn to_display_name(slug: &str) -> String {
    slug.split('-')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Manifest formatting
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, serde::Serialize)]
#[allow(dead_code)]
struct ToolMeta {
    name: String,
    display_name: String,
    description: String,
    version: String,
    requires_setup: bool,
    setup_description: Option<String>,
    input_schema: serde_json::Value,
    output_schema: serde_json::Value,
}

#[allow(dead_code)]
struct ToolEntry {
    name: String,
    display_name: String,
    description: String,
    version: String,
    requires_setup: bool,
    setup_description: Option<String>,
    input_schema: serde_json::Value,
    output_schema: serde_json::Value,
}

fn format_tools_manifest(tools: &[ToolEntry]) -> String {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let mut out = format!(
        "# SDLC Tools\n\nProject-specific tools installed by sdlc. Use `sdlc tool run <name>` to invoke.\n\nGenerated: {now}\n\n---\n\n"
    );

    if tools.is_empty() {
        out.push_str("No tools installed. Run `sdlc init` or `sdlc update`.\n");
        return out;
    }

    for tool in tools {
        out.push_str(&format!(
            "## {} — {}\n\n{}\n\n",
            tool.name, tool.display_name, tool.description
        ));

        out.push_str(&format!("**Run:** `sdlc tool run {}`\n", tool.name));

        if tool.requires_setup {
            let setup_desc = tool
                .setup_description
                .as_deref()
                .unwrap_or("Initialize before first use");
            out.push_str(&format!(
                "**Setup required:** Yes — `sdlc tool run {} --setup`  \n_{setup_desc}_\n",
                tool.name
            ));
        }

        out.push_str(&format!(
            "\n**Input:**\n```json\n{}\n```\n",
            serde_json::to_string_pretty(&tool.input_schema).unwrap_or_default()
        ));

        out.push_str(&format!(
            "\n**Output:**\n```json\n{}\n```\n\n---\n\n",
            serde_json::to_string_pretty(&tool.output_schema).unwrap_or_default()
        ));
    }

    out.push_str(
        "## Adding a Custom Tool\n\nRun `sdlc tool scaffold <name> \"<description>\"` to create a new tool skeleton.\nThen implement the `run()` function in `.sdlc/tools/<name>/tool.ts` and run `sdlc tool sync`.\n"
    );

    out
}
