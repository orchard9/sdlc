export type CommandCategory =
  | 'lifecycle'
  | 'planning'
  | 'workspace'
  | 'analysis'
  | 'tooling'
  | 'project'

export interface CommandEntry {
  slug: string
  invocation: string
  description: string
  category: CommandCategory
}

export const CATEGORY_LABELS: Record<CommandCategory, string> = {
  lifecycle: 'Lifecycle',
  planning: 'Planning',
  workspace: 'Workspace',
  analysis: 'Analysis & Quality',
  tooling: 'Tooling',
  project: 'Project Setup',
}

export const CATEGORY_ORDER: CommandCategory[] = [
  'lifecycle',
  'planning',
  'workspace',
  'analysis',
  'tooling',
  'project',
]

export const COMMANDS: CommandEntry[] = [
  // lifecycle
  {
    slug: 'sdlc-next',
    invocation: '/sdlc-next <feature-slug>',
    description: 'Get the next directive for a feature and act on it.',
    category: 'lifecycle',
  },
  {
    slug: 'sdlc-run',
    invocation: '/sdlc-run <feature-slug>',
    description: 'Autonomously drive a feature to completion.',
    category: 'lifecycle',
  },
  {
    slug: 'sdlc-approve',
    invocation: '/sdlc-approve <feature-slug>',
    description: 'Approve the current pending artifact for a feature.',
    category: 'lifecycle',
  },
  {
    slug: 'sdlc-status',
    invocation: '/sdlc-status',
    description: 'Show project and feature status overview.',
    category: 'lifecycle',
  },

  // planning
  {
    slug: 'sdlc-plan',
    invocation: '/sdlc-plan',
    description: 'Distribute a plan into milestones, features, and tasks.',
    category: 'planning',
  },
  {
    slug: 'sdlc-prepare',
    invocation: '/sdlc-prepare <milestone-slug>',
    description: 'Pre-flight a milestone — align features with vision, fix gaps, write wave plan.',
    category: 'planning',
  },
  {
    slug: 'sdlc-run-wave',
    invocation: '/sdlc-run-wave <milestone-slug>',
    description: 'Execute Wave 1 features in parallel, advance to next wave.',
    category: 'planning',
  },
  {
    slug: 'sdlc-pressure-test',
    invocation: '/sdlc-pressure-test <milestone-slug>',
    description: 'Pressure-test a milestone against user perspectives.',
    category: 'planning',
  },
  {
    slug: 'sdlc-milestone-uat',
    invocation: '/sdlc-milestone-uat <milestone-slug>',
    description: 'Run acceptance test for a milestone.',
    category: 'planning',
  },
  {
    slug: 'sdlc-specialize',
    invocation: '/sdlc-specialize <feature-slug>',
    description: 'Shape a feature with specialized domain knowledge.',
    category: 'planning',
  },

  // workspace
  {
    slug: 'sdlc-ponder',
    invocation: '/sdlc-ponder [slug or new idea]',
    description: 'Open the ideation workspace with recruited thought partners.',
    category: 'workspace',
  },
  {
    slug: 'sdlc-ponder-commit',
    invocation: '/sdlc-ponder-commit <slug>',
    description: 'Crystallize a pondered idea into milestones and features.',
    category: 'workspace',
  },
  {
    slug: 'sdlc-recruit',
    invocation: '/sdlc-recruit <role>',
    description: 'Recruit an expert thought partner as a persistent agent.',
    category: 'workspace',
  },
  {
    slug: 'sdlc-empathy',
    invocation: '/sdlc-empathy <subject>',
    description: 'Deep user perspective interviews before making decisions.',
    category: 'workspace',
  },
  {
    slug: 'sdlc-spike',
    invocation: '/sdlc-spike <topic>',
    description: 'Time-boxed technical investigation on an uncertain area.',
    category: 'workspace',
  },
  {
    slug: 'sdlc-hypothetical-planning',
    invocation: '/sdlc-hypothetical-planning <scenario>',
    description: 'Plan a hypothetical scenario without committing state.',
    category: 'workspace',
  },
  {
    slug: 'sdlc-hypothetical-do',
    invocation: '/sdlc-hypothetical-do <scenario>',
    description: 'Execute a hypothetical scenario.',
    category: 'workspace',
  },
  {
    slug: 'sdlc-convo-mine',
    invocation: '/sdlc-convo-mine',
    description: 'Extract insights and tasks from conversation history.',
    category: 'workspace',
  },

  // analysis
  {
    slug: 'sdlc-enterprise-readiness',
    invocation: '/sdlc-enterprise-readiness',
    description: 'Production readiness analysis.',
    category: 'analysis',
  },
  {
    slug: 'sdlc-quality-fix',
    invocation: '/sdlc-quality-fix',
    description: 'Fix failing quality-check results — triage by failure count and apply targeted fixes.',
    category: 'analysis',
  },
  {
    slug: 'sdlc-setup-quality-gates',
    invocation: '/sdlc-setup-quality-gates',
    description: 'Set up pre-commit hooks.',
    category: 'analysis',
  },
  {
    slug: 'sdlc-vision-adjustment',
    invocation: '/sdlc-vision-adjustment',
    description: 'Adjust the project vision document.',
    category: 'analysis',
  },
  {
    slug: 'sdlc-architecture-adjustment',
    invocation: '/sdlc-architecture-adjustment',
    description: 'Adjust the architecture document.',
    category: 'analysis',
  },
  {
    slug: 'sdlc-guideline',
    invocation: '/sdlc-guideline [slug]',
    description: 'Open the guideline workspace — gather evidence and publish engineering guidelines.',
    category: 'analysis',
  },

  // tooling
  {
    slug: 'sdlc-tool-run',
    invocation: '/sdlc-tool-run <tool-name>',
    description: 'Run a custom tool.',
    category: 'tooling',
  },
  {
    slug: 'sdlc-tool-build',
    invocation: '/sdlc-tool-build <tool-name>',
    description: 'Build a new custom tool.',
    category: 'tooling',
  },
  {
    slug: 'sdlc-skill-build',
    invocation: '/sdlc-skill-build <skill-name>',
    description: 'Build a new agent skill.',
    category: 'tooling',
  },
  {
    slug: 'sdlc-tool-audit',
    invocation: '/sdlc-tool-audit <tool-name>',
    description: 'Audit a tool for correctness and quality.',
    category: 'tooling',
  },
  {
    slug: 'sdlc-tool-uat',
    invocation: '/sdlc-tool-uat <tool-name>',
    description: 'User acceptance test a custom tool.',
    category: 'tooling',
  },
  {
    slug: 'sdlc-cookbook',
    invocation: '/sdlc-cookbook',
    description: 'Browse and run cookbook recipes.',
    category: 'tooling',
  },
  {
    slug: 'sdlc-cookbook-run',
    invocation: '/sdlc-cookbook-run <recipe>',
    description: 'Run a specific cookbook recipe.',
    category: 'tooling',
  },
  {
    slug: 'sdlc-suggest',
    invocation: '/sdlc-suggest',
    description: 'Get suggestions for what to work on next.',
    category: 'tooling',
  },

  // project
  {
    slug: 'sdlc-init',
    invocation: '/sdlc-init',
    description: 'Interview to bootstrap vision, architecture, config, and team for a new project.',
    category: 'project',
  },
]
