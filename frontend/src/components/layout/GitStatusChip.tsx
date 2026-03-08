import { cn } from '@/lib/utils'
import { GitCommitHorizontal } from 'lucide-react'
import { useGitStatus } from '@/hooks/useGitStatus'
import { useCallback, useEffect, useRef, useState } from 'react'
import { GitDetailsPopover } from './GitDetailsPopover'

interface GitStatusChipProps {
  collapsed: boolean
}

function severityClass(severity: 'green' | 'yellow' | 'red'): string {
  switch (severity) {
    case 'green': return 'bg-emerald-500 shadow-[0_0_4px_rgba(16,185,129,0.4)]'
    case 'yellow': return 'bg-amber-500 shadow-[0_0_4px_rgba(245,158,11,0.4)]'
    case 'red': return 'bg-red-500 shadow-[0_0_4px_rgba(239,68,68,0.4)]'
  }
}

function summaryText(s: { branch: string; dirty_count: number; staged_count: number; ahead: number; has_conflicts: boolean }): string {
  if (s.has_conflicts) return `${s.branch} \u2014 conflicts`
  if (s.dirty_count > 0) return `${s.branch} \u2014 ${s.dirty_count} modified`
  if (s.staged_count > 0) return `${s.branch} \u2014 ${s.staged_count} staged`
  if (s.ahead > 0) return `${s.branch} \u2014 ${s.ahead} ahead`
  return `${s.branch} \u2014 clean`
}

export function GitStatusChip({ collapsed }: GitStatusChipProps) {
  const { status, loading, error, refetch } = useGitStatus()
  const [committing, setCommitting] = useState(false)
  const [popoverOpen, setPopoverOpen] = useState(false)
  const leaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const chipRef = useRef<HTMLDivElement>(null)

  const handleCommit = useCallback(async (e: React.MouseEvent) => {
    e.stopPropagation()
    if (committing) return
    setCommitting(true)
    try {
      const res = await fetch('/api/git/commit', { method: 'POST', headers: { 'Content-Type': 'application/json' } })
      if (!res.ok) {
        console.warn('Commit failed:', res.statusText)
      }
      await refetch()
    } catch (err) {
      console.warn('Commit error:', err)
    } finally {
      setCommitting(false)
    }
  }, [committing, refetch])

  // Click outside to dismiss
  useEffect(() => {
    if (!popoverOpen) return
    const handleClickOutside = (e: MouseEvent) => {
      if (chipRef.current && !chipRef.current.contains(e.target as Node)) {
        setPopoverOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [popoverOpen])

  // Clear leave timer on unmount
  useEffect(() => {
    return () => {
      if (leaveTimerRef.current) clearTimeout(leaveTimerRef.current)
    }
  }, [])

  const handleMouseEnter = useCallback(() => {
    if (leaveTimerRef.current) {
      clearTimeout(leaveTimerRef.current)
      leaveTimerRef.current = null
    }
    if (status) setPopoverOpen(true)
  }, [status])

  const handleMouseLeave = useCallback(() => {
    leaveTimerRef.current = setTimeout(() => {
      setPopoverOpen(false)
    }, 150)
  }, [])

  const handleClick = useCallback(() => {
    if (status) setPopoverOpen(prev => !prev)
  }, [status])

  // Determine dot color and tooltip
  const dotClass = status ? severityClass(status.severity) : 'bg-muted-foreground/30'
  const tooltip = error
    ? 'Git status unavailable'
    : loading
      ? 'Loading git status\u2026'
      : status
        ? summaryText(status)
        : 'Git status unavailable'

  return (
    <div
      ref={chipRef}
      className={cn(
        'relative flex items-center rounded-lg text-sm transition-colors cursor-pointer',
        collapsed ? 'justify-center p-2' : 'gap-2.5 px-3 py-2',
        'text-muted-foreground hover:bg-accent/50',
      )}
      title={collapsed ? tooltip : undefined}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onClick={handleClick}
    >
      {/* Severity dot */}
      <span className={cn('w-2 h-2 rounded-full shrink-0', dotClass)} />

      {/* Text + commit button (expanded only) */}
      {!collapsed && (
        <>
          <span className="flex-1 text-xs truncate">{tooltip}</span>
          {status && status.staged_count > 0 && (
            <button
              onClick={handleCommit}
              disabled={committing}
              className="flex items-center gap-1 px-1.5 py-0.5 rounded border border-border text-[11px] text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors disabled:opacity-50"
              title="Commit staged changes"
            >
              <GitCommitHorizontal className="w-3 h-3" />
              {committing ? '\u2026' : 'Commit'}
            </button>
          )}
        </>
      )}

      {/* Details popover */}
      {popoverOpen && status && (
        <GitDetailsPopover status={status} collapsed={collapsed} />
      )}
    </div>
  )
}
