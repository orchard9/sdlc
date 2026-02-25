import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { AppShell } from '@/components/layout/AppShell'
import { Dashboard } from '@/pages/Dashboard'
import { FeatureDetail } from '@/pages/FeatureDetail'
import { FeaturesPage } from '@/pages/FeaturesPage'
import { MilestonesPage } from '@/pages/MilestonesPage'
import { MilestoneDetail } from '@/pages/MilestoneDetail'
import { ConfigPage } from '@/pages/ConfigPage'

export default function App() {
  return (
    <BrowserRouter>
      <AppShell>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/features" element={<FeaturesPage />} />
          <Route path="/features/:slug" element={<FeatureDetail />} />
          <Route path="/milestones" element={<MilestonesPage />} />
          <Route path="/milestones/:slug" element={<MilestoneDetail />} />
<Route path="/config" element={<ConfigPage />} />
        </Routes>
      </AppShell>
    </BrowserRouter>
  )
}
