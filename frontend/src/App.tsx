import { useEffect, useState } from 'react'
import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { AgentRunProvider } from '@/contexts/AgentRunContext'
import { SseProvider } from '@/contexts/SseContext'
import { AppShell } from '@/components/layout/AppShell'
import { Dashboard } from '@/pages/Dashboard'
import { FeatureDetail } from '@/pages/FeatureDetail'
import { FeaturesPage } from '@/pages/FeaturesPage'
import { MilestonesPage } from '@/pages/MilestonesPage'
import { MilestoneDetail } from '@/pages/MilestoneDetail'
import { PonderPage } from '@/pages/PonderPage'
import { InvestigationPage } from '@/pages/InvestigationPage'
import { EvolvePage } from '@/pages/EvolvePage'
import { GuidelinePage } from '@/pages/GuidelinePage'
import { KnowledgePage } from '@/pages/KnowledgePage'
import { SettingsPage } from '@/pages/SettingsPage'
import { SecretsPage } from '@/pages/SecretsPage'
import { ToolsPage } from '@/pages/ToolsPage'
import { FeedbackPage } from '@/pages/FeedbackPage'
import { NetworkPage } from '@/pages/NetworkPage'
import { VisionPage } from '@/pages/VisionPage'
import { ArchitecturePage } from '@/pages/ArchitecturePage'
import { DocsPage } from '@/pages/DocsPage'
import { AgentsPage } from '@/pages/AgentsPage'
import { SetupPage } from '@/pages/SetupPage'
import { ActionsPage } from '@/pages/ActionsPage'
import { ThreadsPage } from '@/pages/ThreadsPage'
import { RunsPage } from '@/pages/RunsPage'
import { HubPage } from '@/pages/HubPage'
import { SpikePage } from '@/pages/SpikePage'

type HubMode = 'loading' | 'hub' | 'normal'

/** Detect whether the server is running in hub mode by probing /api/hub/projects.
 *  200 → hub mode; 503 or any error → normal mode. */
function useHubMode(): HubMode {
  const [mode, setMode] = useState<HubMode>('loading')

  useEffect(() => {
    fetch('/api/hub/projects', { method: 'GET' })
      .then(res => {
        setMode(res.ok ? 'hub' : 'normal')
      })
      .catch(() => setMode('normal'))
  }, [])

  return mode
}

export default function App() {
  const hubMode = useHubMode()

  if (hubMode === 'loading') {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="w-6 h-6 border-2 border-border border-t-primary rounded-full animate-spin" />
      </div>
    )
  }

  if (hubMode === 'hub') {
    return <HubPage />
  }

  return (
    <BrowserRouter>
      <SseProvider>
      <AgentRunProvider>
        <AppShell>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/setup" element={<SetupPage />} />
            <Route path="/vision" element={<VisionPage />} />
            <Route path="/architecture" element={<ArchitecturePage />} />
            <Route path="/features" element={<FeaturesPage />} />
            <Route path="/features/:slug" element={<FeatureDetail />} />
            <Route path="/milestones" element={<MilestonesPage />} />
            <Route path="/milestones/:slug" element={<MilestoneDetail />} />
            <Route path="/ponder" element={<PonderPage />} />
            <Route path="/ponder/:slug" element={<PonderPage />} />
            <Route path="/investigations" element={<InvestigationPage />} />
            <Route path="/investigations/:slug" element={<InvestigationPage />} />
            <Route path="/evolve" element={<EvolvePage />} />
            <Route path="/evolve/:slug" element={<EvolvePage />} />
            <Route path="/guidelines" element={<GuidelinePage />} />
            <Route path="/guidelines/:slug" element={<GuidelinePage />} />
            <Route path="/knowledge" element={<KnowledgePage />} />
            <Route path="/knowledge/:slug" element={<KnowledgePage />} />
            <Route path="/spikes" element={<SpikePage />} />
            <Route path="/spikes/:slug" element={<SpikePage />} />
            <Route path="/secrets" element={<SecretsPage />} />
            <Route path="/tools" element={<ToolsPage />} />
            <Route path="/tools/:toolId" element={<ToolsPage />} />
            <Route path="/feedback" element={<FeedbackPage />} />
            <Route path="/threads" element={<ThreadsPage />} />
            <Route path="/threads/:slug" element={<ThreadsPage />} />
            <Route path="/network" element={<NetworkPage />} />
            <Route path="/agents" element={<AgentsPage />} />
            <Route path="/actions" element={<ActionsPage />} />
            <Route path="/runs" element={<RunsPage />} />
            <Route path="/config" element={<SettingsPage />} />
            <Route path="/docs" element={<DocsPage />} />
            <Route path="/docs/:section" element={<DocsPage />} />
          </Routes>
        </AppShell>
      </AgentRunProvider>
      </SseProvider>
    </BrowserRouter>
  )
}
