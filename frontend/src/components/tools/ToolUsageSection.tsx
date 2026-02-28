import { useState } from 'react'
import { CommandBlock } from '@/components/shared/CommandBlock'
import { Bot, Terminal, ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { ToolMeta } from '@/lib/types'

// ---------------------------------------------------------------------------
// CLI help construction from ToolMeta
// ---------------------------------------------------------------------------

type SchemaProps = Record<string, { type?: string; description?: string }>

export function buildCliCommand(tool: ToolMeta): string {
  const schema = tool.input_schema as { properties?: SchemaProps; required?: string[] }
  const props = schema?.properties ?? {}
  const required = schema?.required ?? []

  const flags = Object.entries(props)
    .filter(([key]) => required.includes(key))
    .map(([key, prop]) => {
      if (tool.name === 'ama' && key === 'question') return '--question "..."'
      return `--${key} <${(prop.type ?? 'value').toUpperCase()}>`
    })

  const optionals = Object.entries(props)
    .filter(([key]) => !required.includes(key))
    .map(([key, prop]) => {
      if (tool.name === 'quality-check' && key === 'scope') return '[--scope <filter>]'
      return `[--${key} <${(prop.type ?? 'value').toUpperCase()}>]`
    })

  const allFlags = [...flags, ...optionals]
  return `sdlc tool run ${tool.name}${allFlags.length ? ' ' + allFlags.join(' ') : ''}`
}

export function buildCliHelp(tool: ToolMeta): string {
  const schema = tool.input_schema as { properties?: SchemaProps; required?: string[] }
  const props = schema?.properties ?? {}
  const required = (schema?.required ?? []) as string[]
  const outSchema = tool.output_schema as { properties?: SchemaProps }
  const outProps = outSchema?.properties ?? {}

  const lines: string[] = []

  lines.push(`sdlc tool run ${tool.name} [OPTIONS]`)
  lines.push('')
  lines.push(`  ${tool.description}`)
  lines.push('')

  if (tool.requires_setup) {
    lines.push('Setup (required before first use):')
    lines.push(`  sdlc tool run ${tool.name} --setup`)
    if (tool.setup_description) {
      lines.push(`  └ ${tool.setup_description}`)
    }
    lines.push('')
  }

  if (Object.keys(props).length > 0) {
    lines.push('Options:')
    for (const [key, prop] of Object.entries(props)) {
      const req = required.includes(key)
      const type = (prop.type ?? 'value').toUpperCase()
      const desc = prop.description ?? ''
      lines.push(`  --${key} <${type}>${req ? '  (required)' : '  (optional)'}`)
      if (desc) lines.push(`    ${desc}`)
    }
    lines.push('')
  }

  lines.push('Input:')
  if (Object.keys(props).length > 0) {
    const ex: Record<string, string> = {}
    for (const [key, prop] of Object.entries(props)) {
      ex[key] = `<${prop.type ?? 'value'}>`
    }
    lines.push('  ' + JSON.stringify(ex, null, 2).split('\n').join('\n  '))
  } else {
    lines.push('  {}  (no input required)')
  }
  lines.push('')

  lines.push('Output:')
  if (Object.keys(outProps).length > 0) {
    const ex: Record<string, string> = {}
    for (const [key, prop] of Object.entries(outProps)) {
      ex[key] = `<${prop.type ?? 'value'}>`
    }
    lines.push('  ' + JSON.stringify({ ok: '<boolean>', data: ex }, null, 2).split('\n').join('\n  '))
  } else {
    lines.push('  { "ok": <boolean>, "data": { ... } }')
  }

  return lines.join('\n')
}

// ---------------------------------------------------------------------------
// Usage section component
// ---------------------------------------------------------------------------

export function ToolUsageSection({ tool }: { tool: ToolMeta }) {
  const [expanded, setExpanded] = useState(false)
  const agentCmd = `/sdlc-tool-run ${tool.name}`
  const cliCmd = buildCliCommand(tool)

  return (
    <div className="rounded-lg border border-border/50 bg-muted/20 overflow-hidden">
      <div className="px-3 py-2.5 space-y-2">
        {/* Agent */}
        <div>
          <div className="flex items-center gap-1.5 mb-1">
            <Bot className="w-3 h-3 text-muted-foreground/60" />
            <span className="text-[10px] font-medium text-muted-foreground/60 uppercase tracking-wider">Agent</span>
          </div>
          <CommandBlock cmd={agentCmd} />
        </div>
        {/* CLI */}
        <div>
          <div className="flex items-center gap-1.5 mb-1">
            <Terminal className="w-3 h-3 text-muted-foreground/60" />
            <span className="text-[10px] font-medium text-muted-foreground/60 uppercase tracking-wider">CLI</span>
          </div>
          <CommandBlock cmd={cliCmd} />
        </div>
      </div>

      {/* Expandable full reference */}
      <button
        onClick={() => setExpanded(v => !v)}
        className="w-full flex items-center gap-1 px-3 py-1.5 text-[10px] font-medium text-muted-foreground/50 hover:text-muted-foreground border-t border-border/40 transition-colors"
      >
        <ChevronDown className={cn('w-3 h-3 transition-transform', expanded && 'rotate-180')} />
        {expanded ? 'Hide reference' : 'Full reference'}
      </button>

      {expanded && (
        <pre className="px-3 pb-3 text-[11px] font-mono text-muted-foreground/70 whitespace-pre overflow-x-auto leading-relaxed border-t border-border/40 bg-muted/10 pt-2">
          {buildCliHelp(tool)}
        </pre>
      )}
    </div>
  )
}
