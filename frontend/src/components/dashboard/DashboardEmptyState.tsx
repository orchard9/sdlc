import { Link } from 'react-router-dom'
import { Target, Layers, Lightbulb, Plus } from 'lucide-react'

interface SuggestionChipProps {
  icon: React.ReactNode
  label: string
  description: string
  to: string
}

function SuggestionChip({ icon, label, description, to }: SuggestionChipProps) {
  return (
    <Link
      to={to}
      className="flex items-start gap-3 p-3 rounded-lg border border-border bg-card hover:bg-accent transition-colors"
    >
      <span className="mt-0.5 text-muted-foreground shrink-0">{icon}</span>
      <div className="min-w-0">
        <p className="text-sm font-medium leading-snug">{label}</p>
        <p className="text-xs text-muted-foreground mt-0.5">{description}</p>
      </div>
    </Link>
  )
}

interface DashboardEmptyStateProps {
  hasVision: boolean
  hasArch: boolean
}

export function DashboardEmptyState({ hasVision, hasArch }: DashboardEmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center py-20 text-center">
      <div className="max-w-sm w-full space-y-4">
        <div className="space-y-1">
          <p className="text-lg font-semibold leading-snug">
            Ponder turns ideas into shipped software.
          </p>
          <p className="text-sm text-muted-foreground">
            Where do you want to start?
          </p>
        </div>

        <div className="space-y-2 text-left">
          {!hasVision && (
            <SuggestionChip
              icon={<Target className="w-4 h-4" />}
              label="Define Vision"
              description="Agents use this to stay aligned on every decision."
              to="/setup"
            />
          )}
          {!hasArch && (
            <SuggestionChip
              icon={<Layers className="w-4 h-4" />}
              label="Define Architecture"
              description="Gives agents the system map before they write code."
              to="/setup"
            />
          )}
          {hasVision && hasArch && (
            <SuggestionChip
              icon={<Lightbulb className="w-4 h-4" />}
              label="Start a Ponder"
              description="Explore an idea with AI thought partners before committing."
              to="/ponder?new=1"
            />
          )}
          <SuggestionChip
            icon={<Plus className="w-4 h-4" />}
            label="Create a Feature directly"
            description="Skip planning and go straight to implementation."
            to="/features?new=1"
          />
        </div>
      </div>
    </div>
  )
}
