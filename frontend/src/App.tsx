import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { AgentRunProvider } from '@/contexts/AgentRunContext'
import { AppShell } from '@/components/layout/AppShell'
import { Dashboard } from '@/pages/Dashboard'
import { FeatureDetail } from '@/pages/FeatureDetail'
import { FeaturesPage } from '@/pages/FeaturesPage'
import { MilestonesPage } from '@/pages/MilestonesPage'
import { MilestoneDetail } from '@/pages/MilestoneDetail'
import { PonderPage } from '@/pages/PonderPage'
import { SettingsPage } from '@/pages/SettingsPage'

export default function App() {
  return (
    <BrowserRouter>
      <AgentRunProvider>
        <AppShell>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/features" element={<FeaturesPage />} />
            <Route path="/features/:slug" element={<FeatureDetail />} />
            <Route path="/milestones" element={<MilestonesPage />} />
            <Route path="/milestones/archive" element={<MilestonesPage filter="released" />} />
            <Route path="/milestones/:slug" element={<MilestoneDetail />} />
            <Route path="/ponder" element={<PonderPage />} />
            <Route path="/ponder/:slug" element={<PonderPage />} />
            <Route path="/config" element={<SettingsPage />} />
          </Routes>
        </AppShell>
      </AgentRunProvider>
    </BrowserRouter>
  )
}
