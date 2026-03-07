import { Activity, ArrowUpRight, CircleAlert, CircleCheckBig, Info, Rocket } from 'lucide-react'
import type { HubActivityEntry } from '@/lib/types'

function iconFor(kind: string) {
  if (kind.includes('provision')) return Rocket
  return Activity
}

function severityTone(severity: HubActivityEntry['severity']) {
  switch (severity) {
    case 'error':
      return 'text-rose-300 border-rose-500/20 bg-rose-500/10'
    case 'warning':
      return 'text-amber-200 border-amber-500/20 bg-amber-500/10'
    case 'success':
      return 'text-emerald-200 border-emerald-500/20 bg-emerald-500/10'
    default:
      return 'text-sky-200 border-sky-500/20 bg-sky-500/10'
  }
}

function severityIcon(severity: HubActivityEntry['severity']) {
  switch (severity) {
    case 'error':
    case 'warning':
      return CircleAlert
    case 'success':
      return CircleCheckBig
    default:
      return Info
  }
}

export function ActivityFeed({ items }: { items: HubActivityEntry[] }) {
  return (
    <section className="rounded-xl border border-border bg-card p-5">
      <div className="flex items-center gap-3 mb-4">
        <div className="flex h-10 w-10 items-center justify-center rounded-lg border border-border bg-muted/30 text-primary">
          <Activity className="w-5 h-5" />
        </div>
        <div>
          <h2 className="text-base font-semibold tracking-tight">Activity</h2>
          <p className="text-sm text-muted-foreground">Recent movement across the fleet and provisioning pipeline.</p>
        </div>
      </div>

      <div className="space-y-3">
        {items.length === 0 ? (
          <div className="rounded-lg border border-border bg-background/40 px-4 py-5 text-sm text-muted-foreground">
            No recent activity yet.
          </div>
        ) : (
          items.slice(0, 12).map(item => {
            const KindIcon = iconFor(item.kind)
            const SeverityIcon = severityIcon(item.severity)
            return (
              <div key={item.id} className="rounded-xl border border-border/70 bg-background/40 p-4">
                <div className="flex items-start gap-3">
                  <div className={`flex h-9 w-9 items-center justify-center rounded-lg border ${severityTone(item.severity)}`}>
                    <KindIcon className="w-4 h-4" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-start justify-between gap-3">
                      <div className="font-medium tracking-tight">{item.title}</div>
                      <div className="text-xs text-muted-foreground whitespace-nowrap">
                        {new Date(item.created_at).toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' })}
                      </div>
                    </div>
                    {item.detail && <div className="text-sm text-muted-foreground mt-1">{item.detail}</div>}
                    <div className="mt-3 flex items-center gap-3 text-xs text-muted-foreground">
                      <span className="inline-flex items-center gap-1">
                        <SeverityIcon className="w-3.5 h-3.5" />
                        {item.severity}
                      </span>
                      {item.slug && <span>{item.slug}</span>}
                      {item.url && (
                        <button
                          onClick={() => window.open(item.url!, '_blank')}
                          className="inline-flex items-center gap-1 rounded-md px-2 py-1 hover:bg-accent hover:text-foreground transition-colors"
                        >
                          Open
                          <ArrowUpRight className="w-3.5 h-3.5" />
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            )
          })
        )}
      </div>
    </section>
  )
}
