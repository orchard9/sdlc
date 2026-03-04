import { useState } from 'react'
import { useAgentRuns } from '@/contexts/AgentRunContext'

export function useMilestoneUatRun(slug: string) {
  const key = `milestone-uat:${slug}`
  const { isRunning, startRun, focusRun, getRunForKey } = useAgentRuns()
  const running = isRunning(key)
  const activeRun = getRunForKey(key)
  const [modalOpen, setModalOpen] = useState(false)

  const handleStart = () => {
    startRun({
      key,
      runType: 'milestone_uat',
      target: slug,
      label: `UAT: ${slug}`,
      startUrl: `/api/milestone/${encodeURIComponent(slug)}/uat`,
      stopUrl: `/api/milestone/${encodeURIComponent(slug)}/uat/stop`,
    })
  }

  const handleFocus = () => {
    if (activeRun) focusRun(activeRun.id)
  }

  return { running, handleStart, handleFocus, modalOpen, setModalOpen }
}
