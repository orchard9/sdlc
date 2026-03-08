import { useCallback, useEffect, useState } from 'react'

export interface GitCommit {
  hash: string
  short_hash: string
  author_name: string
  author_email: string
  date: string
  subject: string
  body: string
}

interface GitLogResponse {
  commits: GitCommit[]
  page: number
  per_page: number
  total_commits: number
}

interface UseGitLogReturn {
  commits: GitCommit[]
  loading: boolean
  error: string | null
  hasMore: boolean
  loadMore: () => Promise<void>
  refetch: () => Promise<void>
}

export function useGitLog(perPage = 25): UseGitLogReturn {
  const [commits, setCommits] = useState<GitCommit[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [page, setPage] = useState(1)
  const [totalCommits, setTotalCommits] = useState(0)

  const fetchPage = useCallback(async (pageNum: number, append: boolean) => {
    if (!append) setLoading(true)
    try {
      const res = await fetch(`/api/git/log?page=${pageNum}&per_page=${perPage}`)
      if (res.status === 404) {
        setError('not_available')
        return
      }
      if (!res.ok) throw new Error(res.statusText)
      const data: GitLogResponse | { error: string } = await res.json()
      if ('error' in data) {
        setError(data.error)
        return
      }
      setTotalCommits(data.total_commits)
      setCommits(prev => append ? [...prev, ...data.commits] : data.commits)
      setPage(pageNum)
      setError(null)
    } catch {
      setError('fetch_failed')
    } finally {
      setLoading(false)
    }
  }, [perPage])

  // Initial fetch on mount
  useEffect(() => {
    fetchPage(1, false)
  }, [fetchPage])

  const hasMore = commits.length < totalCommits

  const loadMore = useCallback(async () => {
    if (!hasMore) return
    await fetchPage(page + 1, true)
  }, [fetchPage, page, hasMore])

  const refetch = useCallback(async () => {
    setCommits([])
    setPage(1)
    await fetchPage(1, false)
  }, [fetchPage])

  return { commits, loading, error, hasMore, loadMore, refetch }
}
