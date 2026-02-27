import { Link, useLocation } from 'react-router-dom'
import { cn } from '@/lib/utils'
import { LayoutDashboard, FolderKanban, Milestone, Settings, Search, Archive } from 'lucide-react'

const navItems = [
  { path: '/', label: 'Dashboard', icon: LayoutDashboard, exact: true },
  { path: '/features', label: 'Features', icon: FolderKanban, exact: false },
  { path: '/milestones', label: 'Milestones', icon: Milestone, exact: true },
  { path: '/milestones/archive', label: 'Archive', icon: Archive, exact: true },
  { path: '/config', label: 'Config', icon: Settings, exact: false },
]

interface SidebarProps {
  /** Called after a nav link is clicked (used to close mobile sidebar). */
  onNavigate?: () => void
  /** Called when the search trigger button is clicked. */
  onSearch?: () => void
}

export function Sidebar({ onNavigate, onSearch }: SidebarProps) {
  const location = useLocation()

  return (
    <aside className="w-56 h-full bg-card border-r border-border flex flex-col">
      <div className="px-4 py-5 border-b border-border">
        <h1 className="text-lg font-semibold tracking-tight">SDLC</h1>
        <p className="text-xs text-muted-foreground mt-0.5">Feature Lifecycle</p>
      </div>
      <nav className="flex-1 px-2 py-3 space-y-0.5">
        {navItems.map(({ path, label, icon: Icon, exact }) => {
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
      </nav>
      <div className="px-2 py-3 border-t border-border">
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
