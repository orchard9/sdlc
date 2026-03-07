/**
 * Maps a run's type and target to the entity detail route, or null if no page exists.
 */
export function runTargetRoute(runType: string, target: string): string | null {
  if (!target) return null
  switch (runType) {
    case 'feature':
      return `/features/${target}`
    case 'milestone_uat':
    case 'milestone_prepare':
    case 'milestone_run_wave':
      return `/milestones/${target}`
    case 'ponder':
      return `/ponder/${target}`
    case 'investigation':
      return `/investigations/${target}`
    default:
      return null
  }
}
