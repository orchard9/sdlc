import { Link, useLocation } from 'react-router-dom'
import { LayoutDashboard, Lightbulb, Wrench, Target, MoreHorizontal } from 'lucide-react'
import { cn } from '@/lib/utils'

const TABS = [
  {
    label: 'Work',
    icon: LayoutDashboard,
    roots: ['/milestones', '/features'],
    exact: '/',
  },
  {
    label: 'Plan',
    icon: Lightbulb,
    roots: ['/feedback', '/ponder', '/investigations', '/evolve', '/guidelines'],
    exact: null,
  },
  {
    label: 'Setup',
    icon: Wrench,
    roots: ['/tools', '/secrets', '/agents'],
    exact: null,
  },
  {
    label: 'Project',
    icon: Target,
    roots: ['/vision', '/architecture', '/network'],
    exact: null,
  },
] as const

interface BottomTabBarProps {
  onMore: () => void
}

export function BottomTabBar({ onMore }: BottomTabBarProps) {
  const { pathname } = useLocation()

  function isActive(tab: typeof TABS[number]) {
    if (tab.exact && pathname === tab.exact) return true
    return tab.roots.some(r => pathname.startsWith(r))
  }

  return (
    <nav className="md:hidden fixed bottom-0 left-0 right-0 h-11 bg-card border-t border-border z-30 flex items-stretch">
      {TABS.map(tab => {
        const active = isActive(tab)
        const to = tab.exact ?? tab.roots[0]
        return (
          <Link
            key={tab.label}
            to={to}
            className={cn(
              'flex-1 flex flex-col items-center justify-center gap-0.5 text-[10px] font-medium transition-colors',
              active ? 'text-accent-foreground' : 'text-muted-foreground'
            )}
          >
            <tab.icon className={cn('w-4 h-4', active && 'stroke-[2.25]')} />
            {tab.label}
          </Link>
        )
      })}

      <button
        onClick={onMore}
        className="flex-1 flex flex-col items-center justify-center gap-0.5 text-[10px] font-medium text-muted-foreground transition-colors hover:text-foreground"
      >
        <MoreHorizontal className="w-4 h-4" />
        More
      </button>
    </nav>
  )
}
