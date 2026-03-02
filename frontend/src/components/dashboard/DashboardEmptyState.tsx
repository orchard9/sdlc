import { useNavigate } from 'react-router-dom'
import { Lightbulb } from 'lucide-react'

export function DashboardEmptyState() {
  const navigate = useNavigate()
  return (
    <div className="flex flex-col items-center justify-center py-20 text-center">
      <div className="max-w-md space-y-4">
        <p className="text-lg font-semibold leading-snug">
          SDLC turns ideas into shipped software.
        </p>
        <p className="text-sm text-muted-foreground">
          Describe what you're building — agents will build it in parallel waves.
        </p>
        <button
          onClick={() => navigate('/ponder?new=1')}
          className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors"
        >
          <Lightbulb className="w-4 h-4" />
          New Ponder
        </button>
      </div>
    </div>
  )
}
