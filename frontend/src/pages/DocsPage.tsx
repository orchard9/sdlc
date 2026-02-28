import { useParams } from 'react-router-dom'
import { Rocket, Terminal, Map, Code2, ScrollText } from 'lucide-react'

const sections = {
  quickstart: {
    icon: Rocket,
    title: 'Quick Start',
    description: 'Get up and running with sdlc in minutes.',
    placeholder: 'Quick start guide — installation, initialization, and your first feature lifecycle.',
  },
  commands: {
    icon: Terminal,
    title: 'Commands',
    description: 'Reference for all /sdlc-* agent commands.',
    placeholder: 'Templated agent commands — /sdlc-run, /sdlc-next, /sdlc-ponder, /sdlc-plan, and more.',
  },
  'planning-flow': {
    icon: Map,
    title: 'Planning Flow',
    description: 'How to move from idea to planned feature.',
    placeholder: 'Planning workflow — ponder → commit → prepare → wave plan.',
  },
  'development-flow': {
    icon: Code2,
    title: 'Development Flow',
    description: 'How features progress through implementation to release.',
    placeholder: 'Development workflow — draft → specify → implement → review → audit → QA → merge.',
  },
  'release-notes': {
    icon: ScrollText,
    title: 'Release Notes',
    description: 'Changelog and version history.',
    placeholder: 'Release notes — what changed, when, and why.',
  },
} as const

type SectionKey = keyof typeof sections

export function DocsPage() {
  const { section = 'quickstart' } = useParams<{ section: string }>()
  const entry = sections[section as SectionKey] ?? sections.quickstart
  const Icon = entry.icon

  return (
    <div className="max-w-3xl mx-auto p-6">
      <div className="flex items-center gap-2.5 mb-6">
        <Icon className="w-5 h-5 text-muted-foreground" />
        <h2 className="text-xl font-semibold">{entry.title}</h2>
      </div>

      <div className="border border-dashed border-border rounded-xl p-10 text-center">
        <Icon className="w-8 h-8 text-muted-foreground/30 mx-auto mb-3" />
        <p className="text-sm text-muted-foreground">{entry.description}</p>
        <p className="text-xs text-muted-foreground/60 mt-1">{entry.placeholder}</p>
      </div>
    </div>
  )
}
