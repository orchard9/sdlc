import { Link, useLocation } from 'react-router-dom'
import { cn } from '@/lib/utils'
import { LayoutDashboard, FolderKanban, Milestone, Search, Archive, Lightbulb, Microscope, Lock, Wrench, TrendingUp, MessageSquarePlus, Wifi, Target, GitBranch, Rocket, Terminal, Map, Code2, ScrollText, Zap, Bot } from 'lucide-react'

const navGroups = [
  {
    label: 'work',
    items: [
      { path: '/', label: 'Dashboard', icon: LayoutDashboard, exact: true },
      { path: '/milestones', label: 'Milestones', icon: Milestone, exact: true },
      { path: '/features', label: 'Features', icon: FolderKanban, exact: false },
      { path: '/milestones/archive', label: 'Archive', icon: Archive, exact: true },
    ],
  },
  {
    label: 'plan',
    items: [
      { path: '/feedback', label: 'Feedback', icon: MessageSquarePlus, exact: false },
      { path: '/ponder', label: 'Ponder', icon: Lightbulb, exact: false },
      { path: '/investigations', label: 'Root Cause', icon: Microscope, exact: false },
      { path: '/evolve', label: 'Evolve', icon: TrendingUp, exact: false },
    ],
  },
  {
    label: 'setup',
    items: [
      { path: '/tools', label: 'Tools', icon: Wrench, exact: false },
      { path: '/secrets', label: 'Secrets', icon: Lock, exact: false },
      { path: '/agents', label: 'Agents', icon: Bot, exact: false },
    ],
  },
  {
    label: 'integrate',
    items: [
      { path: '/network', label: 'Network', icon: Wifi, exact: false },
    ],
  },
  {
    label: 'project',
    items: [
      { path: '/vision', label: 'Vision', icon: Target, exact: true },
      { path: '/architecture', label: 'Architecture', icon: GitBranch, exact: true },
    ],
  },
  {
    label: 'docs',
    items: [
      { path: '/docs/quickstart', label: 'Quick Start', icon: Rocket, exact: true },
      { path: '/docs/commands', label: 'Commands', icon: Terminal, exact: true },
      { path: '/docs/planning-flow', label: 'Planning Flow', icon: Map, exact: true },
      { path: '/docs/development-flow', label: 'Development Flow', icon: Code2, exact: true },
      { path: '/docs/release-notes', label: 'Release Notes', icon: ScrollText, exact: true },
    ],
  },
]

interface SidebarProps {
  /** Called after a nav link is clicked (used to close mobile sidebar). */
  onNavigate?: () => void
  /** Called when the search trigger button is clicked. */
  onSearch?: () => void
  /** Called when the Fix Right Away button is clicked. */
  onFixRightAway?: () => void
}

export function Sidebar({ onNavigate, onSearch, onFixRightAway }: SidebarProps) {
  const location = useLocation()

  return (
    <aside className="w-56 h-full bg-card border-r border-border flex flex-col">
      <div className="px-4 py-5 border-b border-border">
        <h1 className="text-lg font-semibold tracking-tight">SDLC</h1>
        <p className="text-xs text-muted-foreground mt-0.5">Feature Lifecycle</p>
      </div>
      <nav className="flex-1 px-2 py-4 space-y-4 overflow-y-auto">
        {navGroups.map(group => (
          <div key={group.label}>
            <p className="px-3 pb-1 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/40">
              {group.label}
            </p>
            <div className="space-y-0.5">
              {group.items.map(({ path, label, icon: Icon, exact }) => {
                const active = exact ? location.pathname === path : location.pathname.startsWith(path)
                return (
                  <Link
                    key={path}
                    to={path}
                    onClick={onNavigate}
                    className={cn(
                      'flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm transition-colors',
                      active
                        ? 'bg-accent text-accent-foreground font-medium'
                        : 'text-muted-foreground hover:text-foreground hover:bg-accent/50'
                    )}
                  >
                    <Icon className="w-4 h-4" />
                    {label}
                  </Link>
                )
              })}
            </div>
          </div>
        ))}
      </nav>
      <div className="px-2 py-3 border-t border-border space-y-0.5">
        <button
          onClick={onFixRightAway}
          className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
        >
          <Zap className="w-4 h-4" />
          <span className="flex-1 text-left">Fix Right Away</span>
          <kbd className="text-xs bg-muted border border-border/50 rounded px-1.5 py-0.5 font-mono">⌘⇧F</kbd>
        </button>
        <button
          onClick={onSearch}
          className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
        >
          <Search className="w-4 h-4" />
          <span className="flex-1 text-left">Search</span>
          <kbd className="text-xs bg-muted border border-border/50 rounded px-1.5 py-0.5 font-mono">⌘K</kbd>
        </button>
      </div>
    </aside>
  )
}
