import { useState } from 'react'
import { Link } from 'react-router-dom'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { CopyButton } from '@/components/shared/CopyButton'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import type { Wave, BlockedFeatureItem } from '@/lib/types'
import { ChevronDown, ChevronRight, GitBranch, Loader2, Play } from 'lucide-react'

interface WavePlanProps {
  waves: Wave[]
  blocked: BlockedFeatureItem[]
  nextCommands?: string[]
  showCommands?: boolean
}

function WaveSection({ wave, defaultExpanded }: { wave: Wave; defaultExpanded: boolean }) {
  const [expanded, setExpanded] = useState(defaultExpanded)
  const { isRunning, startRun, focusRun, getRunForKey } = useAgentRuns()

  return (
    <div className="border border-border rounded-lg overflow-hidden">
      <button
        onClick={() => setExpanded(v => !v)}
        className="w-full flex items-center gap-2 px-3 py-2 bg-muted/30 hover:bg-muted/50 transition-colors text-left"
      >
        {expanded
          ? <ChevronDown className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
          : <ChevronRight className="w-3.5 h-3.5 text-muted-foreground shrink-0" />}
        <span className="text-xs font-semibold">Wave {wave.number}</span>
        <span className="text-xs text-muted-foreground">— {wave.label}</span>
        <span className="text-xs text-muted-foreground ml-auto">{wave.items.length} features</span>
        {wave.needs_worktrees && (
          <span className="flex items-center gap-0.5 text-xs text-amber-400">
            <GitBranch className="w-3 h-3" />
            worktrees
          </span>
        )}
      </button>

      {expanded && (
        <div className="divide-y divide-border/50">
          {wave.items.map(item => {
            const running = isRunning(item.slug)
            const activeRun = getRunForKey(item.slug)

            const handleRun = () => startRun({
              key: item.slug,
              runType: 'feature',
              target: item.slug,
              label: item.slug,
              startUrl: `/api/run/${item.slug}`,
              stopUrl: `/api/run/${item.slug}/stop`,
            })

            const handleFocus = () => { if (activeRun) focusRun(activeRun.id) }

            return (
              <div
                key={item.slug}
                className="flex items-center gap-3 px-3 py-2 text-xs"
              >
                <Link
                  to={`/features/${item.slug}`}
                  className="font-medium hover:text-primary transition-colors min-w-0 truncate"
                  style={{ flex: '0 0 160px' }}
                >
                  {item.slug}
                </Link>
                <StatusBadge status={item.phase} />
                <span className="text-muted-foreground flex-1 truncate">{item.action}</span>
                {item.needs_worktree && !running && (
                  <span title="Needs worktree"><GitBranch className="w-3 h-3 text-amber-400 shrink-0" /></span>
                )}
                {item.blocked_by.length > 0 && !running && (
                  <span className="text-muted-foreground/60 truncate" style={{ maxWidth: 120 }}>
                    ← {item.blocked_by.join(', ')}
                  </span>
                )}
                {running ? (
                  <button
                    onClick={handleFocus}
                    className="shrink-0 inline-flex items-center gap-1 px-2 py-0.5 rounded border border-border bg-muted text-muted-foreground hover:bg-muted/80 transition-colors"
                  >
                    <Loader2 className="w-3 h-3 animate-spin" />
                    Running
                  </button>
                ) : (
                  <>
                    <CopyButton text={`/sdlc-run ${item.slug}`} className="shrink-0 p-1 rounded border border-border/50 bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors" />
                    <button
                      onClick={handleRun}
                      className="shrink-0 inline-flex items-center gap-1 px-2 py-0.5 rounded border border-primary/40 bg-primary/10 text-primary hover:bg-primary/20 transition-colors"
                    >
                      <Play className="w-3 h-3" />
                      Run
                    </button>
                  </>
                )}
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}

export function WavePlan({ waves, blocked, nextCommands, showCommands = true }: WavePlanProps) {
  const { isRunning, startRun, focusRun, getRunForKey } = useAgentRuns()

  // Derive the command list: prefer next_commands from the API; fall back to
  // one /sdlc-run per Wave-1 item so the UI always shows something actionable.
  const commands = nextCommands && nextCommands.length > 0
    ? nextCommands
    : waves[0]?.items.map(item => `/sdlc-run ${item.slug}`) ?? []

  return (
    <div className="space-y-2">
      {waves.map((wave) => (
        <WaveSection key={wave.number} wave={wave} defaultExpanded={false} />
      ))}

      {blocked.length > 0 && (
        <div className="border border-amber-500/20 bg-amber-950/20 rounded-lg px-3 py-2 space-y-1">
          <p className="text-xs font-semibold text-amber-400">Blocked</p>
          {blocked.map(b => (
            <div key={b.slug} className="flex items-center gap-2 text-xs text-muted-foreground">
              <Link to={`/features/${b.slug}`} className="font-medium hover:text-foreground transition-colors">
                {b.slug}
              </Link>
              <span>— {b.reason}</span>
            </div>
          ))}
        </div>
      )}

      {showCommands && commands.length > 0 && (
        <div className="flex flex-wrap gap-1.5 pt-1">
          {commands.map(cmd => {
            // Resolve which MCP endpoint (if any) backs this command.
            let runKey: string | null = null
            let runOpts: Parameters<typeof startRun>[0] | null = null

            if (cmd.startsWith('/sdlc-run ')) {
              const slug = cmd.slice('/sdlc-run '.length)
              runKey = slug
              runOpts = { key: slug, runType: 'feature', target: slug, label: slug, startUrl: `/api/run/${slug}`, stopUrl: `/api/run/${slug}/stop` }
            } else if (cmd.startsWith('/sdlc-prepare ')) {
              const slug = cmd.slice('/sdlc-prepare '.length)
              runKey = `milestone-prepare:${slug}`
              runOpts = { key: `milestone-prepare:${slug}`, runType: 'milestone_prepare', target: slug, label: `prepare: ${slug}`, startUrl: `/api/milestone/${slug}/prepare`, stopUrl: `/api/milestone/${slug}/prepare/stop` }
            }

            const running = runKey ? isRunning(runKey) : false
            const activeRun = runKey ? getRunForKey(runKey) : undefined
            const handleFocus = () => { if (activeRun) focusRun(activeRun.id) }

            return (
              <div key={cmd} className="flex items-center gap-1.5">
                <code className="text-[10px] font-mono bg-muted/60 border border-border/50 px-2 py-0.5 rounded text-muted-foreground">
                  {cmd}
                </code>
                <CopyButton text={cmd} className="shrink-0 p-1 rounded border border-border/50 bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors" />
                {runOpts && (
                  running ? (
                    <button
                      onClick={handleFocus}
                      className="shrink-0 inline-flex items-center gap-1 px-2 py-0.5 rounded border border-border bg-muted text-muted-foreground text-[10px] hover:bg-muted/80 transition-colors"
                    >
                      <Loader2 className="w-3 h-3 animate-spin" />
                      Running
                    </button>
                  ) : (
                    <button
                      onClick={() => startRun(runOpts)}
                      className="shrink-0 inline-flex items-center gap-1 px-2 py-0.5 rounded border border-primary/40 bg-primary/10 text-primary text-[10px] hover:bg-primary/20 transition-colors"
                    >
                      <Play className="w-3 h-3" />
                      Run
                    </button>
                  )
                )}
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}
