import { useState, useEffect, type ReactNode } from 'react'
import { Sidebar } from './Sidebar'
import { AgentPanel } from './AgentPanel'
import { AgentPanelFab } from './AgentPanelFab'
import { SearchModal } from '@/components/shared/SearchModal'
import { useAgentRuns } from '@/contexts/AgentRunContext'
import { Menu, X, PanelRightOpen } from 'lucide-react'

interface AppShellProps {
  children: ReactNode
}

export function AppShell({ children }: AppShellProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const [searchOpen, setSearchOpen] = useState(false)
  const { panelOpen, setPanelOpen } = useAgentRuns()

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault()
        setSearchOpen(prev => !prev)
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
        <Sidebar onNavigate={() => setSidebarOpen(false)} onSearch={() => setSearchOpen(true)} />
      </div>

      <div className="flex-1 flex flex-col overflow-hidden min-w-0">
        {/* Mobile header with hamburger */}
        <header className="flex items-center gap-3 px-4 py-3 border-b border-border md:hidden">
          <button
            onClick={() => setSidebarOpen(prev => !prev)}
            className="p-1.5 rounded-lg hover:bg-accent transition-colors"
            aria-label={sidebarOpen ? 'Close menu' : 'Open menu'}
          >
            {sidebarOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
          </button>
          <span className="text-sm font-semibold tracking-tight">SDLC</span>
        </header>

        <main className="flex-1 overflow-y-auto p-6">
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

      <SearchModal open={searchOpen} onClose={() => setSearchOpen(false)} />
    </div>
  )
}
