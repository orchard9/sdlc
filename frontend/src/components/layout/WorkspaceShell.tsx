import { cn } from '@/lib/utils'

interface WorkspaceShellProps {
  /** Content for the fixed-width left list pane. */
  listPane: React.ReactNode
  /** Content for the flexible right detail pane. */
  detailPane: React.ReactNode
  /** When true (a detail slug is active), the right pane is visible and the left pane hides on mobile. */
  showDetail: boolean
  /** Tailwind width class for the left pane. Defaults to "w-72". */
  listWidth?: string
}

/**
 * WorkspaceShell — shared two-pane list/detail page layout component.
 *
 * Renders a responsive shell with a fixed-width left list pane and a flexible
 * right detail pane. On mobile, only one pane is shown at a time based on
 * whether a detail item is selected (showDetail).
 */
export function WorkspaceShell({
  listPane,
  detailPane,
  showDetail,
  listWidth = 'w-72',
}: WorkspaceShellProps) {
  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 flex min-h-0">
        {/* Left pane: list/navigation */}
        <div className={cn(
          listWidth,
          'shrink-0 border-r border-border flex flex-col bg-card',
          showDetail ? 'hidden lg:flex' : 'flex',
        )}>
          {listPane}
        </div>

        {/* Right pane: detail */}
        <div className={cn(
          'flex-1 min-w-0',
          showDetail ? 'flex flex-col' : 'hidden lg:flex lg:flex-col',
        )}>
          {detailPane}
        </div>
      </div>
    </div>
  )
}
