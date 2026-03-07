import { AlertTriangle, ArrowUpRight, Loader2, Siren } from 'lucide-react'
import type { HubAttentionItem } from '@/lib/types'
import { cn } from '@/lib/utils'

function severityClasses(severity: HubAttentionItem['severity']) {
  switch (severity) {
    case 'error':
      return 'border-rose-500/20 bg-rose-500/5'
    case 'warning':
      return 'border-amber-400/20 bg-amber-400/5'
    case 'success':
      return 'border-emerald-500/20 bg-emerald-500/5'
    default:
      return 'border-sky-500/20 bg-sky-500/5'
  }
}

function severityIconClasses(severity: HubAttentionItem['severity']) {
  switch (severity) {
    case 'error':
      return 'text-rose-300'
    case 'warning':
      return 'text-amber-300'
    case 'success':
      return 'text-emerald-300'
    default:
      return 'text-sky-300'
  }
}

export function AttentionZone({
  items,
  onOpenProject,
}: {
  items: HubAttentionItem[]
  onOpenProject: (url: string) => void
}) {
  return (
    <section className="rounded-xl border border-border bg-card p-5">
      <div className="flex items-center gap-3 mb-4">
        <div className="flex h-10 w-10 items-center justify-center rounded-lg border border-border bg-muted/30 text-amber-300">
          <Siren className="w-5 h-5" />
        </div>
        <div>
          <h2 className="text-base font-semibold tracking-tight">Attention</h2>
          <p className="text-sm text-muted-foreground">The handful of things a human should look at first.</p>
        </div>
      </div>

      {items.length === 0 ? (
        <div className="rounded-lg border border-emerald-500/20 bg-emerald-500/5 px-4 py-5 text-sm text-emerald-100">
          Fleet is quiet. No projects currently need intervention.
        </div>
      ) : (
        <div className="space-y-3">
          {items.slice(0, 6).map(item => (
            <div
              key={item.id}
              className={cn(
                'rounded-xl border px-4 py-4 transition-colors',
                severityClasses(item.severity),
              )}
            >
              <div className="flex items-start gap-3">
                <div className={cn('mt-0.5', severityIconClasses(item.severity))}>
                  {item.title.toLowerCase().includes('provision')
                    ? <Loader2 className="w-4 h-4 animate-spin" />
                    : <AlertTriangle className="w-4 h-4" />}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="font-medium tracking-tight">{item.title}</div>
                  <div className="mt-1 text-sm text-muted-foreground">{item.detail}</div>
                </div>
                {item.url && (
                  <button
                    onClick={() => onOpenProject(item.url!)}
                    className="inline-flex items-center gap-1 rounded-lg border border-border bg-background px-3 py-1.5 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
                  >
                    Open
                    <ArrowUpRight className="w-3.5 h-3.5" />
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </section>
  )
}
