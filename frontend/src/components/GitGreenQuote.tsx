import { cn } from '@/lib/utils'
import { getWeeklyQuote, type Quote } from '@/lib/quotes'

interface GitGreenQuoteProps {
  /** Override the displayed quote (defaults to the weekly rotation). */
  quote?: Quote
  className?: string
}

/**
 * Renders a motivational quote when the git working tree is clean.
 * Designed to sit inside the git status chip area.
 */
export function GitGreenQuote({ quote, className }: GitGreenQuoteProps) {
  const q = quote ?? getWeeklyQuote()

  return (
    <div className={cn('py-1', className)}>
      <p className="text-xs italic leading-relaxed text-emerald-300/70">
        &ldquo;{q.text}&rdquo;
      </p>
      <p className="mt-1 text-right text-[11px] text-emerald-400/50">
        &mdash; {q.author}
      </p>
    </div>
  )
}
