import { useState, useEffect, useRef } from 'react'
import { api } from '@/api/client'
import type { QuerySearchResult, QueryPonderSearchResult } from '@/lib/types'

export interface SearchResultItem {
  kind: 'feature' | 'ponder'
  slug: string
  title: string
  status: string // phase for features, status for ponders
  score: number
}

export function useSearch() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResultItem[]>([])
  const [loading, setLoading] = useState(false)
  const [parseError, setParseError] = useState<string | null>(null)
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const abortRef = useRef<AbortController | null>(null)

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current)
    // Abort any in-flight request from a previous query
    abortRef.current?.abort()

    if (!query.trim()) {
      setResults([])
      setParseError(null)
      setLoading(false)
      return
    }

    setLoading(true)
    debounceRef.current = setTimeout(async () => {
      const controller = new AbortController()
      abortRef.current = controller
      try {
        const res = await api.querySearch(query, 10)
        // Only apply results if this request was not superseded
        if (!controller.signal.aborted) {
          const featureItems: SearchResultItem[] = res.results.map((r: QuerySearchResult) => ({
            kind: 'feature' as const,
            slug: r.slug,
            title: r.title,
            status: r.phase,
            score: r.score,
          }))
          const ponderItems: SearchResultItem[] = (res.ponder_results ?? []).map((r: QueryPonderSearchResult) => ({
            kind: 'ponder' as const,
            slug: r.slug,
            title: r.title,
            status: r.status,
            score: r.score,
          }))
          // Merge and sort by score descending
          const merged = [...featureItems, ...ponderItems].sort((a, b) => b.score - a.score)
          setResults(merged)
          setParseError(res.parse_error)
        }
      } catch {
        if (!controller.signal.aborted) {
          setResults([])
          setParseError(null)
        }
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false)
        }
      }
    }, 200)

    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current)
      abortRef.current?.abort()
    }
  }, [query])

  return { query, setQuery, results, loading, parseError }
}
