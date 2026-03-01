import { useState, useEffect, type ReactNode } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { Sidebar } from './Sidebar'
import { AgentPanel } from './AgentPanel'
import { AgentPanelFab } from './AgentPanelFab'
import { BottomTabBar } from './BottomTabBar'
import { SearchModal } from '@/components/shared/SearchModal'
import { FixRightAwayModal } from '@/components/shared/FixRightAwayModal'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { PanelRightOpen, ChevronLeft, MoreHorizontal } from 'lucide-react'

const DETAIL_BASES = ['/ponder/', '/investigations/', '/evolve/']

const PATH_LABELS: Record<string, string> = {
  '/': 'Dashboard',
  '/milestones': 'Milestones',
  '/features': 'Features',
  '/milestones/archive': 'Archive',
  '/feedback': 'Feedback',
  '/ponder': 'Ponder',
  '/investigations': 'Root Cause',
  '/evolve': 'Evolve',
  '/tools': 'Tools',
  '/secrets': 'Secrets',
  '/agents': 'Agents',
  '/network': 'Network',
  '/vision': 'Vision',
  '/architecture': 'Architecture',
}

function titleFromPath(pathname: string): string {
  if (PATH_LABELS[pathname]) return PATH_LABELS[pathname]
  // Match prefix for detail pages
  for (const [path, label] of Object.entries(PATH_LABELS)) {
    if (path !== '/' && pathname.startsWith(path + '/')) return label
  }
  return 'SDLC'
}

interface AppShellProps {
  children: ReactNode
}

export function AppShell({ children }: AppShellProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const [searchOpen, setSearchOpen] = useState(false)
  const [fixOpen, setFixOpen] = useState(false)
  const { panelOpen, setPanelOpen } = useAgentRuns()
  const location = useLocation()
  const navigate = useNavigate()

  const isDetailView = DETAIL_BASES.some(base => location.pathname.startsWith(base))

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault()
        setSearchOpen(prev => !prev)
      }
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'f') {
        e.preventDefault()
        setFixOpen(prev => !prev)
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [])

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Mobile overlay */}
      {sidebarOpen && (
        <div
          className="fixed inset-0 z-30 bg-black/50 md:hidden"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      {/* Sidebar — hidden on mobile by default, shown when toggled */}
      <div
        className={`
          fixed inset-y-0 left-0 z-40 md:static md:z-auto
          transition-transform duration-200 ease-in-out
          ${sidebarOpen ? 'translate-x-0' : '-translate-x-full'}
          md:translate-x-0
        `}
      >
        <Sidebar onNavigate={() => setSidebarOpen(false)} onSearch={() => setSearchOpen(true)} onFixRightAway={() => setFixOpen(true)} />
      </div>

      <div className="flex-1 flex flex-col overflow-hidden min-w-0">
        {/* Mobile header */}
        <header className="flex items-center gap-3 px-4 py-3 border-b border-border md:hidden">
          {isDetailView ? (
            <button
              onClick={() => navigate(-1)}
              className="p-1.5 rounded-lg hover:bg-accent transition-colors"
              aria-label="Go back"
            >
              <ChevronLeft className="w-5 h-5" />
            </button>
          ) : (
            <button
              onClick={() => setSidebarOpen(true)}
              className="p-1.5 rounded-lg hover:bg-accent transition-colors"
              aria-label="Open menu"
            >
              <MoreHorizontal className="w-5 h-5" />
            </button>
          )}
          <span className="text-sm font-semibold tracking-tight">{titleFromPath(location.pathname)}</span>
        </header>

        <main className="flex-1 overflow-y-auto pb-11 md:pb-0">
          {children}
        </main>
      </div>

      {/* Desktop agent panel */}
      <AgentPanel />

      {/* Panel open button when collapsed (desktop only) */}
      {!panelOpen && (
        <button
          onClick={() => setPanelOpen(true)}
          className="hidden md:flex items-center justify-center w-8 border-l border-border bg-background hover:bg-muted transition-colors shrink-0"
          aria-label="Open agent panel"
        >
          <PanelRightOpen className="w-4 h-4 text-muted-foreground" />
        </button>
      )}

      {/* Mobile FAB + drawer */}
      <AgentPanelFab />

      {/* Mobile bottom tab bar */}
      <BottomTabBar onMore={() => setSidebarOpen(true)} />

      <SearchModal open={searchOpen} onClose={() => setSearchOpen(false)} />
      <FixRightAwayModal open={fixOpen} onClose={() => setFixOpen(false)} />
    </div>
  )
}
