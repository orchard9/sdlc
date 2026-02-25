import type { Phase } from './types'

export const PHASES: Phase[] = [
  'draft',
  'specified',
  'planned',
  'ready',
  'implementation',
  'review',
  'audit',
  'qa',
  'merge',
  'released',
]

export function phaseIndex(phase: Phase): number {
  return PHASES.indexOf(phase)
}

export function phaseLabel(phase: Phase): string {
  return phase.charAt(0).toUpperCase() + phase.slice(1)
}

export function phaseColor(phase: Phase): string {
  const colors: Record<Phase, string> = {
    draft: 'bg-[var(--color-phase-draft)]',
    specified: 'bg-[var(--color-phase-specified)]',
    planned: 'bg-[var(--color-phase-planned)]',
    ready: 'bg-[var(--color-phase-ready)]',
    implementation: 'bg-[var(--color-phase-implementation)]',
    review: 'bg-[var(--color-phase-review)]',
    audit: 'bg-[var(--color-phase-audit)]',
    qa: 'bg-[var(--color-phase-qa)]',
    merge: 'bg-[var(--color-phase-merge)]',
    released: 'bg-[var(--color-phase-released)]',
  }
  return colors[phase] ?? 'bg-muted'
}
