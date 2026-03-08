import { lazy, Suspense, useEffect, useState } from 'react'
import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { AgentRunProvider } from '@/contexts/AgentRunContext'
import { SseProvider } from '@/contexts/SseContext'
import { AppShell } from '@/components/layout/AppShell'
import { HubPage } from '@/pages/HubPage'

// Lazy-loaded page components — each becomes a separate chunk
const Dashboard = lazy(() => import('@/pages/Dashboard').then(m => ({ default: m.Dashboard })))
const FeatureDetail = lazy(() => import('@/pages/FeatureDetail').then(m => ({ default: m.FeatureDetail })))
const FeaturesPage = lazy(() => import('@/pages/FeaturesPage').then(m => ({ default: m.FeaturesPage })))
const MilestonesPage = lazy(() => import('@/pages/MilestonesPage').then(m => ({ default: m.MilestonesPage })))
const MilestoneDetail = lazy(() => import('@/pages/MilestoneDetail').then(m => ({ default: m.MilestoneDetail })))
const PonderPage = lazy(() => import('@/pages/PonderPage').then(m => ({ default: m.PonderPage })))
const InvestigationPage = lazy(() => import('@/pages/InvestigationPage').then(m => ({ default: m.InvestigationPage })))
const EvolvePage = lazy(() => import('@/pages/EvolvePage').then(m => ({ default: m.EvolvePage })))
const GuidelinePage = lazy(() => import('@/pages/GuidelinePage').then(m => ({ default: m.GuidelinePage })))
const KnowledgePage = lazy(() => import('@/pages/KnowledgePage').then(m => ({ default: m.KnowledgePage })))
const SettingsPage = lazy(() => import('@/pages/SettingsPage').then(m => ({ default: m.SettingsPage })))
const SecretsPage = lazy(() => import('@/pages/SecretsPage').then(m => ({ default: m.SecretsPage })))
const ToolsPage = lazy(() => import('@/pages/ToolsPage').then(m => ({ default: m.ToolsPage })))
const FeedbackPage = lazy(() => import('@/pages/FeedbackPage').then(m => ({ default: m.FeedbackPage })))
const NetworkPage = lazy(() => import('@/pages/NetworkPage').then(m => ({ default: m.NetworkPage })))
const VisionPage = lazy(() => import('@/pages/VisionPage').then(m => ({ default: m.VisionPage })))
const ArchitecturePage = lazy(() => import('@/pages/ArchitecturePage').then(m => ({ default: m.ArchitecturePage })))
const DocsPage = lazy(() => import('@/pages/DocsPage').then(m => ({ default: m.DocsPage })))
const AgentsPage = lazy(() => import('@/pages/AgentsPage').then(m => ({ default: m.AgentsPage })))
const SetupPage = lazy(() => import('@/pages/SetupPage').then(m => ({ default: m.SetupPage })))
const ActionsPage = lazy(() => import('@/pages/ActionsPage').then(m => ({ default: m.ActionsPage })))
const ThreadsPage = lazy(() => import('@/pages/ThreadsPage').then(m => ({ default: m.ThreadsPage })))
const RunsPage = lazy(() => import('@/pages/RunsPage').then(m => ({ default: m.RunsPage })))
const SpikePage = lazy(() => import('@/pages/SpikePage').then(m => ({ default: m.SpikePage })))
const GitPage = lazy(() => import('@/pages/GitPage'))

function PageSpinner() {
  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="w-6 h-6 border-2 border-border border-t-primary rounded-full animate-spin" />
    </div>
  )
}

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
          <Suspense fallback={<PageSpinner />}>
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
            <Route path="/git" element={<GitPage />} />
            <Route path="/git/*" element={<GitPage />} />
            <Route path="/agents" element={<AgentsPage />} />
            <Route path="/actions" element={<ActionsPage />} />
            <Route path="/runs" element={<RunsPage />} />
            <Route path="/config" element={<SettingsPage />} />
            <Route path="/docs" element={<DocsPage />} />
            <Route path="/docs/:section" element={<DocsPage />} />
          </Routes>
          </Suspense>
        </AppShell>
      </AgentRunProvider>
      </SseProvider>
    </BrowserRouter>
  )
}
