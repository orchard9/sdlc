import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { AgentRunProvider } from '@/contexts/AgentRunContext'
import { AppShell } from '@/components/layout/AppShell'
import { Dashboard } from '@/pages/Dashboard'
import { FeatureDetail } from '@/pages/FeatureDetail'
import { FeaturesPage } from '@/pages/FeaturesPage'
import { MilestonesPage } from '@/pages/MilestonesPage'
import { MilestoneDetail } from '@/pages/MilestoneDetail'
import { PonderPage } from '@/pages/PonderPage'
import { InvestigationPage } from '@/pages/InvestigationPage'
import { EvolvePage } from '@/pages/EvolvePage'
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

export default function App() {
  return (
    <BrowserRouter>
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
            <Route path="/milestones/archive" element={<MilestonesPage filter="released" />} />
            <Route path="/milestones/:slug" element={<MilestoneDetail />} />
            <Route path="/ponder" element={<PonderPage />} />
            <Route path="/ponder/:slug" element={<PonderPage />} />
            <Route path="/investigations" element={<InvestigationPage />} />
            <Route path="/investigations/:slug" element={<InvestigationPage />} />
            <Route path="/evolve" element={<EvolvePage />} />
            <Route path="/evolve/:slug" element={<EvolvePage />} />
            <Route path="/secrets" element={<SecretsPage />} />
            <Route path="/tools" element={<ToolsPage />} />
            <Route path="/feedback" element={<FeedbackPage />} />
            <Route path="/network" element={<NetworkPage />} />
            <Route path="/agents" element={<AgentsPage />} />
            <Route path="/config" element={<SettingsPage />} />
            <Route path="/docs" element={<DocsPage />} />
            <Route path="/docs/:section" element={<DocsPage />} />
          </Routes>
        </AppShell>
      </AgentRunProvider>
    </BrowserRouter>
  )
}
