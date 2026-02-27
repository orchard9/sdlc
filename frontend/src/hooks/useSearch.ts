import { useState, useEffect, useRef } from 'react'
import { api } from '@/api/client'
import type { QuerySearchResult } from '@/lib/types'

export function useSearch() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<QuerySearchResult[]>([])
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
          setResults(res.results)
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
