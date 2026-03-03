import { useState } from 'react'
import { Link } from 'react-router-dom'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { StatusBadge } from '@/components/shared/StatusBadge'
import type { MilestoneSummary } from '@/lib/types'

interface ArchiveZoneProps {
  milestones: MilestoneSummary[]
}

export function ArchiveZone({ milestones }: ArchiveZoneProps) {
  const [expanded, setExpanded] = useState(false)

  if (milestones.length === 0) return null

  return (
    <section className="mb-8">
      <button
        onClick={() => setExpanded(v => !v)}
        className="flex items-center gap-2 w-full text-left px-3 py-2 rounded-lg border border-border/50 bg-muted/20 hover:bg-muted/40 transition-colors mb-2"
      >
        {expanded
          ? <ChevronDown className="w-4 h-4 text-muted-foreground shrink-0" />
          : <ChevronRight className="w-4 h-4 text-muted-foreground shrink-0" />
        }
        <span className="text-sm font-medium">Archive</span>
        <span className="text-xs text-muted-foreground">({milestones.length} released)</span>
      </button>

      {expanded && (
        <div className="space-y-1.5">
          {milestones.map(m => (
            <div
              key={m.slug}
              className="flex items-center gap-2 px-3 py-2 bg-muted/30 border border-border/40 rounded-lg"
            >
              <Link
                to={`/milestones/${m.slug}`}
                className="text-sm font-medium hover:text-primary transition-colors"
              >
                {m.title}
              </Link>
              <StatusBadge status={m.status} />
              <span className="text-xs font-mono text-muted-foreground/50">{m.slug}</span>
              <span className="text-xs text-muted-foreground ml-auto">{m.features.length} features</span>
            </div>
          ))}
        </div>
      )}
    </section>
  )
}
