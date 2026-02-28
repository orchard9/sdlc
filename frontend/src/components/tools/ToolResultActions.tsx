import type { ElementType } from 'react'

export interface ToolResultAction {
  id: string
  icon: ElementType
  label: string
  onClick: () => void
  disabled?: boolean
}

export function ToolResultActions({ actions }: { actions: ToolResultAction[] }) {
  return (
    <div className="flex items-center gap-2 border-t border-border/30 pt-3 mt-3">
      {actions.map(action => {
        const Icon = action.icon
        return (
          <button
            key={action.id}
            onClick={action.onClick}
            disabled={action.disabled}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground hover:bg-muted/60 rounded-lg border border-border/50 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
          >
            <Icon className="w-3.5 h-3.5" />
            {action.label}
          </button>
        )
      })}
    </div>
  )
}
