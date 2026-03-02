import { useState, useMemo } from 'react'
import { Search } from 'lucide-react'
import { CopyButton } from '@/components/shared/CopyButton'
import {
  COMMANDS,
  CATEGORY_LABELS,
  CATEGORY_ORDER,
  type CommandEntry,
  type CommandCategory,
} from './commands-data'

function CommandRow({ entry }: { entry: CommandEntry }) {
  return (
    <div className="flex items-start gap-3 py-3 px-4 rounded-lg border border-border bg-card hover:bg-accent/30 transition-colors">
      <div className="flex-1 min-w-0">
        <code className="text-sm font-mono font-medium text-foreground">{entry.invocation}</code>
        <p className="text-xs text-muted-foreground mt-0.5">{entry.description}</p>
      </div>
      <CopyButton text={entry.invocation} />
    </div>
  )
}

function CategorySection({
  category,
  commands,
}: {
  category: CommandCategory
  commands: CommandEntry[]
}) {
  return (
    <div>
      <div className="flex items-center gap-2 mb-2 mt-6 first:mt-0">
        <h3 className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">
          {CATEGORY_LABELS[category]}
        </h3>
        <span className="text-[10px] text-muted-foreground/40">· {commands.length}</span>
      </div>
      <div className="space-y-1.5">
        {commands.map(entry => (
          <CommandRow key={entry.slug} entry={entry} />
        ))}
      </div>
    </div>
  )
}

export function CommandsCatalog() {
  const [query, setQuery] = useState('')

  const filtered = useMemo(() => {
    if (!query.trim()) return COMMANDS
    const q = query.toLowerCase()
    return COMMANDS.filter(
      cmd => cmd.slug.includes(q) || cmd.description.toLowerCase().includes(q)
    )
  }, [query])

  const grouped = useMemo(() => {
    const map = new Map<CommandCategory, CommandEntry[]>()
    for (const cat of CATEGORY_ORDER) {
      const entries = filtered.filter(c => c.category === cat)
      if (entries.length > 0) {
        map.set(cat, entries)
      }
    }
    return map
  }, [filtered])

  return (
    <div>
      {/* Search bar */}
      <div className="relative mb-4">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground/50 pointer-events-none" />
        <input
          type="text"
          value={query}
          onChange={e => setQuery(e.target.value)}
          placeholder="Search commands…"
          className="w-full pl-9 pr-4 py-2 text-sm rounded-lg border border-border bg-background placeholder:text-muted-foreground/50 focus:outline-none focus:ring-2 focus:ring-ring"
        />
        <span className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-muted-foreground/40">
          {filtered.length} / {COMMANDS.length}
        </span>
      </div>

      {/* Results */}
      {grouped.size === 0 ? (
        <div className="py-12 text-center">
          <p className="text-sm text-muted-foreground">No commands match &ldquo;{query}&rdquo;</p>
        </div>
      ) : (
        <div>
          {CATEGORY_ORDER.map(cat => {
            const entries = grouped.get(cat)
            if (!entries) return null
            return (
              <CategorySection key={cat} category={cat} commands={entries} />
            )
          })}
        </div>
      )}
    </div>
  )
}
