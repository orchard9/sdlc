import { Link, useLocation } from 'react-router-dom'
import { cn } from '@/lib/utils'
import {
  LayoutDashboard, FolderKanban, Milestone, Search, Lightbulb, Microscope, Lock, Wrench,
  TrendingUp, MessagesSquare, Wifi, Target, GitBranch, Rocket, Terminal, Map, Code2, ScrollText,
  Zap, Bot, BookMarked, Library, CalendarClock, ChevronsLeft, ChevronsRight, BarChart2, FlaskConical,
} from 'lucide-react'

const navGroups = [
  {
    label: 'work',
    items: [
      { path: '/', label: 'Dashboard', icon: LayoutDashboard, exact: true },
      { path: '/milestones', label: 'Milestones', icon: Milestone, exact: true },
      { path: '/features', label: 'Features', icon: FolderKanban, exact: false },
      { path: '/runs', label: 'Run History', icon: BarChart2, exact: true },
    ],
  },
  {
    label: 'plan',
    items: [
      { path: '/threads', label: 'Threads', icon: MessagesSquare, exact: false },
      { path: '/ponder', label: 'Ponder', icon: Lightbulb, exact: false },
      { path: '/investigations', label: 'Root Cause', icon: Microscope, exact: false },
      { path: '/evolve', label: 'Evolve', icon: TrendingUp, exact: false },
      { path: '/guidelines', label: 'Guidelines', icon: BookMarked, exact: false },
      { path: '/spikes', label: 'Spikes', icon: FlaskConical, exact: false },
      { path: '/knowledge', label: 'Knowledge', icon: Library, exact: false },
    ],
  },
  {
    label: 'setup',
    items: [
      { path: '/tools', label: 'Tools', icon: Wrench, exact: false },
      { path: '/secrets', label: 'Secrets', icon: Lock, exact: false },
      { path: '/agents', label: 'Agents', icon: Bot, exact: false },
      { path: '/actions', label: 'Actions', icon: CalendarClock, exact: false },
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
  collapsed?: boolean
  onToggle?: () => void
  /** Called after a nav link is clicked (used to close mobile sidebar). */
  onNavigate?: () => void
  /** Called when the search trigger button is clicked. */
  onSearch?: () => void
  /** Called when the Fix Right Away button is clicked. */
  onFixRightAway?: () => void
  /** Called when the Ask Ponder button is clicked. */
  onAskPonder?: () => void
  /** Project name from config.project.name */
  projectName?: string
}

export function Sidebar({ collapsed = false, onToggle, onNavigate, onSearch, onFixRightAway, onAskPonder, projectName = 'Ponder' }: SidebarProps) {
  const location = useLocation()

  return (
    <aside
      data-testid="sidebar-rail"
      className={cn(
        'h-full bg-card border-r border-border flex flex-col transition-[width] duration-200 ease-in-out overflow-hidden',
        collapsed ? 'w-[52px]' : 'w-56',
      )}
    >
      {/* Header */}
      <div className={cn(
        'flex items-center border-b border-border shrink-0',
        collapsed ? 'justify-center px-1 py-5' : 'justify-between px-4 py-5',
      )}>
        {!collapsed && (
          <div>
            <h1 className="text-lg font-semibold tracking-tight">{projectName}</h1>
          </div>
        )}
        <button
          data-testid="sidebar-toggle"
          onClick={onToggle}
          className="p-1 rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors shrink-0"
          aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {collapsed
            ? <ChevronsRight className="w-4 h-4" />
            : <ChevronsLeft className="w-4 h-4" />
          }
        </button>
      </div>

      {/* Nav */}
      <nav className="flex-1 px-1.5 py-4 space-y-4 overflow-y-auto overflow-x-hidden">
        {navGroups.map(group => (
          <div key={group.label}>
            {!collapsed && (
              <p className="px-3 pb-1 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/40">
                {group.label}
              </p>
            )}
            <div className="space-y-0.5">
              {group.items.map(({ path, label, icon: Icon, exact }) => {
                const active = exact ? location.pathname === path : location.pathname.startsWith(path)
                return (
                  <Link
                    key={path}
                    to={path}
                    onClick={onNavigate}
                    title={collapsed ? label : undefined}
                    className={cn(
                      'flex items-center rounded-lg text-sm transition-colors',
                      collapsed ? 'justify-center p-2' : 'gap-2.5 px-3 py-2',
                      active
                        ? 'bg-accent text-accent-foreground font-medium'
                        : 'text-muted-foreground hover:text-foreground hover:bg-accent/50'
                    )}
                  >
                    <Icon className="w-4 h-4 shrink-0" />
                    {!collapsed && label}
                  </Link>
                )
              })}
            </div>
          </div>
        ))}
      </nav>

      {/* Bottom utility */}
      <div className="px-1.5 py-3 border-t border-border space-y-0.5">
        <button
          onClick={onAskPonder}
          title={collapsed ? 'Ask Code' : undefined}
          className={cn(
            'w-full flex items-center rounded-lg text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors',
            collapsed ? 'justify-center p-2' : 'gap-2.5 px-3 py-2',
          )}
        >
          <Code2 className="w-4 h-4 shrink-0" />
          {!collapsed && (
            <>
              <span className="flex-1 text-left">Ask Code</span>
              <kbd className="text-xs bg-muted border border-border/50 rounded px-1.5 py-0.5 font-mono">⌘/</kbd>
            </>
          )}
        </button>
        <button
          onClick={onFixRightAway}
          title={collapsed ? 'Fix Right Away' : undefined}
          className={cn(
            'w-full flex items-center rounded-lg text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors',
            collapsed ? 'justify-center p-2' : 'gap-2.5 px-3 py-2',
          )}
        >
          <Zap className="w-4 h-4 shrink-0" />
          {!collapsed && (
            <>
              <span className="flex-1 text-left">Fix Right Away</span>
              <kbd className="text-xs bg-muted border border-border/50 rounded px-1.5 py-0.5 font-mono">⌘⇧F</kbd>
            </>
          )}
        </button>
        <button
          onClick={onSearch}
          title={collapsed ? 'Search' : undefined}
          className={cn(
            'w-full flex items-center rounded-lg text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors',
            collapsed ? 'justify-center p-2' : 'gap-2.5 px-3 py-2',
          )}
        >
          <Search className="w-4 h-4 shrink-0" />
          {!collapsed && (
            <>
              <span className="flex-1 text-left">Search</span>
              <kbd className="text-xs bg-muted border border-border/50 rounded px-1.5 py-0.5 font-mono">⌘K</kbd>
            </>
          )}
        </button>
      </div>
    </aside>
  )
}
