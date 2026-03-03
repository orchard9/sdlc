import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { Telescope } from 'lucide-react'
import { api } from '@/api/client'
import { StatusBadge } from '@/components/shared/StatusBadge'
import type { MilestoneSummary, FeatureSummary, PonderSummary } from '@/lib/types'

interface HorizonZoneProps {
  milestones: MilestoneSummary[]
  featureBySlug: Map<string, FeatureSummary>
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)
  return (
    <button
      onClick={async () => {
        await navigator.clipboard.writeText(text)
        setCopied(true)
        setTimeout(() => setCopied(false), 1500)
      }}
      className="text-xs text-muted-foreground hover:text-foreground shrink-0 px-1.5 py-0.5 rounded border border-border/50 hover:border-border transition-colors"
      title={text}
    >
      {copied ? '✓' : 'copy'}
    </button>
  )
}

export function HorizonZone({ milestones, featureBySlug }: HorizonZoneProps) {
  const [activePonders, setActivePonders] = useState<PonderSummary[]>([])

  useEffect(() => {
    api.getRoadmap().then(all => {
      setActivePonders(
        all.filter(p => p.status === 'exploring' || p.status === 'converging')
      )
    }).catch(() => { /* silent — roadmap list is optional context */ })
  }, [])

  // Horizon milestones: active milestones where all assigned features are still in draft
  const horizonMilestones = milestones.filter(m => {
    if (m.features.length === 0) return true
    return m.features.every(slug => {
      const f = featureBySlug.get(slug)
      return !f || f.phase === 'draft'
    })
  })

  if (horizonMilestones.length === 0 && activePonders.length === 0) return null

  return (
    <section className="mb-8">
      <div className="flex items-center gap-2 px-1 mb-3">
        <Telescope className="w-4 h-4 text-muted-foreground" />
        <h2 className="text-sm font-semibold text-muted-foreground">Horizon</h2>
      </div>

      <div className="bg-card border border-border rounded-xl overflow-hidden">
        {/* Upcoming milestones */}
        {horizonMilestones.length > 0 && (
          <div>
            <div className="px-4 py-2 border-b border-border/50 bg-muted/20">
              <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                Upcoming Milestones
              </span>
            </div>
            <div className="divide-y divide-border/30">
              {horizonMilestones.map(m => (
                <div key={m.slug} className="flex items-center gap-3 px-4 py-2.5">
                  <Link
                    to={`/milestones/${m.slug}`}
                    className="text-sm font-medium hover:text-primary transition-colors flex-1 min-w-0 truncate"
                  >
                    {m.title}
                  </Link>
                  <StatusBadge status={m.status} />
                  <span className="text-xs text-muted-foreground shrink-0">
                    {m.features.length} feature{m.features.length !== 1 ? 's' : ''}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Active ponders */}
        {activePonders.length > 0 && (
          <div className={horizonMilestones.length > 0 ? 'border-t border-border/30' : ''}>
            <div className="px-4 py-2 border-b border-border/50 bg-muted/20">
              <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                Active Ponders
              </span>
            </div>
            <div className="divide-y divide-border/30">
              {activePonders.map(p => (
                <div key={p.slug} className="flex items-center gap-3 px-4 py-2.5">
                  <Link
                    to={`/ponder/${p.slug}`}
                    className="text-sm font-medium hover:text-primary transition-colors flex-1 min-w-0 truncate"
                  >
                    {p.title}
                  </Link>
                  <StatusBadge status={p.status} />
                  {p.tags.slice(0, 2).map(tag => (
                    <span
                      key={tag}
                      className="text-xs text-muted-foreground/70 bg-muted/60 px-1.5 py-0.5 rounded font-mono shrink-0"
                    >
                      #{tag}
                    </span>
                  ))}
                  <CopyButton text={`/sdlc-ponder ${p.slug}`} />
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </section>
  )
}
