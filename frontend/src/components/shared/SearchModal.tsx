import { useEffect, useId, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useSearch } from '@/hooks/useSearch'
import { StatusBadge } from '@/components/shared/StatusBadge'
import { cn } from '@/lib/utils'
import { Lightbulb } from 'lucide-react'

interface SearchModalProps {
  open: boolean
  onClose: () => void
}

export function SearchModal({ open, onClose }: SearchModalProps) {
  const navigate = useNavigate()
  const { query, setQuery, results, loading } = useSearch()
  const [highlightedIndex, setHighlightedIndex] = useState(0)
  const listboxId = useId()
  const getOptionId = (index: number) => `${listboxId}-option-${index}`

  // Reset state when modal opens
  useEffect(() => {
    if (open) {
      setQuery('')
      setHighlightedIndex(0)
    }
  }, [open, setQuery])

  // Reset highlighted index when results change
  useEffect(() => {
    setHighlightedIndex(0)
  }, [results])

  const navigateToResult = (index: number) => {
    const result = results[index]
    if (!result) return
    const path = result.kind === 'ponder'
      ? `/ponder/${result.slug}`
      : `/features/${result.slug}`
    navigate(path)
    onClose()
  }

  // Keyboard handling
  useEffect(() => {
    if (!open) return

    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose()
      } else if (e.key === 'ArrowDown') {
        e.preventDefault()
        if (results.length > 0) {
          setHighlightedIndex(prev => Math.min(prev + 1, results.length - 1))
        }
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        setHighlightedIndex(prev => Math.max(prev - 1, 0))
      } else if (e.key === 'Enter') {
        navigateToResult(highlightedIndex)
      }
    }

    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [open, results, highlightedIndex, navigate, onClose])

  if (!open) return null

  const showResults = results.length > 0
  const showEmpty = query.trim() && !loading && results.length === 0
  const showLoading = loading && query.trim()

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Search features and ideas"
      className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/60"
      onClick={onClose}
    >
      <div
        className="w-full max-w-xl mx-4 bg-card border border-border rounded-xl shadow-2xl overflow-hidden"
        onClick={e => e.stopPropagation()}
      >
        <input
          // autoFocus is intentional: the modal is a deliberate user action
          // eslint-disable-next-line jsx-a11y/no-autofocus
          autoFocus
          type="text"
          role="combobox"
          aria-expanded={showResults}
          aria-controls={listboxId}
          aria-activedescendant={showResults ? getOptionId(highlightedIndex) : undefined}
          aria-autocomplete="list"
          value={query}
          onChange={e => setQuery(e.target.value)}
          placeholder="Search features and ideas..."
          className="w-full px-4 py-3 text-sm bg-transparent outline-none placeholder:text-muted-foreground"
        />

        {(showResults || showEmpty || showLoading) && (
          <div className="border-t border-border" />
        )}

        {showLoading && (
          <p className="text-xs text-muted-foreground px-4 py-3">Searching...</p>
        )}

        {showEmpty && (
          <p className="px-4 py-6 text-center text-sm text-muted-foreground">
            No results for &ldquo;{query}&rdquo;
          </p>
        )}

        {showResults && (
          <ul
            id={listboxId}
            role="listbox"
            aria-label="Search results"
            className="overflow-y-auto max-h-80"
          >
            {results.map((result, index) => (
              <li
                key={`${result.kind}-${result.slug}`}
                id={getOptionId(index)}
                role="option"
                aria-selected={index === highlightedIndex}
              >
                <button
                  className={cn(
                    'w-full text-left px-4 py-2.5 flex items-center gap-3 transition-colors',
                    index === highlightedIndex ? 'bg-accent' : 'hover:bg-accent'
                  )}
                  onMouseEnter={() => setHighlightedIndex(index)}
                  onClick={() => navigateToResult(index)}
                >
                  <span className="flex-1 min-w-0">
                    <span className="text-sm font-medium text-foreground block truncate">
                      {result.title}
                    </span>
                  </span>
                  <span className="flex items-center gap-2 shrink-0">
                    {result.kind === 'ponder' && (
                      <Lightbulb className="w-3.5 h-3.5 text-violet-400" />
                    )}
                    <StatusBadge status={result.status} />
                    <span className="text-xs text-muted-foreground font-mono">{result.slug}</span>
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  )
}
